// Business/document-management core. No tauri imports allowed anywhere in
// this tree — the UI layer must stay swappable.
pub mod clock;
pub mod crypto;
pub mod error;
pub mod keychain;
pub mod model;
pub mod retention;
pub mod state;
pub mod store;
