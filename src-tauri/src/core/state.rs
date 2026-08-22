/// Global application state, managed behind a `Mutex` by the shell layer.
/// Starts locked; the unlocked session (master key + open store + catalog)
/// arrives with the store/keychain milestones.
pub enum AppState {
    Locked,
}

impl AppState {
    pub fn is_unlocked(&self) -> bool {
        !matches!(self, AppState::Locked)
    }
}

impl Default for AppState {
    fn default() -> Self {
        AppState::Locked
    }
}
