//! Project export/import support for `.owhmi` tar.gz archives.

#![deny(missing_docs)]

use std::io::{Read, Write};
use std::path::{Component, Path};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use openwebhmi_alarm_engine::AlarmJournal;
use openwebhmi_audit_log::{AuditEvent, AuditLog};
use openwebhmi_historian::HistorianStore;
use openwebhmi_project_store::{ArtifactKind, ProjectStore};
use serde::{Deserialize, Serialize};
use tracing::warn;

const MANIFEST_PATH: &str = "manifest.json";
const ARTIFACT_PREFIX: &str = "artifacts/";
const SCHEMA_VERSION: u32 = 1;
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Import conflict behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportMode {
    /// Replace artifacts present in the archive.
    Replace,
    /// Merge artifacts present in the archive into the existing project.
    Merge,
}

/// Optional database snapshots to include in an export.
#[derive(Clone, Default)]
#[non_exhaustive]
pub struct BackupOptions {
    /// Optional historian SQLite snapshot bytes.
    pub historian_sqlite: Option<Vec<u8>>,
    /// Optional alarm journal SQLite snapshot bytes.
    pub alarm_journal_sqlite: Option<Vec<u8>>,
    /// Optional live historian store to snapshot with SQLite's online backup API.
    pub historian_store: Option<HistorianStore>,
    /// Optional live alarm journal to snapshot with SQLite's online backup API.
    pub alarm_journal: Option<AlarmJournal>,
    /// Optional audit journal used to record the export.
    pub audit_log: Option<AuditLog>,
    /// Authenticated user, when called from the gateway.
    pub user: Option<String>,
}

/// Import options.
#[derive(Clone)]
#[non_exhaustive]
pub struct RestoreOptions {
    /// Import mode.
    pub mode: ImportMode,
    /// Optional live historian store to restore when the archive contains `historian.sqlite`.
    pub historian_store: Option<HistorianStore>,
    /// Optional live alarm journal to restore when the archive contains `alarm-journal.sqlite`.
    pub alarm_journal: Option<AlarmJournal>,
    /// Optional audit journal used to record the import.
    pub audit_log: Option<AuditLog>,
    /// Authenticated user, when called from the gateway.
    pub user: Option<String>,
}

impl Default for RestoreOptions {
    fn default() -> Self {
        Self {
            mode: ImportMode::Replace,
            historian_store: None,
            alarm_journal: None,
            audit_log: None,
            user: None,
        }
    }
}

/// Archive manifest stored at `manifest.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BackupManifest {
    /// Archive schema version.
    pub schema_version: u32,
    /// Project id.
    pub project_id: String,
    /// Export timestamp in Unix epoch milliseconds.
    pub exported_at_ms: u64,
    /// Artifact entries in the archive.
    pub artifacts: Vec<ManifestArtifact>,
    /// Whether `historian.sqlite` is present.
    pub includes_historian: bool,
    /// Whether `alarm-journal.sqlite` is present.
    pub includes_alarm_journal: bool,
}

/// One project artifact entry in the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ManifestArtifact {
    /// Archive path.
    pub path: String,
    /// Project-store artifact selector.
    pub kind: ArtifactKind,
}

