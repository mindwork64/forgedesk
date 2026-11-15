use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use rusqlite::{Connection, OpenFlags};

use crate::error::{CoreError, CoreResult};

pub mod migrations;
pub mod repositories;

use repositories::settings as settings_repo;
use repositories::workspace as workspace_repo;

/// Serialized SQLite access for a single-user desktop app.
///
/// One writer connection is guarded by a mutex; WAL mode allows future read
/// connections without changing this boundary. The connection can be closed and
/// reopened so backup restore can replace the database file safely.
pub struct Database {
    path: PathBuf,
    inner: Mutex<DbInner>,
}

struct DbInner {
    conn: Option<Connection>,
    schema_version: i64,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> CoreResult<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db = Self {
            path,
            inner: Mutex::new(DbInner {
                conn: None,
                schema_version: 0,
            }),
        };
        db.reopen()?;
        Ok(db)
    }

    pub fn open_read_only(path: impl AsRef<Path>) -> CoreResult<Connection> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        conn.busy_timeout(Duration::from_secs(5))?;
        Ok(conn)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn schema_version(&self) -> CoreResult<i64> {
        let guard = self.lock()?;
        Ok(guard.schema_version)
    }

    /// Run a closure with exclusive access to the connection.
    pub fn with_conn<T>(&self, f: impl FnOnce(&Connection) -> CoreResult<T>) -> CoreResult<T> {
        let guard = self.lock()?;
        let conn = guard
            .conn
            .as_ref()
            .ok_or_else(|| CoreError::internal("The local database is closed."))?;
        f(conn)
    }

    pub fn with_transaction<T>(
        &self,
        f: impl FnOnce(&rusqlite::Transaction<'_>) -> CoreResult<T>,
    ) -> CoreResult<T> {
        let mut guard = self.lock()?;
        let conn = guard
            .conn
            .as_mut()
            .ok_or_else(|| CoreError::internal("The local database is closed."))?;
        let tx = conn.transaction()?;
        let value = f(&tx)?;
        tx.commit()?;
        Ok(value)
    }

    pub fn workspace(&self) -> CoreResult<Option<crate::domain::Workspace>> {
        self.with_conn(workspace_repo::load_first)
    }

    pub fn settings(&self) -> CoreResult<crate::domain::AppSettings> {
        self.with_conn(settings_repo::load)
    }

    pub fn save_settings(&self, settings: &crate::domain::AppSettings) -> CoreResult<()> {
        self.with_conn(|conn| settings_repo::save(conn, settings))
    }

    pub fn checkpoint(&self) -> CoreResult<()> {
        self.with_conn(|conn| {
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
            Ok(())
        })
    }

    pub fn close(&self) -> CoreResult<()> {
        let mut guard = self.lock()?;
        if let Some(conn) = guard.conn.take() {
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
            drop(conn);
        }
        Ok(())
    }

    /// Replace the live database file. The caller must have created a safety
    /// backup first. On failure the previous file is restored and reopened.
    pub fn replace_with(&self, source: impl AsRef<Path>) -> CoreResult<()> {
        let source = source.as_ref();
        if !source.is_file() {
            return Err(CoreError::backup(
                "The selected backup file does not exist.",
            ));
        }

        if let Err(error) = self.close() {
            let _ = self.reopen();
            return Err(error);
        }

        let pre_restore = self.path.with_extension("db.pre-restore");
        let _ = std::fs::remove_file(&pre_restore);

        let preparation = (|| -> CoreResult<()> {
            std::fs::copy(&self.path, &pre_restore)?;
            // A stale WAL/SHM pair from the old database must never be replayed
            // against the restored file.
            remove_sidecar(&self.path, "-wal");
            remove_sidecar(&self.path, "-shm");
            std::fs::copy(source, &self.path)?;
            Ok(())
        })();

        if let Err(error) = preparation {
            let _ = self.reopen();
            return Err(error);
        }

        match self.reopen() {
            Ok(()) => {
                let _ = std::fs::remove_file(&pre_restore);
                Ok(())
            }
            Err(error) => {
                let _ = std::fs::copy(&pre_restore, &self.path);
                remove_sidecar(&self.path, "-wal");
                remove_sidecar(&self.path, "-shm");
                let _ = std::fs::remove_file(&pre_restore);
                let _ = self.reopen();
                Err(error)
            }
        }
    }

    fn reopen(&self) -> CoreResult<()> {
        let mut conn = Connection::open(&self.path)?;
        conn.busy_timeout(Duration::from_secs(5))?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        if !is_memory_database(&self.path) {
            let mode: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
            if !mode.eq_ignore_ascii_case("wal") {
                return Err(CoreError::startup(
                    "The local database could not enable WAL journaling.",
                ));
            }
        }
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "temp_store", "MEMORY")?;
        let schema_version = migrations::run(&mut conn)?;

        let mut guard = self.lock()?;
        guard.conn = Some(conn);
        guard.schema_version = schema_version;
        Ok(())
    }

    fn lock(&self) -> CoreResult<std::sync::MutexGuard<'_, DbInner>> {
        self.inner
            .lock()
            .map_err(|_| CoreError::internal("The local database lock is poisoned."))
    }
}

fn remove_sidecar(path: &Path, suffix: &str) {
    let sidecar = PathBuf::from(format!("{}{suffix}", path.to_string_lossy()));
    let _ = std::fs::remove_file(sidecar);
}

fn is_memory_database(path: &Path) -> bool {
    let raw = path.to_string_lossy();
    raw == ":memory:" || raw.starts_with("file::memory:")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::AppSettings;

    fn temp_db() -> (tempfile::TempDir, Database) {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(dir.path().join("test.db")).expect("open db");
        (dir, db)
    }

    #[test]
    fn creates_schema_and_survives_reopen() {
        let (dir, db) = temp_db();
        assert_eq!(db.schema_version().expect("version"), 1);

        let settings = AppSettings::default();
        db.save_settings(&settings).expect("save settings");
        assert_eq!(db.settings().expect("load settings"), settings);

        let path = dir.path().join("test.db");
        drop(db);
        let reopened = Database::open(&path).expect("reopen");
        assert_eq!(
            reopened.settings().expect("settings after reopen"),
            settings
        );
    }

    #[test]
    fn workspace_roundtrip() {
        let (_dir, db) = temp_db();
        assert!(db.workspace().expect("load workspace").is_none());

        let workspace = db
            .with_transaction(|tx| workspace_repo::create_initial(tx, "My Workspace"))
            .expect("create workspace");
        assert_eq!(workspace.name, "My Workspace");
        let loaded = db.workspace().expect("load").expect("some");
        assert_eq!(loaded.id, workspace.id);
        assert_eq!(loaded.key_prefix, "FD");
    }
}
