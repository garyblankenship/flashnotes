mod commands;
mod db;
mod state;

use state::AppState;
use tauri::Manager;
use tauri::menu::{MenuBuilder, SubmenuBuilder, PredefinedMenuItem, MenuItem, AboutMetadata, CheckMenuItem};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            // Initialize database
            let db_path = db::connection::get_db_path(&app.handle());
            println!("Database path: {:?}", db_path);

            let conn = db::connection::create_connection(&db_path)
                .expect("Failed to create database connection");

            db::schema::initialize_schema(&conn)
                .expect("Failed to initialize database schema");

            // Manage app state
            app.manage(AppState::new(conn));

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
                    let conn = state.db.lock();
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
                            let conn = state.db.lock();
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
                    let conn = state.db.lock();
                    db::queries::get_settings(&conn)
                        .map(|s| s.always_on_top)
                        .unwrap_or(false)
                };
                if always_on_top {
                    let _ = window.set_always_on_top(true);
                }
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
            commands::reorder_buffers,
            commands::cleanup_empty_buffers,
            commands::toggle_always_on_top,
        ])
        .run(tauri::generate_context!())
        .expect("error while running flashnotes");
}
