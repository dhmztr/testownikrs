use crate::models::{AppSettings, QuizSession, SessionMetadata};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Storage manager for sessions and settings
pub struct Storage {
    sessions_dir: PathBuf,
    settings_path: PathBuf,
}

impl Storage {
    /// Create a new storage manager
    pub fn new() -> Result<Self> {
        // Use a data directory in the user's home or current directory
        let base_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("testownik-rs");

        fs::create_dir_all(&base_dir).context("Failed to create storage directory")?;

        let sessions_dir = base_dir.join("sessions");
        fs::create_dir_all(&sessions_dir).context("Failed to create sessions directory")?;

        let settings_path = base_dir.join("settings.json");

        Ok(Self {
            sessions_dir,
            settings_path,
        })
    }

    /// Save a quiz session
    pub fn save_session(&self, session: &QuizSession) -> Result<()> {
        let session_path = self.sessions_dir.join(format!("{}.json", session.id));
        let json = serde_json::to_string_pretty(session)
            .context("Failed to serialize session")?;
        fs::write(&session_path, json)
            .context("Failed to write session file")?;
        Ok(())
    }

    /// Load a quiz session by ID
    pub fn load_session(&self, session_id: &str) -> Result<QuizSession> {
        let session_path = self.sessions_dir.join(format!("{}.json", session_id));
        let json = fs::read_to_string(&session_path)
            .context("Failed to read session file")?;
        let session = serde_json::from_str(&json)
            .context("Failed to deserialize session")?;
        Ok(session)
    }

    /// List all available sessions
    pub fn list_sessions(&self) -> Result<Vec<SessionMetadata>> {
        let mut metadata_list = Vec::new();

        if !self.sessions_dir.exists() {
            return Ok(metadata_list);
        }

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(session) = self.load_session_from_path(&path) {
                    metadata_list.push(SessionMetadata::from(&session));
                }
            }
        }

        // Sort by most recently updated
        metadata_list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(metadata_list)
    }

    /// Delete a session by ID
    pub fn delete_session(&self, session_id: &str) -> Result<()> {
        let session_path = self.sessions_dir.join(format!("{}.json", session_id));
        if session_path.exists() {
            fs::remove_file(&session_path)
                .context("Failed to delete session file")?;
        }
        Ok(())
    }

    /// Load settings
    pub fn load_settings(&self) -> Result<AppSettings> {
        if !self.settings_path.exists() {
            return Ok(AppSettings::default());
        }

        let json = fs::read_to_string(&self.settings_path)
            .context("Failed to read settings file")?;
        let settings = serde_json::from_str(&json)
            .context("Failed to deserialize settings")?;
        Ok(settings)
    }

    /// Save settings
    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        let json = serde_json::to_string_pretty(settings)
            .context("Failed to serialize settings")?;
        fs::write(&self.settings_path, json)
            .context("Failed to write settings file")?;
        Ok(())
    }

    // Helper method to load session from a path
    fn load_session_from_path(&self, path: &Path) -> Result<QuizSession> {
        let json = fs::read_to_string(path)?;
        let session = serde_json::from_str(&json)?;
        Ok(session)
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new().expect("Failed to create storage")
    }
}

// Add dirs crate for cross-platform directory support
mod dirs {
    use std::path::PathBuf;

    pub fn data_local_dir() -> Option<PathBuf> {
        if cfg!(target_os = "windows") {
            std::env::var("LOCALAPPDATA").ok().map(PathBuf::from)
        } else if cfg!(target_os = "macos") {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join("Library").join("Application Support"))
        } else {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".local").join("share"))
        }
    }
}
