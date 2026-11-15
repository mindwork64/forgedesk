use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};

pub const DEFAULT_ROUTE: &str = "/my-issues";
pub const MIN_WINDOW_WIDTH: u32 = 1100;
pub const MIN_WINDOW_HEIGHT: u32 = 700;
pub const DEFAULT_WINDOW_WIDTH: u32 = 1440;
pub const DEFAULT_WINDOW_HEIGHT: u32 = 900;
pub const MAX_WINDOW_DIMENSION: u32 = 16_384;
pub const MIN_BACKUP_RETENTION: u32 = 3;
pub const MAX_BACKUP_RETENTION: u32 = 100;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    /// UI/UX spec §6.0: Dark is the default theme for first run.
    #[default]
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LocalePreference {
    #[default]
    System,
    En,
    Ru,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowState {
    pub width: u32,
    pub height: u32,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: DEFAULT_WINDOW_WIDTH,
            height: DEFAULT_WINDOW_HEIGHT,
            x: None,
            y: None,
            maximized: false,
        }
    }
}

impl WindowState {
    /// Clamp physically implausible values from the window manager into a
    /// range the application layout supports. Position is left untouched.
    pub fn sanitized(mut self) -> Self {
        self.width = self.width.clamp(MIN_WINDOW_WIDTH, MAX_WINDOW_DIMENSION);
        self.height = self.height.clamp(MIN_WINDOW_HEIGHT, MAX_WINDOW_DIMENSION);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub locale: LocalePreference,
    /// Persisted appearance preference. `system` follows the OS.
    pub theme: ThemePreference,
    pub sidebar_collapsed: bool,
    /// Last successfully visited internal route, restored on next launch.
    pub last_route: String,
    pub onboarding_completed: bool,
    pub detail_pane_open: bool,
    pub auto_backup_enabled: bool,
    /// Number of managed backups kept before the oldest are pruned.
    pub backup_retention: u32,
    pub window: WindowState,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            locale: LocalePreference::default(),
            theme: ThemePreference::default(),
            sidebar_collapsed: false,
            last_route: DEFAULT_ROUTE.to_owned(),
            onboarding_completed: false,
            detail_pane_open: false,
            auto_backup_enabled: true,
            backup_retention: 20,
            window: WindowState::default(),
        }
    }
}

impl AppSettings {
    pub fn normalized(mut self) -> Self {
        self.window = self.window.sanitized();
        if validate_route(&self.last_route).is_err() {
            self.last_route = DEFAULT_ROUTE.to_owned();
        }
        self.backup_retention = self
            .backup_retention
            .clamp(MIN_BACKUP_RETENTION, MAX_BACKUP_RETENTION);
        self
    }

    pub fn apply(&mut self, patch: SettingsPatch) -> CoreResult<()> {
        if let Some(locale) = patch.locale {
            self.locale = locale;
        }
        if let Some(theme) = patch.theme {
            self.theme = theme;
        }
        if let Some(collapsed) = patch.sidebar_collapsed {
            self.sidebar_collapsed = collapsed;
        }
        if let Some(route) = patch.last_route {
            validate_route(&route)?;
            self.last_route = route;
        }
        if let Some(completed) = patch.onboarding_completed {
            self.onboarding_completed = completed;
        }
        if let Some(open) = patch.detail_pane_open {
            self.detail_pane_open = open;
        }
        if let Some(enabled) = patch.auto_backup_enabled {
            self.auto_backup_enabled = enabled;
        }
        if let Some(retention) = patch.backup_retention {
            if !(MIN_BACKUP_RETENTION..=MAX_BACKUP_RETENTION).contains(&retention) {
                return Err(CoreError::validation(
                    "backupRetention",
                    format!(
                        "Backup retention must be between {MIN_BACKUP_RETENTION} and {MAX_BACKUP_RETENTION}."
                    ),
                ));
            }
            self.backup_retention = retention;
        }
        if let Some(window) = patch.window {
            self.window = window.sanitized();
        }
        Ok(())
    }
}

/// Only internal, absolute SPA routes may be persisted. Reject protocol-relative
/// or traversal-like values so a damaged settings row cannot trigger navigation
/// outside the app.
fn validate_route(route: &str) -> CoreResult<()> {
    let valid = route.starts_with('/')
        && !route.starts_with("//")
        && !route.contains("://")
        && !route.contains("..")
        && route.chars().count() <= 200
        && !route.chars().any(|c| c.is_control() || c == '\\');
    if !valid {
        return Err(CoreError::validation(
            "lastRoute",
            "Invalid application route.",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsPatch {
    pub locale: Option<LocalePreference>,
    pub theme: Option<ThemePreference>,
    pub sidebar_collapsed: Option<bool>,
    pub last_route: Option<String>,
    pub onboarding_completed: Option<bool>,
    pub detail_pane_open: Option<bool>,
    pub auto_backup_enabled: Option<bool>,
    pub backup_retention: Option<u32>,
    pub window: Option<WindowState>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let settings = AppSettings::default();
        assert_eq!(settings.locale, LocalePreference::System);
        assert_eq!(settings.theme, ThemePreference::Dark);
        assert_eq!(settings.last_route, "/my-issues");
        assert!(!settings.onboarding_completed);
        assert_eq!(settings.window.width, 1440);
        assert_eq!(settings.window.height, 900);
    }

    #[test]
    fn patch_validates_route_and_retention() {
        let mut settings = AppSettings::default();
        assert!(settings
            .clone()
            .apply(SettingsPatch {
                last_route: Some("https://evil.example".into()),
                ..Default::default()
            })
            .is_err());
        assert!(settings
            .clone()
            .apply(SettingsPatch {
                last_route: Some("//evil.example".into()),
                ..Default::default()
            })
            .is_err());
        assert!(settings
            .clone()
            .apply(SettingsPatch {
                backup_retention: Some(1),
                ..Default::default()
            })
            .is_err());

        settings
            .apply(SettingsPatch {
                last_route: Some("/projects".into()),
                sidebar_collapsed: Some(true),
                ..Default::default()
            })
            .expect("valid patch");
        assert_eq!(settings.last_route, "/projects");
        assert!(settings.sidebar_collapsed);
    }

    #[test]
    fn window_state_is_clamped() {
        let state = WindowState {
            width: 10,
            height: 20,
            x: None,
            y: None,
            maximized: false,
        }
        .sanitized();
        assert_eq!(state.width, MIN_WINDOW_WIDTH);
        assert_eq!(state.height, MIN_WINDOW_HEIGHT);
    }
}
