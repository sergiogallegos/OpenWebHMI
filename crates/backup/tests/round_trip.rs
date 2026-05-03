use flate2::Compression;
use flate2::write::GzEncoder;
use openwebhmi_alarm_engine::{AlarmJournal, AlarmState, AlarmTransition};
use openwebhmi_backup::{BackupManifest, BackupOptions, ImportMode, RestoreOptions};
use openwebhmi_historian::{Aggregation, HistorianStore};
use openwebhmi_project_store::{ArtifactKind, ProjectStore};
use openwebhmi_protocol::{Quality, TagValue};
use std::io::Write;

#[test]
fn export_import_round_trip_preserves_project_artifacts() {
    let source_dir = tempfile::tempdir().unwrap();
    let target_dir = tempfile::tempdir().unwrap();
    let source = ProjectStore::open(source_dir.path()).unwrap();
    let target = ProjectStore::open(target_dir.path()).unwrap();
    seed_project(&source);

    let archive = openwebhmi_backup::export_project(
        &source,
        "demo",
        BackupOptions {
            historian_sqlite: Some(b"history".to_vec()),
            alarm_journal_sqlite: Some(b"alarms".to_vec()),
            ..BackupOptions::default()
        },
    )
    .unwrap();
    let manifest =
        openwebhmi_backup::import_project(&target, &archive, RestoreOptions::default()).unwrap();

    assert_eq!(manifest.schema_version, 1);
    assert!(manifest.includes_historian);
    assert!(manifest.includes_alarm_journal);
    assert_eq!(source.load("demo").unwrap(), target.load("demo").unwrap());
}

#[test]
fn manifest_schema_version_round_trips() {
    let source_dir = tempfile::tempdir().unwrap();
    let source = ProjectStore::open(source_dir.path()).unwrap();
    seed_project(&source);

    let archive =
        openwebhmi_backup::export_project(&source, "demo", BackupOptions::default()).unwrap();
    let entries = read_archive_entries(&archive);
    let manifest_bytes = entries
        .iter()
        .find(|(path, _)| path == "manifest.json")
        .unwrap()
        .1
        .clone();
    let manifest: BackupManifest = serde_json::from_slice(&manifest_bytes).unwrap();

    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.project_id, "demo");
}

#[test]
fn export_uses_online_backup_for_live_sqlite_stores() {
    let source_dir = tempfile::tempdir().unwrap();
    let source = ProjectStore::open(source_dir.path()).unwrap();
    seed_project(&source);
    let historian = HistorianStore::memory().unwrap();
    historian
        .write_sample("rockwell-1/value", 10, &TagValue::Real(12.5), Quality::Good)
        .unwrap();
    let alarm_journal = AlarmJournal::memory().unwrap();
    alarm_journal
        .write_transition(&AlarmTransition {
            alarm_id: "pressure-high".into(),
            ts_ms: 10,
            from_state: AlarmState::Clear,
            to_state: AlarmState::Active,
            who: Some("admin".into()),
            note: None,
        })
        .unwrap();

    let archive = openwebhmi_backup::export_project(
        &source,
        "demo",
        BackupOptions {
            historian_store: Some(historian),
            alarm_journal: Some(alarm_journal),
            ..BackupOptions::default()
        },
    )
    .unwrap();
    let entries = read_archive_entries(&archive);

    assert!(
        entries
            .iter()
            .any(|(path, bytes)| path == "historian.sqlite" && !bytes.is_empty())
    );
    assert!(
        entries
            .iter()
            .any(|(path, bytes)| path == "alarm-journal.sqlite" && !bytes.is_empty())
    );
}

#[test]
fn historian_rows_round_trip_through_export_import() {
    let source_dir = tempfile::tempdir().unwrap();
    let target_dir = tempfile::tempdir().unwrap();
    let source = ProjectStore::open(source_dir.path()).unwrap();
    let target = ProjectStore::open(target_dir.path()).unwrap();
    seed_project(&source);
    let source_historian = HistorianStore::memory().unwrap();
    let target_historian = HistorianStore::memory().unwrap();
    source_historian
        .write_sample("rockwell-1/value", 10, &TagValue::Real(12.5), Quality::Good)
        .unwrap();
    source_historian
        .write_sample(
            "rockwell-1/value",
            20,
            &TagValue::Real(13.5),
            Quality::Stale,
        )
        .unwrap();

    let archive = openwebhmi_backup::export_project(
        &source,
        "demo",
        BackupOptions {
            historian_store: Some(source_historian.clone()),
            ..BackupOptions::default()
        },
    )
    .unwrap();
    openwebhmi_backup::import_project(
        &target,
        &archive,
        RestoreOptions {
            historian_store: Some(target_historian.clone()),
            ..RestoreOptions::default()
        },
    )
    .unwrap();

    assert_eq!(
        target_historian
            .read("rockwell-1/value", 0, 30, Aggregation::Raw, 10)
            .unwrap(),
        source_historian
            .read("rockwell-1/value", 0, 30, Aggregation::Raw, 10)
            .unwrap()
    );
}

