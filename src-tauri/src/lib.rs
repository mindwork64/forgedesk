pub mod ai;
mod commands;
pub mod state;

use tauri::{Manager, PhysicalPosition, PhysicalSize, RunEvent, WindowEvent};
use tracing_subscriber::EnvFilter;

use crate::state::AppState;

pub fn run() {
    init_logging();

    let builder = tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir().map_err(|error| {
                format!("ForgeDesk could not resolve its application data directory: {error}")
            })?;
            let version = app.package_info().version.to_string();
            app.manage(AppState::initialize(data_dir, version));

            if let Some(window) = app.get_webview_window("main") {
                restore_window_state(app.handle(), &window);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { .. } = event {
                let state = window.state::<AppState>();
                let size = window.inner_size().unwrap_or(PhysicalSize::new(1440, 900));
                let position = window.outer_position().ok();
                let window_state = forgedesk_core::domain::WindowState {
                    width: size.width.max(1100),
                    height: size.height.max(700),
                    x: position.map(|p| p.x),
                    y: position.map(|p| p.y),
                    maximized: window.is_maximized().unwrap_or(false),
                };
                state.save_window_state(window_state);
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::create_workspace,
            commands::save_settings,
            commands::complete_onboarding,
            commands::save_window_state,
            commands::list_backups,
            commands::create_backup,
            commands::restore_backup,
            commands::list_workflow,
            commands::list_issue_statuses,
            commands::list_labels,
            commands::list_projects,
            commands::list_issues,
            commands::get_issue,
            commands::create_issue,
            commands::update_issue,
            commands::archive_issue,
            commands::batch_update_issues,
            commands::duplicate_issue,
            commands::add_issue_comment,
            commands::add_issue_relation,
            commands::remove_issue_relation,
            commands::list_saved_views,
            commands::create_saved_view,
            commands::update_saved_view,
            commands::delete_saved_view,
            commands::list_project_cards,
            commands::get_project,
            commands::create_project,
            commands::update_project,
            commands::archive_project,
            commands::create_milestone,
            commands::update_milestone,
            commands::assign_milestone_issue,
            commands::list_initiatives,
            commands::get_initiative,
            commands::create_initiative,
            commands::update_initiative,
            commands::list_cycles,
            commands::get_cycle,
            commands::create_cycle,
            commands::update_cycle,
            commands::assign_cycle_issue,
            commands::list_documents,
            commands::get_document,
            commands::create_document,
            commands::update_document,
            commands::delete_document,
            commands::export_document,
            commands::git_available,
            commands::list_repositories,
            commands::add_local_repository,
            commands::remove_repository,
            commands::get_git_overview,
            commands::create_git_branch,
            commands::switch_git_branch,
            commands::get_git_log,
            commands::get_git_diff,
            commands::get_api_monitor,
            commands::set_api_paused,
            commands::export_api_diagnostics,
            commands::github_start_device_flow,
            commands::github_poll_device_flow,
            commands::github_connection,
            commands::github_refresh,
            commands::github_disconnect,
            commands::github_discover_repositories,
            commands::github_import_issues,
            commands::github_list_pulls,
            commands::github_pull_detail,
            commands::github_review_requested,
            commands::github_ci_overview,
            commands::github_commit_checks,
            commands::github_pull_link,
            commands::github_link_pull,
            commands::github_unlink_pull,
            commands::github_pull_capabilities,
            commands::github_merge_pull,
            commands::github_close_pull,
            commands::github_submit_review,
            commands::inbox_list,
            commands::inbox_mark_read,
            commands::inbox_archive,
            commands::inbox_mark_all_read,
            commands::get_insights,
            commands::automation_list_rules,
            commands::automation_create,
            commands::automation_toggle,
            commands::automation_delete,
            commands::automation_preview,
            commands::automation_run,
            commands::automation_runs,
            commands::dashboard_get,
            commands::dashboard_add_widget,
            commands::dashboard_remove_widget,
            commands::dashboard_update_width,
            commands::dashboard_reorder,
            commands::github_sync_pull_events,
            commands::provider_capabilities,
            commands::ai_dispatch,
        ]);

    let app = builder
        .build(tauri::generate_context!())
        .expect("ForgeDesk could not initialize the desktop shell");

    app.run(|app_handle, event| {
        if let RunEvent::Exit = event {
            app_handle.state::<AppState>().shutdown();
        }
    });
}

fn init_logging() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("forgedesk=info,warn"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

/// Restore persisted size/position only when the saved position still lies
/// within an available monitor. Off-screen restoration would strand the window.
fn restore_window_state(app: &tauri::AppHandle, window: &tauri::WebviewWindow) {
    let Some(bootstrap) = app.state::<AppState>().try_bootstrap() else {
        return;
    };
    let saved = bootstrap.settings.window;

    if saved.maximized {
        let _ = window.maximize();
        return;
    }

    let _ = window.set_size(PhysicalSize::new(saved.width, saved.height));

    if let (Some(x), Some(y)) = (saved.x, saved.y) {
        let position = PhysicalPosition::new(x, y);
        let width = i64::from(saved.width);
        let height = i64::from(saved.height);
        let visible = window
            .available_monitors()
            .unwrap_or_default()
            .iter()
            .any(|monitor| {
                let monitor_position = monitor.position();
                let monitor_size = monitor.size();
                let monitor_left = i64::from(monitor_position.x);
                let monitor_top = i64::from(monitor_position.y);
                let monitor_right = monitor_left + i64::from(monitor_size.width);
                let monitor_bottom = monitor_top + i64::from(monitor_size.height);
                let left = i64::from(x);
                let top = i64::from(y);
                left < monitor_right
                    && left + width > monitor_left
                    && top < monitor_bottom
                    && top + height > monitor_top
            });
        if visible {
            let _ = window.set_position(position);
        } else {
            let _ = window.center();
        }
    }
}
