// Thin adapters between the Tauri shell and `core::*`. No business logic here.
use std::sync::Mutex;

use tauri::State;

use crate::core::state::AppState;

#[tauri::command]
#[specta::specta]
pub fn is_unlocked(state: State<'_, Mutex<AppState>>) -> bool {
    state.lock().expect("state poisoned").is_unlocked()
}
