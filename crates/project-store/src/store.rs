//! Project store filesystem and SQLite index implementation.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, bail};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::types::{
    AlarmConfig, Project, ProjectMetaFile, ProjectSummary, ScriptConfig, ScriptTriggerConfig,
    TagConfig, Theme, View,
};
use crate::version::{ChangeAction, ProjectChange};

const CHANGE_CAPACITY: usize = 1024;

/// Project artifact selector.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArtifactKind {
    /// Project metadata.
    ProjectMeta,
    /// View by id.
    View {
        /// View id.
        id: String,
    },
    /// Tag definition set.
    Tags,
    /// Alarm definition set.
    Alarms,
    /// Project-level theme singleton.
    Theme,
    /// Script by id.
    Script {
        /// Script id.
        id: String,
    },
    /// Raw Python source for a registered script.
    ScriptSource {
        /// Script id.
        id: String,
    },
}

/// Persistent project store.
#[derive(Clone)]
pub struct ProjectStore {
    inner: Arc<StoreInner>,
}

struct StoreInner {
    root: PathBuf,
    conn: Mutex<Connection>,
    changes: broadcast::Sender<ProjectChange>,
}

impl ProjectStore {
    /// Open or create a project store rooted at `root`.
    pub fn open(root: impl AsRef<Path>) -> anyhow::Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root).with_context(|| format!("failed to create {:?}", root))?;
        let db_path = root.join("_index.sqlite");
        let conn = Connection::open(&db_path)
            .with_context(|| format!("failed to open sqlite index {:?}", db_path))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                version INTEGER NOT NULL,
                last_modified INTEGER NOT NULL
            );",
        )?;
        let (changes, _) = broadcast::channel(CHANGE_CAPACITY);
        Ok(Self {
            inner: Arc::new(StoreInner {
                root,
                conn: Mutex::new(conn),
                changes,
            }),
        })
    }

    /// List projects known to the SQLite metadata index.
    pub fn list(&self) -> anyhow::Result<Vec<ProjectSummary>> {
        let conn = self.lock_conn()?;
        let mut stmt =
            conn.prepare("SELECT id, version, last_modified FROM projects ORDER BY id ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok(ProjectSummary {
                id: row.get(0)?,
                version: row.get::<_, i64>(1)? as u64,
                last_modified: row.get(2)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Load a full project.
    pub fn load(&self, project_id: &str) -> anyhow::Result<Project> {
        let project_dir = self.project_dir(project_id);
        let meta_path = project_dir.join("project.toml");
        let meta_text = fs::read_to_string(&meta_path)
            .with_context(|| format!("failed to read {:?}", meta_path))?;
        let meta: ProjectMetaFile = toml::from_str(&meta_text)
            .with_context(|| format!("failed to parse {:?}", meta_path))?;

        let drivers = meta
            .drivers
            .into_iter()
            .map(|driver| driver.into_driver_config())
            .collect::<anyhow::Result<Vec<_>>>()?;

        let mut tags = read_tags_file(project_dir.join("tags/tags.json"))?;
        tags.extend(meta.tags);
        let alarms = read_alarms_file(project_dir.join("alarms/alarms.json"))?;
        let theme = read_theme_file(project_dir.join("theme/theme.json"))?;
        let scripts = read_scripts_file(project_dir.join("scripts/scripts.json"), &project_dir)?;

        let mut views = Vec::new();
        let views_dir = project_dir.join("views");
        if views_dir.exists() {
            for entry in fs::read_dir(&views_dir)? {
                let path = entry?.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                    let view = serde_json::from_str::<View>(&fs::read_to_string(&path)?)?;
                    views.push(view);
                }
            }
            views.sort_by(|a, b| a.id.cmp(&b.id));
        }

        let version = self.version(project_id)?.unwrap_or(0);
        let project = Project {
            id: project_id.to_string(),
            schema_version: meta.schema_version,
            name: meta.name,
            version,
            drivers,
            tags,
            alarms,
            theme,
            scripts,
            views,
        };
        validate_project(&project)?;
        Ok(project)
    }

    /// Save a single artifact and return the new project version.
    pub fn save_artifact(
        &self,
        project_id: &str,
        kind: ArtifactKind,
        body: serde_json::Value,
    ) -> anyhow::Result<u64> {
        ensure_layout(&self.project_dir(project_id))?;
        let path = self.artifact_path(project_id, &kind)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let existed = path.exists();
        let bytes = match &kind {
            ArtifactKind::ProjectMeta => serde_json_to_toml(&body)?.into_bytes(),
            ArtifactKind::ScriptSource { .. } => body
                .get("source")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("ScriptSource body requires string field 'source'"))?
                .as_bytes()
                .to_vec(),
            _ => serde_json::to_vec_pretty(&body)?,
        };
        atomic_write(&path, &bytes)?;
        let version = self.bump_version(project_id)?;
        let action = if existed {
            ChangeAction::Updated
        } else {
            ChangeAction::Created
        };
        self.broadcast(ProjectChange {
            project_id: project_id.to_string(),
            version,
            artifact: kind,
            action,
        });
        Ok(version)
    }

    /// Read a single artifact.
    pub fn read_artifact(
        &self,
        project_id: &str,
        kind: ArtifactKind,
    ) -> anyhow::Result<Option<serde_json::Value>> {
        let path = self.artifact_path(project_id, &kind)?;
        if !path.exists() {
            return Ok(None);
        }
        let text = fs::read_to_string(&path)?;
        match kind {
            ArtifactKind::ProjectMeta => {
                let value = text.parse::<toml::Value>()?;
                Ok(Some(serde_json::to_value(value)?))
            }
            ArtifactKind::ScriptSource { .. } => Ok(Some(serde_json::json!({ "source": text }))),
            _ => Ok(Some(serde_json::from_str(&text)?)),
        }
    }

    /// Delete a single artifact.
    pub fn delete_artifact(&self, project_id: &str, kind: ArtifactKind) -> anyhow::Result<()> {
        let path = self.artifact_path(project_id, &kind)?;
        if path.exists() {
            fs::remove_file(&path)?;
            let version = self.bump_version(project_id)?;
            self.broadcast(ProjectChange {
                project_id: project_id.to_string(),
                version,
                artifact: kind,
                action: ChangeAction::Deleted,
            });
        }
        Ok(())
    }

    /// Delete all on-disk artifacts and index metadata for a project.
    pub fn delete_project(&self, project_id: &str) -> anyhow::Result<()> {
        let project_dir = self.project_dir(project_id);
        if project_dir.exists() {
            fs::remove_dir_all(project_dir)?;
        }
        self.lock_conn()?
            .execute("DELETE FROM projects WHERE id = ?1", params![project_id])?;
        Ok(())
    }

    /// Subscribe to change events for a project, or all projects when `None`.
    pub fn subscribe_changes(
        &self,
        _project_id: Option<&str>,
    ) -> broadcast::Receiver<ProjectChange> {
        self.inner.changes.subscribe()
    }

    fn artifact_path(&self, project_id: &str, kind: &ArtifactKind) -> anyhow::Result<PathBuf> {
        let dir = self.project_dir(project_id);
        Ok(match kind {
            ArtifactKind::ProjectMeta => dir.join("project.toml"),
            ArtifactKind::View { id } => dir.join("views").join(format!("{id}.json")),
            ArtifactKind::Tags => dir.join("tags/tags.json"),
            ArtifactKind::Alarms => dir.join("alarms/alarms.json"),
            ArtifactKind::Theme => dir.join("theme/theme.json"),
            ArtifactKind::Script { id } => dir.join("scripts").join(format!("{id}.json")),
            ArtifactKind::ScriptSource { id } => self.script_source_path(project_id, id)?,
        })
    }

    fn script_source_path(&self, project_id: &str, script_id: &str) -> anyhow::Result<PathBuf> {
        let project = self.load(project_id)?;
        let script = project
            .scripts
            .iter()
            .find(|script| script.id == script_id)
            .ok_or_else(|| anyhow::anyhow!("script '{script_id}' is not registered"))?;
        let path = Path::new(&script.path);
        Ok(if path.is_absolute() {
            path.to_path_buf()
        } else if path.starts_with("scripts") {
            self.project_dir(project_id).join(path)
        } else {
            self.project_dir(project_id).join("scripts").join(path)
        })
    }

    fn project_dir(&self, project_id: &str) -> PathBuf {
        self.inner.root.join(project_id)
    }

    fn version(&self, project_id: &str) -> anyhow::Result<Option<u64>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare("SELECT version FROM projects WHERE id = ?1")?;
        let mut rows = stmt.query(params![project_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get::<_, i64>(0)? as u64))
        } else {
            Ok(None)
        }
    }

    fn bump_version(&self, project_id: &str) -> anyhow::Result<u64> {
        let now = now_epoch_seconds();
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO projects (id, version, last_modified)
             VALUES (?1, 1, ?2)
             ON CONFLICT(id) DO UPDATE SET
                version = version + 1,
                last_modified = excluded.last_modified",
            params![project_id, now],
        )?;
        let version: i64 = conn.query_row(
            "SELECT version FROM projects WHERE id = ?1",
            params![project_id],
            |row| row.get(0),
        )?;
        Ok(version as u64)
    }

    fn lock_conn(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Connection>> {
        self.inner
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("project-store sqlite lock poisoned"))
    }

    fn broadcast(&self, change: ProjectChange) {
        let _ = self.inner.changes.send(change);
    }
}

