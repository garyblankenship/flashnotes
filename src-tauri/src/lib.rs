mod commands;
mod db;
mod hotkey;
mod state;

use state::AppState;
use tauri::Manager;
use tauri::menu::{MenuBuilder, SubmenuBuilder, PredefinedMenuItem, MenuItem, AboutMetadata};
use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // When second instance launches, focus the existing window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
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

                let file_menu = SubmenuBuilder::new(app, "File")
                    .item(&MenuItem::with_id(app, "import_sublime", "Import from Sublime...", true, None::<&str>)?)
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

                let help_menu = SubmenuBuilder::new(app, "Help")
                    .item(&MenuItem::with_id(app, "github", "GitHub Repository", true, None::<&str>)?)
                    .build()?;

                let menu = MenuBuilder::new(app)
                    .item(&app_menu)
                    .item(&file_menu)
                    .item(&edit_menu)
                    .item(&window_menu)
                    .item(&help_menu)
                    .build()?;

                app.set_menu(menu)?;
            }

            // Handle menu events
            app.on_menu_event(|app_handle, event| {
                match event.id().0.as_str() {
                    "import_sublime" => {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.emit("import-sublime", ());
                        }
                    }
                    "github" => {
                        let _ = tauri_plugin_opener::open_url("https://github.com/garyblankenship/flashnotes", None::<&str>);
                    }
                    _ => {}
                }
            });

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
            commands::delete_buffer,
            commands::toggle_pin,
            commands::get_settings,
            commands::set_setting,
            commands::import_sublime_buffers,
            commands::reorder_buffers,
            commands::cleanup_empty_buffers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running flashnotes");
}
