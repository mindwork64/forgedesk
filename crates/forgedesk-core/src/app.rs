use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::backup::{self, BackupInfo};
use crate::db::Database;
use crate::domain::{AppSettings, SettingsPatch, WindowState, Workspace};
use crate::error::CoreResult;

pub const DB_FILE_NAME: &str = "forgedesk.db";
pub const BACKUPS_DIR_NAME: &str = "backups";

/// One payload for the initial UI render. Keeping this in a single command
/// avoids a waterfall of IPC calls and inconsistent partial state.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub app_version: String,
    pub schema_version: i64,
    pub data_dir: String,
    pub db_path: String,
    pub backups_dir: String,
    pub workspace: Option<Workspace>,
    pub settings: AppSettings,
    pub backups: Vec<BackupInfo>,
    pub startup_warning: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub restored: BackupInfo,
    pub safety_backup: BackupInfo,
    pub bootstrap: Bootstrap,
}

/// Application service owning the local database. All UI-facing operations go
/// through this type; it never depends on Tauri.
pub struct AppService {
    db: Database,
    data_dir: PathBuf,
    db_path: PathBuf,
    backups_dir: PathBuf,
    app_version: String,
    startup_warning: Option<String>,
}

impl AppService {
    pub fn open(data_dir: impl AsRef<Path>, app_version: impl Into<String>) -> CoreResult<Self> {
        let data_dir = data_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&data_dir)?;
        let db_path = data_dir.join(DB_FILE_NAME);
        let backups_dir = data_dir.join(BACKUPS_DIR_NAME);
        let db = Database::open(&db_path)?;

        let mut service = Self {
            db,
            data_dir,
            db_path,
            backups_dir,
            app_version: app_version.into(),
            startup_warning: None,
        };
        service.run_startup_maintenance();
        Ok(service)
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub fn bootstrap(&self) -> CoreResult<Bootstrap> {
        let workspace = self.db.workspace()?;
        let settings = self.db.settings()?;
        let backups = backup::list(&self.backups_dir)?;
        Ok(Bootstrap {
            app_version: self.app_version.clone(),
            schema_version: self.db.schema_version()?,
            data_dir: self.data_dir.to_string_lossy().into_owned(),
            db_path: self.db_path.to_string_lossy().into_owned(),
            backups_dir: self.backups_dir.to_string_lossy().into_owned(),
            workspace,
            settings,
            backups,
            startup_warning: self.startup_warning.clone(),
        })
    }

    pub fn create_workspace(&self, name: &str) -> CoreResult<Workspace> {
        self.db
            .with_transaction(|tx| crate::db::repositories::workspace::create_initial(tx, name))
    }

    pub fn complete_onboarding(&self) -> CoreResult<AppSettings> {
        self.update_settings(SettingsPatch {
            onboarding_completed: Some(true),
            ..Default::default()
        })
    }

    pub fn update_settings(&self, patch: SettingsPatch) -> CoreResult<AppSettings> {
        let mut settings = self.db.settings()?;
        settings.apply(patch)?;
        self.db.save_settings(&settings)?;
        Ok(settings)
    }

    pub fn save_window_state(&self, window: WindowState) -> CoreResult<AppSettings> {
        self.update_settings(SettingsPatch {
            window: Some(window),
            ..Default::default()
        })
    }

    pub fn create_backup(&self) -> CoreResult<BackupInfo> {
        let info = backup::create(&self.db, &self.backups_dir, &self.app_version)?;
        let retention = self
            .db
            .settings()
            .map(|settings| settings.backup_retention)
            .unwrap_or(20);
        let _ = backup::prune(&self.backups_dir, retention);
        Ok(info)
    }

    pub fn list_backups(&self) -> CoreResult<Vec<BackupInfo>> {
        backup::list(&self.backups_dir)
    }

    pub fn restore_backup(&mut self, id: &str) -> CoreResult<RestoreResult> {
        let restored = backup::validate(&self.backups_dir, id)?;
        let db_file = backup::backup_path(&self.backups_dir, id)?;

        // Never replace the live database without a consistent escape hatch.
        let safety_backup = backup::create(&self.db, &self.backups_dir, &self.app_version)?;

        self.db.replace_with(&db_file)?;
        self.startup_warning = None;

        Ok(RestoreResult {
            restored,
            safety_backup,
            bootstrap: self.bootstrap()?,
        })
    }

    pub fn shutdown(&self) {
        let _ = self.db.checkpoint();
        let _ = self.db.close();
    }