/// Validate a project according to schema v1 rules.
pub fn validate_project(project: &Project) -> anyhow::Result<()> {
    if project.schema_version != 1 {
        bail!(
            "unsupported project schema_version {}; expected 1",
            project.schema_version
        );
    }
    let mut driver_ids = HashSet::new();
    for driver in &project.drivers {
        if driver.id.trim().is_empty() {
            bail!("driver id cannot be empty");
        }
        if !driver_ids.insert(driver.id.clone()) {
            bail!("duplicate driver id '{}'", driver.id);
        }
    }
    for tag in &project.tags {
        if !driver_ids.contains(&tag.driver) {
            bail!(
                "tag '{}' references unknown driver '{}'",
                tag.path,
                tag.driver
            );
        }
        let prefix = format!("{}/", tag.driver);
        if !tag.path.starts_with(&prefix) {
            bail!("tag path '{}' must start with '{}'", tag.path, prefix);
        }
    }
    let tag_paths = project
        .tags
        .iter()
        .map(|tag| tag.path.as_str())
        .collect::<HashSet<_>>();
    let mut alarm_ids = HashSet::new();
    for alarm in &project.alarms {
        if alarm.id.trim().is_empty() {
            bail!("alarm id cannot be empty");
        }
        if !alarm_ids.insert(alarm.id.clone()) {
            bail!("duplicate alarm id '{}'", alarm.id);
        }
        if !tag_paths.contains(alarm.tag_path.as_str()) {
            bail!(
                "alarm '{}' references unknown tag '{}'",
                alarm.id,
                alarm.tag_path
            );
        }
        if !(1..=5).contains(&alarm.priority) {
            bail!("alarm '{}' priority must be between 1 and 5", alarm.id);
        }
    }
    let mut script_ids = HashSet::new();
    for script in &project.scripts {
        if script.id.trim().is_empty() {
            bail!("script id cannot be empty");
        }
        if !script_ids.insert(script.id.clone()) {
            bail!("duplicate script id '{}'", script.id);
        }
        if script.path.trim().is_empty() {
            bail!("script '{}' path cannot be empty", script.id);
        }
        for trigger in &script.triggers {
            if let ScriptTriggerConfig::OnTagChange { path } = trigger {
                if !tag_paths.contains(path.as_str()) {
                    bail!("script '{}' references unknown tag '{}'", script.id, path);
                }
            }
        }
    }
    Ok(())
}