/// Export one project to a `.owhmi` tar.gz byte vector.
pub fn export_project(
    store: &ProjectStore,
    project_id: &str,
    options: BackupOptions,
) -> anyhow::Result<Vec<u8>> {
    let project = store.load(project_id)?;
    let mut artifacts = Vec::new();
    let mut files = Vec::new();
    for kind in artifact_kinds(&project) {
        let Some(body) = store.read_artifact(project_id, kind.clone())? else {
            continue;
        };
        let path = artifact_archive_path(&kind);
        artifacts.push(ManifestArtifact {
            path: path.clone(),
            kind,
        });
        files.push((path, serde_json::to_vec_pretty(&body)?));
    }

    let historian_sqlite = match (options.historian_sqlite, options.historian_store) {
        (Some(bytes), _) => Some(bytes),
        (None, Some(store)) => Some(snapshot_historian(&store)?),
        (None, None) => None,
    };
    let alarm_journal_sqlite = match (options.alarm_journal_sqlite, options.alarm_journal) {
        (Some(bytes), _) => Some(bytes),
        (None, Some(journal)) => Some(snapshot_alarm_journal(&journal)?),
        (None, None) => None,
    };

    let manifest = BackupManifest {
        schema_version: SCHEMA_VERSION,
        project_id: project_id.to_string(),
        exported_at_ms: now_ms(),
        artifacts,
        includes_historian: historian_sqlite.is_some(),
        includes_alarm_journal: alarm_journal_sqlite.is_some(),
    };

    let mut tar = Vec::new();
    append_tar_file(
        &mut tar,
        MANIFEST_PATH,
        &serde_json::to_vec_pretty(&manifest)?,
    )?;
    for (path, bytes) in files {
        append_tar_file(&mut tar, &path, &bytes)?;
    }
    if let Some(bytes) = historian_sqlite {
        append_tar_file(&mut tar, "historian.sqlite", &bytes)?;
    }
    if let Some(bytes) = alarm_journal_sqlite {
        append_tar_file(&mut tar, "alarm-journal.sqlite", &bytes)?;
    }
    tar.extend([0u8; 1024]);

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&tar)?;
    let archive = encoder.finish()?;

    if let Some(audit_log) = options.audit_log {
        let _ = audit_log.append(
            options.user,
            None,
            None,
            AuditEvent::ProjectExport {
                project_id: project_id.to_string(),
                includes_historian: manifest.includes_historian,
                includes_alarm_journal: manifest.includes_alarm_journal,
                archive_size_bytes: archive.len() as u64,
            },
        );
    }

    Ok(archive)
}

/// Restore a `.owhmi` tar.gz archive into a project store.
///
/// When `RestoreOptions` includes live historian or alarm-journal handles and the
/// archive contains matching SQLite entries, replace mode restores those live
/// stores from the archive snapshots, while merge mode appends rows and ignores
/// historian duplicate `(tag_path, ts_ms)` samples. Historian/alarm restore
/// errors are logged and do not abort project artifact restoration.
pub fn import_project(
    store: &ProjectStore,
    archive: &[u8],
    options: RestoreOptions,
) -> anyhow::Result<BackupManifest> {
    let entries = read_archive(archive)?;
    let manifest_bytes = entries
        .iter()
        .find(|entry| entry.path == MANIFEST_PATH)
        .ok_or_else(|| anyhow::anyhow!("backup archive missing manifest.json"))?;
    let manifest: BackupManifest = serde_json::from_slice(&manifest_bytes.bytes)?;
    if manifest.schema_version != SCHEMA_VERSION {
        anyhow::bail!(
            "unsupported backup schema_version {}; expected {SCHEMA_VERSION}",
            manifest.schema_version
        );
    }

    if options.mode == ImportMode::Replace {
        store.delete_project(&manifest.project_id)?;
    }

    for artifact in &manifest.artifacts {
        validate_archive_path(&artifact.path)?;
        let entry = entries
            .iter()
            .find(|entry| entry.path == artifact.path)
            .ok_or_else(|| anyhow::anyhow!("backup archive missing {}", artifact.path))?;
        let body = serde_json::from_slice(&entry.bytes)?;
        store.save_artifact(&manifest.project_id, artifact.kind.clone(), body)?;
    }

    restore_sqlite_entries(&entries, &manifest, &options);

    if let Some(audit_log) = options.audit_log {
        let _ = audit_log.append(
            options.user,
            None,
            None,
            AuditEvent::ProjectImport {
                project_id: manifest.project_id.clone(),
                mode: format!("{:?}", options.mode),
                archive_size_bytes: archive.len() as u64,
            },
        );
    }

    Ok(manifest)
}