#[test]
fn alarm_transitions_round_trip_through_export_import() {
    let source_dir = tempfile::tempdir().unwrap();
    let target_dir = tempfile::tempdir().unwrap();
    let source = ProjectStore::open(source_dir.path()).unwrap();
    let target = ProjectStore::open(target_dir.path()).unwrap();
    seed_project(&source);
    let source_journal = AlarmJournal::memory().unwrap();
    let target_journal = AlarmJournal::memory().unwrap();
    source_journal
        .write_transition(&alarm_transition("pressure-high", 10))
        .unwrap();
    source_journal
        .write_transition(&AlarmTransition {
            ts_ms: 20,
            from_state: AlarmState::Active,
            to_state: AlarmState::Acked,
            ..alarm_transition("pressure-high", 20)
        })
        .unwrap();

    let archive = openwebhmi_backup::export_project(
        &source,
        "demo",
        BackupOptions {
            alarm_journal: Some(source_journal.clone()),
            ..BackupOptions::default()
        },
    )
    .unwrap();
    openwebhmi_backup::import_project(
        &target,
        &archive,
        RestoreOptions {
            alarm_journal: Some(target_journal.clone()),
            ..RestoreOptions::default()
        },
    )
    .unwrap();

    assert_eq!(
        target_journal.read_alarm("pressure-high").unwrap(),
        source_journal.read_alarm("pressure-high").unwrap()
    );
}

#[test]
fn replace_mode_wipes_target_historian_and_alarm_journal() {
    let source_dir = tempfile::tempdir().unwrap();
    let target_dir = tempfile::tempdir().unwrap();
    let source = ProjectStore::open(source_dir.path()).unwrap();
    let target = ProjectStore::open(target_dir.path()).unwrap();
    seed_project(&source);
    let source_historian = HistorianStore::memory().unwrap();
    let target_historian = HistorianStore::memory().unwrap();
    let source_journal = AlarmJournal::memory().unwrap();
    let target_journal = AlarmJournal::memory().unwrap();
    source_historian
        .write_sample("rockwell-1/value", 10, &TagValue::Real(12.5), Quality::Good)
        .unwrap();
    target_historian
        .write_sample("target-only", 5, &TagValue::Real(1.0), Quality::Good)
        .unwrap();
    source_journal
        .write_transition(&alarm_transition("pressure-high", 10))
        .unwrap();
    target_journal
        .write_transition(&alarm_transition("target-only", 5))
        .unwrap();

    let archive = openwebhmi_backup::export_project(
        &source,
        "demo",
        BackupOptions {
            historian_store: Some(source_historian),
            alarm_journal: Some(source_journal),
            ..BackupOptions::default()
        },
    )
    .unwrap();
    openwebhmi_backup::import_project(
        &target,
        &archive,
        RestoreOptions {
            mode: ImportMode::Replace,
            historian_store: Some(target_historian.clone()),
            alarm_journal: Some(target_journal.clone()),
            ..RestoreOptions::default()
        },
    )
    .unwrap();

    assert!(
        target_historian
            .read("target-only", 0, 20, Aggregation::Raw, 10)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        target_historian
            .read("rockwell-1/value", 0, 20, Aggregation::Raw, 10)
            .unwrap()
            .len(),
        1
    );
    assert!(target_journal.read_alarm("target-only").unwrap().is_empty());
    assert_eq!(target_journal.read_alarm("pressure-high").unwrap().len(), 1);
}

#[test]
fn malicious_archive_path_is_rejected() {
    let target_dir = tempfile::tempdir().unwrap();
    let target = ProjectStore::open(target_dir.path()).unwrap();
    let archive = make_malicious_archive();

    let err = openwebhmi_backup::import_project(
        &target,
        &archive,
        RestoreOptions {
            mode: ImportMode::Replace,
            ..RestoreOptions::default()
        },
    )
    .unwrap_err();

    assert!(err.to_string().contains("unsafe archive path"));
    assert!(!target_dir.path().join("evil").exists());
}

