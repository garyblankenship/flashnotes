use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

/// Register the global shortcut for toggling window visibility
pub fn setup_global_shortcut(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Cmd+Shift+Space
    let shortcut = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::Space);

    let app_handle = app.clone();
    app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            toggle_window(&app_handle);
        }
    })?;

    app.global_shortcut().register(shortcut)?;

    println!("Global shortcut registered: Cmd+Shift+Space");
    Ok(())
}

/// Toggle the main window visibility
fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        match window.is_visible() {
            Ok(true) => {
                let _ = window.hide();
            }
            Ok(false) => {
                let _ = window.show();
                let _ = window.set_focus();
                // Emit event to frontend to focus editor
                let _ = app.emit("focus-editor", ());
            }
            Err(e) => {
                eprintln!("Failed to check window visibility: {}", e);
            }
        }
    }
}

/// Unregister all global shortcuts (called on app exit)
#[allow(dead_code)]
pub fn cleanup_shortcuts(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    app.global_shortcut().unregister_all()?;
    Ok(())
}