fn restore_sqlite_entries(
    entries: &[ArchiveEntry],
    manifest: &BackupManifest,
    options: &RestoreOptions,
) {
    if manifest.includes_historian
        && let Some(historian_store) = &options.historian_store
        && let Some(entry) = entries
            .iter()
            .find(|entry| entry.path == "historian.sqlite")
        && let Err(err) = restore_historian(historian_store, &entry.bytes, options.mode)
    {
        warn!(error = %err, "failed to restore historian backup entry");
    }

    if manifest.includes_alarm_journal
        && let Some(alarm_journal) = &options.alarm_journal
        && let Some(entry) = entries
            .iter()
            .find(|entry| entry.path == "alarm-journal.sqlite")
        && let Err(err) = restore_alarm_journal(alarm_journal, &entry.bytes, options.mode)
    {
        warn!(error = %err, "failed to restore alarm-journal backup entry");
    }
}

fn artifact_kinds(project: &openwebhmi_project_store::Project) -> Vec<ArtifactKind> {
    let mut kinds = vec![
        ArtifactKind::ProjectMeta,
        ArtifactKind::Tags,
        ArtifactKind::Alarms,
    ];
    kinds.extend(project.views.iter().map(|view| ArtifactKind::View {
        id: view.id.clone(),
    }));
    if project.theme.is_some() {
        kinds.push(ArtifactKind::Theme);
    }
    kinds.extend(project.scripts.iter().flat_map(|script| {
        [
            ArtifactKind::Script {
                id: script.id.clone(),
            },
            ArtifactKind::ScriptSource {
                id: script.id.clone(),
            },
        ]
    }));
    kinds
}

fn artifact_archive_path(kind: &ArtifactKind) -> String {
    match kind {
        ArtifactKind::ProjectMeta => format!("{ARTIFACT_PREFIX}project_meta.json"),
        ArtifactKind::Tags => format!("{ARTIFACT_PREFIX}tags.json"),
        ArtifactKind::Alarms => format!("{ARTIFACT_PREFIX}alarms.json"),
        ArtifactKind::Theme => format!("{ARTIFACT_PREFIX}theme/theme.json"),
        ArtifactKind::View { id } => format!("{ARTIFACT_PREFIX}views/{id}.json"),
        ArtifactKind::Script { id } => format!("{ARTIFACT_PREFIX}scripts/{id}.json"),
        ArtifactKind::ScriptSource { id } => format!("{ARTIFACT_PREFIX}scripts/{id}.source.json"),
    }
}

fn snapshot_historian(store: &HistorianStore) -> anyhow::Result<Vec<u8>> {
    let path = temp_snapshot_path("openwebhmi-historian-snapshot");
    let result = (|| {
        store.backup_to_path(&path)?;
        Ok(std::fs::read(&path)?)
    })();
    let _ = std::fs::remove_file(&path);
    result
}

fn snapshot_alarm_journal(journal: &AlarmJournal) -> anyhow::Result<Vec<u8>> {
    let path = temp_snapshot_path("openwebhmi-alarm-snapshot");
    let result = (|| {
        journal.backup_to_path(&path)?;
        Ok(std::fs::read(&path)?)
    })();
    let _ = std::fs::remove_file(&path);
    result
}

fn restore_historian(store: &HistorianStore, bytes: &[u8], mode: ImportMode) -> anyhow::Result<()> {
    let path = temp_snapshot_path("openwebhmi-historian-restore");
    let result = (|| {
        std::fs::write(&path, bytes)?;
        match mode {
            ImportMode::Replace => store.restore_from_path(&path),
            ImportMode::Merge => store.merge_from_path(&path),
        }
    })();
    let _ = std::fs::remove_file(&path);
    result
}

fn restore_alarm_journal(
    journal: &AlarmJournal,
    bytes: &[u8],
    mode: ImportMode,
) -> anyhow::Result<()> {
    let path = temp_snapshot_path("openwebhmi-alarm-restore");
    let result = (|| {
        std::fs::write(&path, bytes)?;
        match mode {
            ImportMode::Replace => journal.restore_from_path(&path),
            ImportMode::Merge => journal.merge_from_path(&path),
        }
    })();
    let _ = std::fs::remove_file(&path);
    result
}

