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
        assert_eq!(loaded.font, "sf-mono");
        assert_eq!(loaded.font_size, 16);

        let custom = Settings {
            theme: "gruvbox".into(),
            font: "menlo".into(),
            font_size: 14,
            dashboard_panels: vec!["storage".into()],
            dashboard_style: "braille".into(),
            dashboard_size: "large".into(),
        };
        save(dir.path(), &custom).unwrap();
        let loaded = load(dir.path());
        assert_eq!(loaded.theme, "gruvbox");
        assert_eq!(loaded.font, "menlo");
        assert_eq!(loaded.font_size, 14);
        // No stray temp file left behind.
        assert!(!dir.path().join("settings.json.tmp").exists());
    }

    #[test]
    fn legacy_theme_only_file_gets_font_defaults() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("settings.json"), br#"{"theme":"nord"}"#).unwrap();
        let loaded = load(dir.path());
        assert_eq!(loaded.theme, "nord");
        assert_eq!(loaded.font, "sf-mono");
        assert_eq!(loaded.font_size, 16);
        assert_eq!(loaded.dashboard_panels, vec!["activity", "tasks", "storage"]);
        assert_eq!(loaded.dashboard_style, "blocks");
        assert_eq!(loaded.dashboard_size, "medium");
    }

    #[test]
    fn corrupt_file_falls_back_to_defaults() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("settings.json"), b"{not json").unwrap();
        assert_eq!(load(dir.path()).theme, "tokyo-night");
    }
}
