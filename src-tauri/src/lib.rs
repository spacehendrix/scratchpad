mod commands;
pub mod core;

use std::sync::Mutex;

use crate::core::state::AppState;

fn specta_builder() -> tauri_specta::Builder {
    tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            commands::is_unlocked,
            commands::unlock,
            commands::lock,
            commands::start_fresh,
            commands::list_documents,
            commands::get_document,
            commands::save_document,
            commands::toggle_pin,
            commands::delete_document,
            commands::search,
            commands::storage_stats,
            commands::run_retention_now,
            commands::get_settings,
            commands::set_settings,
        ])
}

pub fn run() {
    let builder = specta_builder();

    #[cfg(debug_assertions)]
    export_bindings(&builder);

    tauri::Builder::default()
        // A second launch focuses the existing window instead of racing on
        // the database.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(builder.invoke_handler())
        .setup(|app| {
            spawn_retention_timer(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Re-run retention every 6 hours while the app is open (and unlocked).
fn spawn_retention_timer(handle: tauri::AppHandle) {
    use crate::core::clock::{Clock, SystemClock};
    use tauri::Manager;

    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(6 * 60 * 60));
        let state = handle.state::<Mutex<AppState>>();
        let mut guard = state.lock().expect("state poisoned");
        if let Ok(session) = guard.session_mut() {
            let _ = crate::core::retention::run(session, SystemClock.now_ms());
        }
    });
}

#[cfg(debug_assertions)]
fn export_bindings(builder: &tauri_specta::Builder) {
    use specta_typescript::{BigIntExportBehavior, Typescript};
    builder
        .export(
            Typescript::default()
                .bigint(BigIntExportBehavior::Number)
                .header("// @ts-nocheck"),
            "../src/lib/bindings.ts",
        )
        .expect("failed to export typescript bindings");
}

#[cfg(test)]
mod tests {
    // Regenerates src/lib/bindings.ts without launching the app:
    // `cargo test export_bindings`.
    #[test]
    fn export_bindings() {
        super::export_bindings(&super::specta_builder());
    }
}
