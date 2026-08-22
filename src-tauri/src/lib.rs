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
        ])
}

pub fn run() {
    let builder = specta_builder();

    #[cfg(debug_assertions)]
    export_bindings(&builder);

    tauri::Builder::default()
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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