    fn run_startup_maintenance(&mut self) {
        let settings = match self.db.settings() {
            Ok(settings) => settings,
            Err(error) => {
                self.startup_warning = Some(error.user_message());
                return;
            }
        };
        if !settings.auto_backup_enabled {
            return;
        }

        let needs_backup = match backup::latest_backup_age_hours(&self.backups_dir) {
            Ok(None) => true,
            Ok(Some(hours)) => hours >= backup::AUTO_BACKUP_INTERVAL_HOURS,
            Err(error) => {
                self.startup_warning = Some(error.user_message());
                true
            }
        };

        if needs_backup {
            match backup::create(&self.db, &self.backups_dir, &self.app_version) {
                Ok(_) => {
                    if let Err(error) = backup::prune(&self.backups_dir, settings.backup_retention)
                    {
                        self.startup_warning = Some(error.user_message());
                    }
                }
                Err(error) => {
                    // A failed automatic snapshot must not prevent local work.
                    self.startup_warning = Some(format!(
                        "Automatic backup could not be created: {}",
                        error.user_message()
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{LocalePreference, ThemePreference};

    fn service() -> (tempfile::TempDir, AppService) {
        let dir = tempfile::tempdir().expect("tempdir");
        let service = AppService::open(dir.path(), "0.1.0").expect("open service");
        (dir, service)
    }

    #[test]
    fn bootstraps_empty_installation() {
        let (_dir, service) = service();
        let bootstrap = service.bootstrap().expect("bootstrap");
        assert_eq!(bootstrap.schema_version, 1);
        assert!(bootstrap.workspace.is_none());
        assert!(!bootstrap.settings.onboarding_completed);
        assert!(Path::new(&bootstrap.db_path).is_file());
        // First launch with auto-backup enabled creates an initial snapshot.
        assert_eq!(bootstrap.backups.len(), 1);
    }

    #[test]
    fn creates_workspace_and_completes_onboarding() {
        let (_dir, service) = service();
        let workspace = service
            .create_workspace("Product Studio")
            .expect("workspace");
        assert_eq!(workspace.name, "Product Studio");

        let settings = service
            .update_settings(SettingsPatch {
                locale: Some(LocalePreference::Ru),
                theme: Some(ThemePreference::Light),
                ..Default::default()
            })
            .expect("settings");
        assert_eq!(settings.locale, LocalePreference::Ru);

        let completed = service.complete_onboarding().expect("complete");
        assert!(completed.onboarding_completed);

        let bootstrap = service.bootstrap().expect("bootstrap");
        assert_eq!(
            bootstrap.workspace.expect("workspace").name,
            "Product Studio"
        );
        assert!(bootstrap.settings.onboarding_completed);
    }

    #[test]
    fn restore_roundtrips_settings_and_workspace() {
        let (_dir, mut service) = service();
        service.create_workspace("Workspace A").expect("workspace");
        service
            .update_settings(SettingsPatch {
                theme: Some(ThemePreference::Dark),
                ..Default::default()
            })
            .expect("settings");
        let snapshot = service.create_backup().expect("backup");

        service
            .update_settings(SettingsPatch {
                theme: Some(ThemePreference::Light),
                ..Default::default()
            })
            .expect("settings");
        assert_eq!(
            service.bootstrap().expect("bootstrap").settings.theme,
            ThemePreference::Light
        );

        let result = service.restore_backup(&snapshot.id).expect("restore");
        assert_eq!(result.restored.id, snapshot.id);
        assert_eq!(result.bootstrap.settings.theme, ThemePreference::Dark);
        assert_eq!(
            result.bootstrap.workspace.expect("workspace").name,
            "Workspace A"
        );
        assert!(result.bootstrap.backups.iter().any(|b| b.id == snapshot.id));
        assert!(result
            .bootstrap
            .backups
            .iter()
            .any(|b| b.id == result.safety_backup.id));
    }

    #[test]
    fn failed_restore_leaves_service_usable() {
        let (_dir, mut service) = service();
        service.create_workspace("Workspace A").expect("workspace");
        let snapshot = service.create_backup().expect("backup");

        let path = Path::new(&snapshot.file_path).to_path_buf();
        let mut bytes = std::fs::read(&path).expect("read backup");
        bytes.truncate(bytes.len() / 2);
        std::fs::write(&path, bytes).expect("corrupt backup");

        let error = service.restore_backup(&snapshot.id).expect_err("must fail");
        assert_eq!(error.category(), crate::error::ErrorCategory::Backup);
        let bootstrap = service.bootstrap().expect("still usable");
        assert_eq!(bootstrap.workspace.expect("workspace").name, "Workspace A");
    }

    #[test]
    fn rejects_unknown_backup_id() {
        let (_dir, mut service) = service();
        assert!(service.restore_backup("does-not-exist").is_err());
        assert!(service.restore_backup("../forgedesk.db").is_err());
    }
}
