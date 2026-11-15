//! ForgeDesk core: local domain, SQLite storage, settings, backups and
//! application services.
//!
//! This crate intentionally has no dependency on Tauri, a webview, an HTTP
//! client or an AI SDK. The desktop shell wraps it through typed commands.

pub mod app;
pub mod backup;
pub mod db;
pub mod domain;
pub mod error;
pub mod git;
pub mod github_auth;

pub use app::AppService;
pub use error::{CoreError, WireError};
