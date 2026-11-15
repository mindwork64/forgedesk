use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::db::migrations::CURRENT_SCHEMA_VERSION;
use crate::db::Database;
use crate::domain::time::{now_rfc3339, timestamp_for_id};
use crate::error::{CoreError, CoreResult};

pub const BACKUP_FORMAT_VERSION: u32 = 1;
pub const BACKUP_PREFIX: &str = "forgedesk-";
pub const AUTO_BACKUP_INTERVAL_HOURS: i64 = 24;

/// Backup metadata exposed to the UI. File paths are absolute for display only;
/// restore operations accept an opaque backup id, never a raw path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub id: String,
    pub file_path: String,
    pub created_at: String,
    pub schema_version: i64,
    pub size_bytes: u64,
    pub app_version: String,
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupManifest {
    format_version: u32,
    app_version: String,
    schema_version: i64,
    created_at: String,
    database_file: String,
    size_bytes: u64,
    sha256: String,
    attachments_included: bool,
}

pub fn create(db: &Database, backups_dir: &Path, app_version: &str) -> CoreResult<BackupInfo> {
    std::fs::create_dir_all(backups_dir)?;

    let created_at = now_rfc3339();
    let id = format!(
        "{BACKUP_PREFIX}{}-{}",
        timestamp_for_id(),
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    );
    let temp_db = backups_dir.join(format!(".tmp-{}.db", uuid::Uuid::now_v7()));
    let final_db = backups_dir.join(format!("{id}.db"));
    let manifest_path = backups_dir.join(format!("{id}.json"));

    if temp_db.exists() {
        std::fs::remove_file(&temp_db)?;
    }
    if final_db.exists() || manifest_path.exists() {
        return Err(CoreError::backup(
            "A backup with the generated identifier already exists.",
        ));
    }

    let vacuum_result = db.with_conn(|conn| {
        conn.execute("VACUUM INTO ?1", [temp_db.to_string_lossy().as_ref()])?;
        Ok(())
    });

    if let Err(error) = vacuum_result {
        let _ = std::fs::remove_file(&temp_db);
        return Err(error);
    }

    let result = (|| -> CoreResult<BackupInfo> {
        let schema_version = db.schema_version()?;
        let size_bytes = std::fs::metadata(&temp_db)?.len();
        let sha256 = sha256_file(&temp_db)?;
        std::fs::rename(&temp_db, &final_db)?;

        let database_file = final_db
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| CoreError::backup("Backup file name is invalid."))?
            .to_owned();

        let manifest = BackupManifest {
            format_version: BACKUP_FORMAT_VERSION,
            app_version: app_version.to_owned(),
            schema_version,
            created_at: created_at.clone(),
            database_file,
            size_bytes,
            sha256,
            attachments_included: false,
        };
        write_manifest(&manifest_path, &manifest)?;

        Ok(BackupInfo {
            id,
            file_path: final_db.to_string_lossy().into_owned(),
            created_at,
            schema_version,
            size_bytes,
            app_version: app_version.to_owned(),
            valid: true,
        })
    })();

    if result.is_err() {
        let _ = std::fs::remove_file(&final_db);
        let _ = std::fs::remove_file(&manifest_path);
    }
    result
}

pub fn list(backups_dir: &Path) -> CoreResult<Vec<BackupInfo>> {
    if !backups_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    for entry in std::fs::read_dir(backups_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if !is_valid_backup_id(id) {
            continue;
        }
        let Ok(manifest) = read_manifest(&path) else {
            continue;
        };
        if manifest.database_file != format!("{id}.db") {
            continue;
        }
        let db_path = backups_dir.join(&manifest.database_file);
        backups.push(BackupInfo {
            id: id.to_owned(),
            file_path: db_path.to_string_lossy().into_owned(),
            created_at: manifest.created_at,
            schema_version: manifest.schema_version,
            size_bytes: manifest.size_bytes,
            app_version: manifest.app_version,
            valid: db_path.is_file(),
        });
    }

    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(backups)
}

pub fn find(backups_dir: &Path, id: &str) -> CoreResult<BackupInfo> {
    if !is_valid_backup_id(id) {
        return Err(CoreError::validation("id", "Invalid backup identifier."));
    }
    list(backups_dir)?
        .into_iter()
        .find(|backup| backup.id == id)
        .ok_or_else(|| CoreError::not_found("Backup"))
}

/// Full validation: manifest, checksum, SQLite integrity and schema version.
pub fn validate(backups_dir: &Path, id: &str) -> CoreResult<BackupInfo> {
    let info = find(backups_dir, id)?;
    let manifest_path = backups_dir.join(format!("{id}.json"));
    let manifest = read_manifest(&manifest_path)?;
    if manifest.database_file != format!("{id}.db") {
        return Err(CoreError::backup(
            "The backup manifest references an unexpected database file.",
        ));
    }
    let db_path = backups_dir.join(&manifest.database_file);

    if manifest.format_version != BACKUP_FORMAT_VERSION {
        return Err(CoreError::backup(
            "This backup was created by an unsupported backup format.",
        ));
    }
    if manifest.schema_version > CURRENT_SCHEMA_VERSION {
        return Err(CoreError::backup(
            "This backup belongs to a newer ForgeDesk version and cannot be restored.",
        ));
    }
    let metadata = std::fs::metadata(&db_path)
        .map_err(|_| CoreError::backup("The backup database file is missing."))?;
    if metadata.len() != manifest.size_bytes {
        return Err(CoreError::backup(
            "The backup file size does not match its manifest.",
        ));
    }
    if sha256_file(&db_path)? != manifest.sha256 {
        return Err(CoreError::backup(
            "The backup checksum does not match. The file may be corrupted.",
        ));
    }

    let conn = Database::open_read_only(&db_path)?;
    let integrity: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if !integrity.eq_ignore_ascii_case("ok") {
        return Err(CoreError::backup(
            "The backup database failed its integrity check.",
        ));
    }
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version != manifest.schema_version {
        return Err(CoreError::backup(
            "The backup schema version does not match its manifest.",
        ));
    }
    let has_workspace: i64 = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'workspace'",
        [],
        |row| row.get(0),
    )?;
    if has_workspace != 1 {
        return Err(CoreError::backup(
            "The backup does not contain a ForgeDesk workspace database.",
        ));
    }

    Ok(BackupInfo {
        valid: true,
        ..info
    })
}

