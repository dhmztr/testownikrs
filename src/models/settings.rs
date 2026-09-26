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

/// Default number of consecutive correct answers needed to show the hot streak
pub const DEFAULT_HOT_STREAK_THRESHOLD: u32 = 3;
/// Allowed range for the hot streak threshold
pub const MIN_HOT_STREAK_THRESHOLD: u32 = 2;
pub const MAX_HOT_STREAK_THRESHOLD: u32 = 50;

/// Global application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub theme: Theme,
    pub repetition_settings: RepetitionSettings,
    /// Whether the hot streak indicator is shown
    pub hot_streak_enabled: bool,
    /// How many consecutive correct answers are needed to show the hot streak
    pub hot_streak_threshold: u32,
    /// Animated fire rising from the bottom of the screen during a hot streak
    pub fire_effect_enabled: bool,
}

impl AppSettings {
    /// Hot streak level for a given streak: 0 = none, 1 = hot, 2 = hotter, 3+ = on fire.
    /// Each level is reached after another `hot_streak_threshold` correct answers.
    pub fn hot_streak_level(&self, streak: u32) -> u32 {
        if !self.hot_streak_enabled {
            return 0;
        }
        let threshold = self.hot_streak_threshold.max(MIN_HOT_STREAK_THRESHOLD);
        (streak / threshold).min(3)
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            repetition_settings: RepetitionSettings::default(),
            hot_streak_enabled: true,
            hot_streak_threshold: DEFAULT_HOT_STREAK_THRESHOLD,
            fire_effect_enabled: true,
        }
    }
}
