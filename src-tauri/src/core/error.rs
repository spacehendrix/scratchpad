/// Single error surface crossing IPC, serialized as a tagged union so the
/// frontend can react to specific kinds (e.g. `locked` → show unlock screen).
#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
#[serde(tag = "kind", content = "detail", rename_all = "camelCase")]
pub enum CoreError {
    #[error("app is locked")]
    Locked,
    #[error("keychain access denied")]
    KeychainDenied,
    #[error("keychain item missing")]
    KeychainItemMissing,
    #[error("storage limit reached and only pinned documents remain")]
    StorageFull,
    #[error("document not found")]
    NotFound,
    #[error("io error: {0}")]
    Io(String),
    #[error("document {id} could not be decrypted")]
    Corrupt { id: String },
}

impl From<std::io::Error> for CoreError {
    fn from(e: std::io::Error) -> Self {
        CoreError::Io(e.to_string())
    }
}

pub type CoreResult<T> = Result<T, CoreError>;