fn temp_snapshot_path(prefix: &str) -> std::path::PathBuf {
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let unique = format!(
        "{prefix}-{}-{}-{counter}.sqlite",
        std::process::id(),
        now_ms()
    );
    std::env::temp_dir().join(unique)
}

#[derive(Debug)]
struct ArchiveEntry {
    path: String,
    bytes: Vec<u8>,
}

fn read_archive(archive: &[u8]) -> anyhow::Result<Vec<ArchiveEntry>> {
    let mut decoder = GzDecoder::new(archive);
    let mut tar = Vec::new();
    decoder.read_to_end(&mut tar)?;
    let mut offset = 0usize;
    let mut entries = Vec::new();
    while offset + 512 <= tar.len() {
        let header = &tar[offset..offset + 512];
        offset += 512;
        if header.iter().all(|byte| *byte == 0) {
            break;
        }
        let path = tar_string(&header[0..100])?;
        validate_archive_path(&path)?;
        let size = parse_octal(&header[124..136])?;
        let data_end = offset
            .checked_add(size)
            .ok_or_else(|| anyhow::anyhow!("tar entry size overflow"))?;
        if data_end > tar.len() {
            anyhow::bail!("tar entry {path} exceeds archive length");
        }
        entries.push(ArchiveEntry {
            path,
            bytes: tar[offset..data_end].to_vec(),
        });
        offset = data_end + padding(size);
    }
    Ok(entries)
}

fn append_tar_file(out: &mut Vec<u8>, path: &str, bytes: &[u8]) -> anyhow::Result<()> {
    validate_archive_path(path)?;
    if path.len() > 100 {
        anyhow::bail!("archive path too long for ustar header: {path}");
    }
    let mut header = [0u8; 512];
    header[0..path.len()].copy_from_slice(path.as_bytes());
    write_octal(&mut header[100..108], 0o644);
    write_octal(&mut header[108..116], 0);
    write_octal(&mut header[116..124], 0);
    write_octal(&mut header[124..136], bytes.len() as u64);
    write_octal(&mut header[136..148], now_ms() / 1000);
    header[148..156].fill(b' ');
    header[156] = b'0';
    header[257..263].copy_from_slice(b"ustar\0");
    header[263..265].copy_from_slice(b"00");
    let checksum = header.iter().map(|byte| *byte as u64).sum::<u64>();
    write_checksum(&mut header[148..156], checksum);
    out.extend(header);
    out.extend(bytes);
    out.extend(std::iter::repeat_n(0, padding(bytes.len())));
    Ok(())
}

fn validate_archive_path(path: &str) -> anyhow::Result<()> {
    if path.is_empty()
        || path.starts_with('/')
        || path.starts_with('\\')
        || path.contains(':')
        || path.split(['/', '\\']).any(|part| part == "..")
    {
        anyhow::bail!("unsafe archive path: {path}");
    }
    for component in Path::new(path).components() {
        match component {
            Component::Normal(_) => {}
            _ => anyhow::bail!("unsafe archive path: {path}"),
        }
    }
    Ok(())
}

fn tar_string(field: &[u8]) -> anyhow::Result<String> {
    let end = field
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(field.len());
    Ok(std::str::from_utf8(&field[..end])?.to_string())
}

fn parse_octal(field: &[u8]) -> anyhow::Result<usize> {
    let text = std::str::from_utf8(field)?
        .trim_matches(char::from(0))
        .trim();
    Ok(usize::from_str_radix(text, 8)?)
}

fn write_octal(field: &mut [u8], value: u64) {
    field.fill(0);
    let text = format!("{value:0width$o}", width = field.len() - 1);
    field[..text.len()].copy_from_slice(text.as_bytes());
}

fn write_checksum(field: &mut [u8], value: u64) {
    field.fill(0);
    let text = format!("{value:06o}\0 ");
    field[..text.len()].copy_from_slice(text.as_bytes());
}

fn padding(size: usize) -> usize {
    (512 - (size % 512)) % 512
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}
