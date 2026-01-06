mod commands;
mod db;
mod state;

use state::AppState;
use std::path::PathBuf;
use tauri::Manager;
use tauri::menu::{MenuBuilder, SubmenuBuilder, PredefinedMenuItem, MenuItem, AboutMetadata, CheckMenuItem};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tracing::{info, error, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize logging with file output and daily rotation
fn init_logging(app_data_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = app_data_dir.join("logs");
    std::fs::create_dir_all(&log_dir)?;

    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        log_dir,
        "flashnotes.log",
    );

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Keep the guard alive for the lifetime of the app
    // by leaking it (acceptable for app-lifetime resources)
    Box::leak(Box::new(_guard));

    tracing_subscriber::registry()
        .with(EnvFilter::new("flashnotes=info,warn"))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .with(fmt::layer().with_writer(std::io::stderr))
        .init();

    Ok(())
}

/// Database initialization result
struct DbInit {
    writer: rusqlite::Connection,
    reader_pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
    app_data_dir: PathBuf,
}

/// Initialize database with proper error handling
fn init_database(app: &tauri::AppHandle) -> Result<DbInit, String> {
    let app_data_dir = db::connection::get_app_data_dir(app)
        .map_err(|e| format!("{}", e))?;

    let db_path = db::connection::get_db_path(app)
        .map_err(|e| format!("{}", e))?;

    info!("Database path: {:?}", db_path);

    // Create writer connection
    let writer = db::connection::create_connection(&db_path)
        .map_err(|e| format!("Failed to create database connection: {}", e))?;

    // Initialize schema
    db::schema::initialize_schema(&writer)
        .map_err(|e| format!("Failed to initialize database schema: {}", e))?;

    // Create reader pool
    let reader_pool = db::connection::create_reader_pool(&db_path)
        .map_err(|e| format!("{}", e))?;

    // Check if backup is needed (daily backup on startup)
    if db::backup::needs_backup(&app_data_dir) {
        match db::backup::create_backup(&writer, &app_data_dir) {
            Ok(path) => info!("Daily backup created: {:?}", path),
            Err(e) => warn!("Failed to create daily backup: {}", e),
        }
    }

    Ok(DbInit {
        writer,
        reader_pool,
        app_data_dir,
    })
}

