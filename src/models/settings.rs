use serde::{Deserialize, Serialize};

/// Settings for the spaced repetition system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepetitionSettings {
    /// How many times a question appears initially (default: 2)
    pub initial_repetitions: i32,
    /// How much to decrease repetitions on correct answer (default: 1)
    pub correct_decrease: i32,
    /// How much to increase repetitions on incorrect answer (default: 1)
    pub incorrect_increase: i32,
    /// Maximum repetitions cap (default: 10)
    pub max_repetitions: i32,
}

impl Default for RepetitionSettings {
    fn default() -> Self {
        Self {
            initial_repetitions: 2,
            correct_decrease: 1,
            incorrect_increase: 1,
            max_repetitions: 10,
        }
    }
}

/// Application theme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}

/// Global application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: Theme,
    pub repetition_settings: RepetitionSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            repetition_settings: RepetitionSettings::default(),
        }
    }
}