#[test]
fn replace_mode_removes_target_orphan_artifacts() {
    let source_dir = tempfile::tempdir().unwrap();
    let target_dir = tempfile::tempdir().unwrap();
    let source = ProjectStore::open(source_dir.path()).unwrap();
    let target = ProjectStore::open(target_dir.path()).unwrap();
    seed_project(&source);
    seed_project(&target);
    target
        .save_artifact(
            "demo",
            ArtifactKind::View {
                id: "orphan".into(),
            },
            serde_json::json!({
                "id": "orphan",
                "title": "Orphan",
                "schema_version": 1,
                "root": {"id":"root","kind":"Container","props":{},"bindings":[],"children":[]}
            }),
        )
        .unwrap();

    let archive =
        openwebhmi_backup::export_project(&source, "demo", BackupOptions::default()).unwrap();
    openwebhmi_backup::import_project(
        &target,
        &archive,
        RestoreOptions {
            mode: ImportMode::Replace,
            ..RestoreOptions::default()
        },
    )
    .unwrap();

    assert!(
        target
            .read_artifact(
                "demo",
                ArtifactKind::View {
                    id: "orphan".into()
                }
            )
            .unwrap()
            .is_none()
    );
}

fn seed_project(store: &ProjectStore) {
    store
        .save_artifact(
            "demo",
            ArtifactKind::ProjectMeta,
            serde_json::json!({
                "schema_version": 1,
                "name": "Demo",
                "drivers": [{"id": "rockwell-1", "type": "rockwell", "config": {"host":"127.0.0.1","slot":0}}],
                "tags": []
            }),
        )
        .unwrap();
    store
        .save_artifact(
            "demo",
            ArtifactKind::Tags,
            serde_json::json!([{"path":"rockwell-1/value","driver":"rockwell-1","address":"value","data_type":"real"}]),
        )
        .unwrap();
    store
        .save_artifact("demo", ArtifactKind::Alarms, serde_json::json!([]))
        .unwrap();
    store
        .save_artifact(
            "demo",
            ArtifactKind::View { id: "home".into() },
            serde_json::json!({
                "id": "home",
                "title": "Home",
                "schema_version": 1,
                "root": {"id":"root","kind":"Container","props":{},"bindings":[],"children":[]}
            }),
        )
        .unwrap();
}

fn alarm_transition(alarm_id: &str, ts_ms: u64) -> AlarmTransition {
    AlarmTransition {
        alarm_id: alarm_id.into(),
        ts_ms,
        from_state: AlarmState::Clear,
        to_state: AlarmState::Active,
        who: Some("admin".into()),
        note: None,
    }
}

fn make_malicious_archive() -> Vec<u8> {
    let mut tar = Vec::new();
    append_test_tar_file(&mut tar, "../evil", b"owned");
    tar.extend([0u8; 1024]);
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&tar).unwrap();
    encoder.finish().unwrap()
}

fn read_archive_entries(archive: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut decoder = flate2::read::GzDecoder::new(archive);
    let mut tar = Vec::new();
    std::io::Read::read_to_end(&mut decoder, &mut tar).unwrap();
    let mut entries = Vec::new();
    let mut offset = 0usize;
    while offset + 512 <= tar.len() {
        let header = &tar[offset..offset + 512];
        offset += 512;
        if header.iter().all(|byte| *byte == 0) {
            break;
        }
        let path_end = header[0..100]
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(100);
        let path = std::str::from_utf8(&header[0..path_end])
            .unwrap()
            .to_string();
        let size_text = std::str::from_utf8(&header[124..136])
            .unwrap()
            .trim_matches(char::from(0))
            .trim();
        let size = usize::from_str_radix(size_text, 8).unwrap();
        entries.push((path, tar[offset..offset + size].to_vec()));
        offset += size + ((512 - (size % 512)) % 512);
    }
    entries
}

fn append_test_tar_file(out: &mut Vec<u8>, path: &str, bytes: &[u8]) {
    let mut header = [0u8; 512];
    header[0..path.len()].copy_from_slice(path.as_bytes());
    write_octal(&mut header[100..108], 0o644);
    write_octal(&mut header[108..116], 0);
    write_octal(&mut header[116..124], 0);
    write_octal(&mut header[124..136], bytes.len() as u64);
    write_octal(&mut header[136..148], 0);
    header[148..156].fill(b' ');
    header[156] = b'0';
    header[257..263].copy_from_slice(b"ustar\0");
    header[263..265].copy_from_slice(b"00");
    let checksum = header.iter().map(|byte| *byte as u64).sum::<u64>();
    let text = format!("{checksum:06o}\0 ");
    header[148..148 + text.len()].copy_from_slice(text.as_bytes());
    out.extend(header);
    out.extend(bytes);
    out.extend(std::iter::repeat_n(0, (512 - (bytes.len() % 512)) % 512));
}

fn write_octal(field: &mut [u8], value: u64) {
    field.fill(0);
    let text = format!("{value:0width$o}", width = field.len() - 1);
    field[..text.len()].copy_from_slice(text.as_bytes());
}
