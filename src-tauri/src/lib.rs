mod commands;
mod db;
mod hotkey;
mod state;

use state::AppState;
use tauri::Manager;

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

            // Hide from dock (macOS only) - app lives in tray
            #[cfg(target_os = "macos")]
            {
                use tauri::ActivationPolicy;
                app.set_activation_policy(ActivationPolicy::Accessory);
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
