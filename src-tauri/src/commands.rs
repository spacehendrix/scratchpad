// Thin adapters between the Tauri shell and `core::*`. No business logic here.
use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, Manager, State};

use crate::core::clock::{Clock, SystemClock};
use crate::core::error::{CoreError, CoreResult};
use crate::core::keychain::KeychainKeySource;
use crate::core::model::{DocMeta, Document, RetentionReport, SearchHit, Settings, StorageStats};
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
    let mut guard = state.lock().expect("state poisoned");
    let result = guard.unlock(&source, &dir);
    #[cfg(debug_assertions)]
    match &result {
        Ok(()) => eprintln!("[scratchpad] unlocked"),
        Err(e) => eprintln!("[scratchpad] unlock failed: {e:?}"),
    }
    // Retention runs right after every unlock.
    if result.is_ok() {
        if let Ok(session) = guard.session_mut() {
            match crate::core::retention::run(session, SystemClock.now_ms()) {
                Ok(_report) => {
                    #[cfg(debug_assertions)]
                    eprintln!("[scratchpad] retention: {_report:?}");
                }
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("[scratchpad] retention failed: {_e:?}");
                }
            }
        }
    }
    result
}

#[tauri::command]
#[specta::specta]
pub fn lock(state: Shared<'_>) {
    state.lock().expect("state poisoned").lock();
}

#[tauri::command]
#[specta::specta]
pub fn list_documents(state: Shared<'_>) -> Result<Vec<DocMeta>, CoreError> {
    Ok(state.lock().expect("state poisoned").session()?.list())
}

#[tauri::command]
#[specta::specta]
pub fn get_document(state: Shared<'_>, id: String) -> Result<Document, CoreError> {
    state.lock().expect("state poisoned").session()?.get(&id)
}

#[tauri::command]
#[specta::specta]
pub fn save_document(
    state: Shared<'_>,
    id: Option<String>,
    title: Option<String>,
    body: String,
) -> Result<DocMeta, CoreError> {
    let now = SystemClock.now_ms();
    state
        .lock()
        .expect("state poisoned")
        .session_mut()?
        .save(id, title, body, now)
}

#[tauri::command]
#[specta::specta]
pub fn toggle_pin(state: Shared<'_>, id: String) -> Result<DocMeta, CoreError> {
    state
        .lock()
        .expect("state poisoned")
        .session_mut()?
        .toggle_pin(&id)
}

#[tauri::command]
#[specta::specta]
pub fn delete_document(state: Shared<'_>, id: String) -> Result<(), CoreError> {
    state
        .lock()
        .expect("state poisoned")
        .session_mut()?
        .delete(&id)
}

/// Async so a full-archive body scan cannot freeze the UI thread.
#[tauri::command]
#[specta::specta]
pub async fn search(
    state: Shared<'_>,
    query: String,
    scope_archived: bool,
) -> Result<Vec<SearchHit>, CoreError> {
    crate::core::search::search(
        state.lock().expect("state poisoned").session()?,
        &query,
        scope_archived,
    )
}

#[tauri::command]
#[specta::specta]
pub fn storage_stats(state: Shared<'_>) -> Result<StorageStats, CoreError> {
    Ok(state.lock().expect("state poisoned").session()?.stats())
}

#[tauri::command]
#[specta::specta]
pub fn run_retention_now(state: Shared<'_>) -> Result<RetentionReport, CoreError> {
    let now = SystemClock.now_ms();
    crate::core::retention::run(state.lock().expect("state poisoned").session_mut()?, now)
}

/// Settings are readable before unlock (the unlock screen is themed).
#[tauri::command]
#[specta::specta]
pub fn get_settings(app: AppHandle) -> Result<Settings, CoreError> {
    Ok(crate::core::settings::load(&data_dir(&app)?))
}

#[tauri::command]
#[specta::specta]
pub fn set_settings(app: AppHandle, settings: Settings) -> Result<(), CoreError> {
    crate::core::settings::save(&data_dir(&app)?, &settings)
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