/// Show error dialog to user
fn show_error_dialog(app: &tauri::AppHandle, title: &str, message: &str) {
    let dialog = app.dialog();
    dialog.message(message)
        .title(title)
        .kind(MessageDialogKind::Error)
        .blocking_show();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // When second instance launches, focus the existing window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            // Initialize logging first (best effort)
            if let Ok(app_data_dir) = app.path().app_data_dir() {
                if let Err(e) = init_logging(&app_data_dir) {
                    eprintln!("Warning: Failed to initialize logging: {}", e);
                }
            }

            info!("Flashnotes starting up");

            // Initialize database with proper error handling
            let db_init = match init_database(&app.handle()) {
                Ok(init) => init,
                Err(e) => {
                    error!("Database initialization failed: {}", e);
                    show_error_dialog(
                        &app.handle(),
                        "Database Error",
                        &format!(
                            "Failed to initialize the database.\n\n\
                            Error: {}\n\n\
                            Please check:\n\
                            - You have write permissions to the app data directory\n\
                            - There is sufficient disk space\n\
                            - The database file is not corrupted\n\n\
                            The application will now exit.",
                            e
                        ),
                    );
                    return Err(e.into());
                }
            };

            // Manage app state
            app.manage(AppState::new(
                db_init.writer,
                db_init.reader_pool,
                db_init.app_data_dir,
            ));

            // Build macOS menu bar
            #[cfg(target_os = "macos")]
            {
                let about_metadata = AboutMetadata {
                    website: Some("https://github.com/garyblankenship/flashnotes".into()),
                    website_label: Some("GitHub".into()),
                    ..Default::default()
                };

                let app_menu = SubmenuBuilder::new(app, "Flashnotes")
                    .item(&PredefinedMenuItem::about(app, Some("About Flashnotes"), Some(about_metadata))?)
                    .separator()
                    .item(&PredefinedMenuItem::hide(app, Some("Hide Flashnotes"))?)
                    .item(&PredefinedMenuItem::hide_others(app, Some("Hide Others"))?)
                    .item(&PredefinedMenuItem::show_all(app, Some("Show All"))?)
                    .separator()
                    .item(&PredefinedMenuItem::quit(app, Some("Quit Flashnotes"))?)
                    .build()?;


                let edit_menu = SubmenuBuilder::new(app, "Edit")
                    .item(&PredefinedMenuItem::undo(app, Some("Undo"))?)
                    .item(&PredefinedMenuItem::redo(app, Some("Redo"))?)
                    .separator()
                    .item(&PredefinedMenuItem::cut(app, Some("Cut"))?)
                    .item(&PredefinedMenuItem::copy(app, Some("Copy"))?)
                    .item(&PredefinedMenuItem::paste(app, Some("Paste"))?)
                    .item(&PredefinedMenuItem::select_all(app, Some("Select All"))?)
                    .build()?;

                // Load always_on_top setting
                let always_on_top = {
                    let state = app.state::<AppState>();
                    let conn = state.writer.lock();
                    db::queries::get_settings(&conn)
                        .map(|s| s.always_on_top)
                        .unwrap_or(false)
                };

                let stay_on_top_item = CheckMenuItem::with_id(
                    app,
                    "stay_on_top",
                    "Stay on Top",
                    true,
                    always_on_top,
                    Some("CmdOrCtrl+Shift+T"),
                )?;

                let window_menu = SubmenuBuilder::new(app, "Window")
                    .item(&PredefinedMenuItem::minimize(app, Some("Minimize"))?)
                    .item(&PredefinedMenuItem::maximize(app, Some("Zoom"))?)
                    .separator()
                    .item(&stay_on_top_item)
                    .separator()
                    .item(&PredefinedMenuItem::close_window(app, Some("Close"))?)
                    .build()?;

                let help_menu = SubmenuBuilder::new(app, "Help")
                    .item(&MenuItem::with_id(app, "github", "GitHub Repository", true, None::<&str>)?)
                    .build()?;

                let menu = MenuBuilder::new(app)
                    .item(&app_menu)
                    .item(&edit_menu)
                    .item(&window_menu)
                    .item(&help_menu)
                    .build()?;

                app.set_menu(menu)?;
            }

            // Handle menu events
            app.on_menu_event(|app_handle, event| {
                match event.id().0.as_str() {
                    "github" => {
                        let _ = tauri_plugin_opener::open_url("https://github.com/garyblankenship/flashnotes", None::<&str>);
                    }
                    "stay_on_top" => {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            // Toggle the state
                            let is_on_top = window.is_always_on_top().unwrap_or(false);
                            let new_state = !is_on_top;
                            let _ = window.set_always_on_top(new_state);

                            // Update the menu item check state
                            if let Some(menu) = app_handle.menu() {
                                if let Some(item) = menu.get("stay_on_top") {
                                    if let Some(check_item) = item.as_check_menuitem() {
                                        let _ = check_item.set_checked(new_state);
                                    }
                                }
                            }

                            // Persist the setting
                            let state = app_handle.state::<AppState>();
                            let conn = state.writer.lock();
                            let _ = db::queries::set_setting(&conn, "always_on_top", if new_state { "true" } else { "false" });
                        }
                    }
                    _ => {}
                }
            });

            // Show the window after setup is complete
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();

                // Apply always_on_top setting from database
                let always_on_top = {
                    let state = app.state::<AppState>();
                    let conn = state.writer.lock();
                    db::queries::get_settings(&conn)
                        .map(|s| s.always_on_top)
                        .unwrap_or(false)
                };
                if always_on_top {
                    let _ = window.set_always_on_top(true);
                }
            }

            info!("Flashnotes startup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_buffer,
            commands::save_buffer,
            commands::get_buffer_content,
            commands::get_sidebar_data,
            commands::search_buffers,
            commands::delete_buffer,
            commands::toggle_pin,
            commands::get_settings,
            commands::set_setting,
            commands::reorder_buffers,
            commands::cleanup_empty_buffers,
            commands::toggle_always_on_top,
        ])
        .run(tauri::generate_context!())
        .expect("error while running flashnotes");
}
