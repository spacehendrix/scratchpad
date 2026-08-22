// Thin adapters between the Tauri shell and `core::*`. No business logic here.
use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, Manager, State};

use crate::core::error::{CoreError, CoreResult};
use crate::core::keychain::KeychainKeySource;
use crate::core::state::AppState;

type Shared<'a> = State<'a, Mutex<AppState>>;

fn data_dir(app: &AppHandle) -> CoreResult<PathBuf> {
    app.path()
        .app_data_dir()
        .map_err(|e| CoreError::Io(e.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn is_unlocked(state: Shared<'_>) -> bool {
    state.lock().expect("state poisoned").is_unlocked()
}

/// Async so the (possibly seconds-long) Touch ID sheet never blocks the main
/// thread. First run generates the key silently; later runs prompt.
#[tauri::command]
#[specta::specta]
pub async fn unlock(app: AppHandle, state: Shared<'_>) -> Result<(), CoreError> {
    let dir = data_dir(&app)?;
    let source = KeychainKeySource::new();
    let result = state.lock().expect("state poisoned").unlock(&source, &dir);
    #[cfg(debug_assertions)]
    match &result {
        Ok(()) => eprintln!("[scratchpad] unlocked"),
        Err(e) => eprintln!("[scratchpad] unlock failed: {e:?}"),
    }
    result
}

#[tauri::command]
#[specta::specta]
pub fn lock(state: Shared<'_>) {
    state.lock().expect("state poisoned").lock();
}

/// Recovery path when the stored data can no longer be decrypted (the
/// keychain item was deleted/regenerated). Quarantines the old database
/// (rename, never delete) and unlocks against a fresh one. The UI gates this
/// behind an explicit typed confirmation.
#[tauri::command]
#[specta::specta]
pub async fn start_fresh(app: AppHandle, state: Shared<'_>) -> Result<(), CoreError> {
    let dir = data_dir(&app)?;
    let mut guard = state.lock().expect("state poisoned");
    if guard.is_unlocked() {
        return Err(CoreError::Io("cannot start fresh while unlocked".into()));
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    crate::core::store::quarantine_db(&dir, &ts.to_string())?;
    guard.unlock(&KeychainKeySource::new(), &dir)
}