fn ensure_layout(project_dir: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(project_dir)?;
    for child in ["views", "tags", "alarms", "scripts", "assets"] {
        fs::create_dir_all(project_dir.join(child))?;
    }
    Ok(())
}

fn read_tags_file(path: PathBuf) -> anyhow::Result<Vec<TagConfig>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let value = serde_json::from_str::<serde_json::Value>(&fs::read_to_string(path)?)?;
    if value.is_array() {
        return Ok(serde_json::from_value(value)?);
    }
    Ok(serde_json::from_value(
        value
            .get("tags")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([])),
    )?)
}

fn read_alarms_file(path: PathBuf) -> anyhow::Result<Vec<AlarmConfig>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let value = serde_json::from_str::<serde_json::Value>(&fs::read_to_string(path)?)?;
    if value.is_array() {
        return Ok(serde_json::from_value(value)?);
    }
    Ok(serde_json::from_value(
        value
            .get("alarms")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([])),
    )?)
}

fn read_theme_file(path: PathBuf) -> anyhow::Result<Option<Theme>> {
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&fs::read_to_string(path)?)?))
}

fn read_scripts_file(path: PathBuf, project_dir: &Path) -> anyhow::Result<Vec<ScriptConfig>> {
    let mut scripts = if path.exists() {
        let value = serde_json::from_str::<serde_json::Value>(&fs::read_to_string(path)?)?;
        let scripts_value = if value.is_array() {
            value
        } else {
            value
                .get("scripts")
                .cloned()
                .unwrap_or_else(|| serde_json::json!([]))
        };
        serde_json::from_value::<Vec<ScriptConfig>>(scripts_value)?
    } else {
        Vec::new()
    };
    let scripts_dir = project_dir.join("scripts");
    if scripts_dir.exists() {
        for entry in fs::read_dir(&scripts_dir)? {
            let path = entry?.path();
            if path.file_name().and_then(|name| name.to_str()) == Some("scripts.json") {
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                let script = serde_json::from_str::<ScriptConfig>(&fs::read_to_string(&path)?)?;
                if let Some(existing) = scripts.iter_mut().find(|item| item.id == script.id) {
                    *existing = script;
                } else {
                    scripts.push(script);
                }
            }
        }
    }
    scripts.sort_by(|a, b| a.id.cmp(&b.id));
    for script in &mut scripts {
        let script_path = Path::new(&script.path);
        if script_path.is_relative() {
            script.path = if script_path.starts_with("scripts") {
                project_dir.join(script_path)
            } else {
                project_dir.join("scripts").join(script_path)
            }
            .to_string_lossy()
            .into_owned();
        }
    }
    Ok(scripts)
}

fn serde_json_to_toml(body: &serde_json::Value) -> anyhow::Result<String> {
    let value = toml::Value::try_from(body.clone())?;
    Ok(toml::to_string_pretty(&value)?)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    let tmp = path.with_extension(format!(
        "{}tmp",
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!("{ext}."))
            .unwrap_or_default()
    ));
    fs::write(&tmp, bytes)?;
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

fn now_epoch_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}