pub fn prune(backups_dir: &Path, keep: u32) -> CoreResult<u32> {
    let keep = keep as usize;
    let backups = list(backups_dir)?;
    let mut removed = 0_u32;
    for backup in backups.into_iter().skip(keep) {
        if !is_valid_backup_id(&backup.id) {
            continue;
        }
        let db_path = backups_dir.join(format!("{}.db", backup.id));
        let manifest_path = backups_dir.join(format!("{}.json", backup.id));
        let _ = std::fs::remove_file(db_path);
        let _ = std::fs::remove_file(manifest_path);
        removed += 1;
    }
    Ok(removed)
}

pub fn latest_backup_age_hours(backups_dir: &Path) -> CoreResult<Option<i64>> {
    let Some(latest) = list(backups_dir)?.into_iter().next() else {
        return Ok(None);
    };
    let parsed = DateTime::parse_from_rfc3339(&latest.created_at)
        .map_err(|_| CoreError::backup("A backup has an invalid creation timestamp."))?;
    Ok(Some((Utc::now() - parsed.with_timezone(&Utc)).num_hours()))
}

fn write_manifest(path: &Path, manifest: &BackupManifest) -> CoreResult<()> {
    let bytes = serde_json::to_vec_pretty(manifest).map_err(|error| {
        CoreError::backup(format!("Backup manifest could not be encoded: {error}"))
    })?;
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, bytes)?;
    std::fs::rename(&temp, path)?;
    Ok(())
}

fn read_manifest(path: &Path) -> CoreResult<BackupManifest> {
    let raw = std::fs::read(path)
        .map_err(|_| CoreError::backup("The backup manifest is missing or unreadable."))?;
    serde_json::from_slice(&raw)
        .map_err(|_| CoreError::backup("The backup manifest is not valid JSON."))
}

fn is_valid_backup_id(id: &str) -> bool {
    id.starts_with(BACKUP_PREFIX)
        && id.len() <= 96
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn sha256_file(path: &Path) -> CoreResult<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    const CHARS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(CHARS[(byte >> 4) as usize] as char);
        out.push(CHARS[(byte & 0x0f) as usize] as char);
    }
    out
}

pub fn backup_path(backups_dir: &Path, id: &str) -> CoreResult<PathBuf> {
    if !is_valid_backup_id(id) {
        return Err(CoreError::validation("id", "Invalid backup identifier."));
    }
    Ok(backups_dir.join(format!("{id}.db")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::domain::AppSettings;

    fn fixture() -> (tempfile::TempDir, Database, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(dir.path().join("forgedesk.db")).expect("open db");
        let backups = dir.path().join("backups");
        (dir, db, backups)
    }

    #[test]
    fn creates_and_validates_backup() {
        let (_dir, db, backups) = fixture();
        db.save_settings(&AppSettings::default()).expect("save");

        let info = create(&db, &backups, "0.1.0").expect("backup");
        assert!(info.valid);
        assert!(Path::new(&info.file_path).is_file());
        assert_eq!(info.schema_version, CURRENT_SCHEMA_VERSION);

        let validated = validate(&backups, &info.id).expect("valid backup");
        assert_eq!(validated.id, info.id);

        let listed = list(&backups).expect("list");
        assert_eq!(listed.len(), 1);
    }

    #[test]
    fn detects_corrupted_backup() {
        let (_dir, db, backups) = fixture();
        let info = create(&db, &backups, "0.1.0").expect("backup");

        let path = backups.join(format!("{}.db", info.id));
        let mut bytes = std::fs::read(&path).expect("read");
        let index = bytes.len() / 2;
        bytes[index] ^= 0xff;
        std::fs::write(&path, bytes).expect("write");

        let error = validate(&backups, &info.id).expect_err("corrupt backup must fail");
        assert_eq!(error.category(), crate::error::ErrorCategory::Backup);
    }

    #[test]
    fn rejects_path_traversal_ids() {
        let (_dir, _db, backups) = fixture();
        assert!(validate(&backups, "../../etc/passwd").is_err());
        assert!(backup_path(&backups, "../evil").is_err());
    }

    #[test]
    fn prunes_old_backups() {
        let (_dir, db, backups) = fixture();
        for _ in 0..4 {
            create(&db, &backups, "0.1.0").expect("backup");
        }
        let removed = prune(&backups, 2).expect("prune");
        assert_eq!(removed, 2);
        assert_eq!(list(&backups).expect("list").len(), 2);
    }
}
