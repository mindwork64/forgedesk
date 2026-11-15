use rusqlite::{Connection, OptionalExtension};

use crate::domain::time::now_rfc3339;
use crate::domain::AppSettings;
use crate::error::{CoreError, CoreResult};

pub const SETTINGS_KEY: &str = "app.settings";

pub fn load(conn: &Connection) -> CoreResult<AppSettings> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT value FROM app_setting WHERE key = ?1",
            [SETTINGS_KEY],
            |row| row.get(0),
        )
        .optional()?;

    match raw {
        None => Ok(AppSettings::default()),
        Some(raw) => {
            let settings: AppSettings = serde_json::from_str(&raw).map_err(|error| {
                CoreError::internal(format!(
                    "Stored application settings are unreadable: {error}"
                ))
            })?;
            Ok(settings.normalized())
        }
    }
}

pub fn save(conn: &Connection, settings: &AppSettings) -> CoreResult<()> {
    let value = serde_json::to_string(settings).map_err(|error| {
        CoreError::internal(format!("Settings could not be serialized: {error}"))
    })?;
    conn.execute(
        "INSERT INTO app_setting (key, value, updated_at) VALUES (?1, ?2, ?3) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        rusqlite::params![SETTINGS_KEY, value, now_rfc3339()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::domain::{LocalePreference, ThemePreference};

    #[test]
    fn defaults_when_absent_and_roundtrips() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(dir.path().join("test.db")).expect("open");

        assert_eq!(db.settings().expect("defaults"), AppSettings::default());

        let settings = AppSettings {
            locale: LocalePreference::Ru,
            theme: ThemePreference::Light,
            sidebar_collapsed: true,
            ..Default::default()
        };
        db.save_settings(&settings).expect("save");
        assert_eq!(db.settings().expect("load"), settings);
    }

    #[test]
    fn corrupt_settings_are_reported_not_reset() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = Database::open(dir.path().join("test.db")).expect("open");
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO app_setting (key, value, updated_at) VALUES (?1, '{not json', ?2)",
                rusqlite::params![SETTINGS_KEY, now_rfc3339()],
            )?;
            Ok(())
        })
        .expect("seed corrupt settings");

        let error = db.settings().expect_err("must not silently reset");
        assert_eq!(error.category(), crate::error::ErrorCategory::Internal);
    }
}
