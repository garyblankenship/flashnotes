mod commands;
mod db;
mod hotkey;
mod state;

use state::AppState;
use tauri::Manager;
use tauri::menu::{MenuBuilder, SubmenuBuilder, PredefinedMenuItem};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Initialize database
            let db_path = db::connection::get_db_path(&app.handle());
            println!("Database path: {:?}", db_path);

            let conn = db::connection::create_connection(&db_path)
                .expect("Failed to create database connection");

            db::schema::initialize_schema(&conn)
                .expect("Failed to initialize database schema");

            // Manage app state
            app.manage(AppState::new(conn));

            // Register global shortcut
            if let Err(e) = hotkey::setup_global_shortcut(&app.handle()) {
                eprintln!("Failed to setup global shortcut: {}", e);
            }

            // Build macOS menu bar
            #[cfg(target_os = "macos")]
            {
                let app_menu = SubmenuBuilder::new(app, "Flashnotes")
                    .item(&PredefinedMenuItem::about(app, Some("About Flashnotes"), None)?)
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

                let window_menu = SubmenuBuilder::new(app, "Window")
                    .item(&PredefinedMenuItem::minimize(app, Some("Minimize"))?)
                    .item(&PredefinedMenuItem::maximize(app, Some("Zoom"))?)
                    .separator()
                    .item(&PredefinedMenuItem::close_window(app, Some("Close"))?)
                    .build()?;

                let menu = MenuBuilder::new(app)
                    .item(&app_menu)
                    .item(&edit_menu)
                    .item(&window_menu)
                    .build()?;

                app.set_menu(menu)?;
            }

            // Show the window after setup is complete
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_buffer,
            commands::save_buffer,
            commands::get_buffer_content,
            commands::get_sidebar_data,
            commands::search_buffers,
            commands::archive_buffer,
            commands::delete_buffer_permanently,
            commands::toggle_pin,
            commands::get_buffer_count,
            commands::get_settings,
            commands::set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running flashnotes");
}
