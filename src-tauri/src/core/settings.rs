//! Plain (unencrypted) app settings — deliberately outside the encrypted
//! store so the unlock screen itself can be themed. Nothing sensitive lives
//! here.

use std::path::Path;

use crate::core::error::{CoreError, CoreResult};
use crate::core::model::Settings;

const FILE: &str = "settings.json";

/// Missing or unreadable settings fall back to defaults — never an error.
pub fn load(dir: &Path) -> Settings {
    std::fs::read(dir.join(FILE))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

/// Atomic write: temp file + rename, so a crash can never truncate settings.
pub fn save(dir: &Path, settings: &Settings) -> CoreResult<()> {
    std::fs::create_dir_all(dir)?;
    let json = serde_json::to_vec_pretty(settings).map_err(|e| CoreError::Io(e.to_string()))?;
    let tmp = dir.join(format!("{FILE}.tmp"));
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, dir.join(FILE))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let loaded = load(dir.path());
        assert_eq!(loaded.theme, "tokyo-night");

        let custom = Settings {
            theme: "gruvbox".into(),
        };
        save(dir.path(), &custom).unwrap();
        assert_eq!(load(dir.path()).theme, "gruvbox");
        // No stray temp file left behind.
        assert!(!dir.path().join("settings.json.tmp").exists());
    }

    #[test]
    fn corrupt_file_falls_back_to_defaults() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("settings.json"), b"{not json").unwrap();
        assert_eq!(load(dir.path()).theme, "tokyo-night");
    }
}
