use crate::models::question::SessionQuestion;
use crate::models::settings::RepetitionSettings;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A quiz session with progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizSession {
    /// Unique identifier for this session
    pub id: String,
    /// Path to the quiz file
    pub quiz_path: PathBuf,
    /// Name/title of the quiz (derived from filename)
    pub quiz_name: String,
    /// All questions with their repetition tracking
    pub questions: Vec<SessionQuestion>,
    /// Current question index in the active set
    pub current_index: usize,
    /// Settings for this session
    pub settings: RepetitionSettings,
    /// When this session was created
    pub created_at: DateTime<Utc>,
    /// When this session was last updated
    pub updated_at: DateTime<Utc>,
    /// Transient flag: true if the current question was just mastered (not serialized)
    #[serde(skip, default)]
    pub current_question_just_mastered: bool,
}

impl QuizSession {
    pub fn new(
        quiz_path: PathBuf,
        questions: Vec<SessionQuestion>,
        settings: RepetitionSettings,
    ) -> Self {
        let quiz_name = quiz_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown Quiz")
            .to_string();

        let id = format!("{}_{}", quiz_name.replace(' ', "_"), Utc::now().timestamp());

        let now = Utc::now();

        Self {
            id,
            quiz_path,
            quiz_name,
            questions,
            current_index: 0,
            settings,
            created_at: now,
            updated_at: now,
            current_question_just_mastered: false,
        }
    }

    /// Get the current question
    pub fn current_question(&self) -> Option<&SessionQuestion> {
        let active = self.active_questions();
        active.get(self.current_index).copied()
    }

    /// Get all questions that still need to be answered (repetitions_remaining > 0)
    pub fn active_questions(&self) -> Vec<&SessionQuestion> {
        self.questions
            .iter()
            .filter(|q| q.repetitions_remaining > 0)
            .collect()
    }

    /// Get the total count of active questions
    pub fn active_count(&self) -> usize {
        self.active_questions().len()
    }

    /// Check if the session is complete (all questions have 0 repetitions)
    pub fn is_complete(&self) -> bool {
        self.active_count() == 0
    }

    /// Move to the next question
    pub fn next_question(&mut self) {
        let active_questions = self.active_questions();
        let active_count = active_questions.len();
        
        if active_count == 0 {
            self.current_index = 0;
            self.current_question_just_mastered = false;
            self.updated_at = Utc::now();
            return;
        }
        
        // If the current question was just mastered, it was removed from the active list.
        // The next question has slid into the current_index position, so we don't need to increment.
        if self.current_question_just_mastered {
            // Clear the flag
            self.current_question_just_mastered = false;
            // Just clamp current_index to valid range (in case it's now out of bounds)
            if self.current_index >= active_count {
                self.current_index = 0;
            }
        } else {
            // Normal case: move to the next question
            self.current_index = (self.current_index + 1) % active_count;
        }
        
        self.updated_at = Utc::now();
    }

    /// Update the repetitions for a question based on the answer
    pub fn update_question(&mut self, question_tag: &str, is_correct: bool) {
        // Get the current question's tag before updating
        let current_question_tag = self.current_question().map(|q| q.question.tag.clone());
        
        // Find and update the question
        self.current_question_just_mastered = if let Some(q) = self
            .questions
            .iter_mut()
            .find(|q| q.question.tag == question_tag)
        {
            let was_active = q.repetitions_remaining > 0;
            
            if is_correct {
                q.record_correct(self.settings.correct_decrease);
            } else {
                q.record_incorrect(
                    self.settings.incorrect_increase,
                    self.settings.max_repetitions,
                );
            }
            
            let is_now_inactive = q.repetitions_remaining == 0;
            // Question became inactive if it was active before and is inactive now
            // AND it's the current question
            was_active && is_now_inactive && question_tag == current_question_tag.as_deref().unwrap_or("")
        } else {
            false
        };
        
        self.updated_at = Utc::now();
    }

    /// Get progress percentage (0-100)
    pub fn progress_percentage(&self) -> f32 {
        if self.questions.is_empty() {
            return 100.0;
        }

        let total_initial_reps: i32 =
            self.questions.len() as i32 * self.settings.initial_repetitions;
        let remaining_reps: i32 = self.questions.iter().map(|q| q.repetitions_remaining).sum();

        if total_initial_reps == 0 {
            return 100.0;
        }

        ((total_initial_reps - remaining_reps) as f32 / total_initial_reps as f32 * 100.0)
            .max(0.0)
            .min(100.0)
    }
}

/// Metadata for a saved session (for display in session list)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub id: String,
    pub quiz_name: String,
    pub progress_percentage: f32,
    pub active_count: usize,
    pub total_count: usize,
    pub updated_at: DateTime<Utc>,
}

impl From<&QuizSession> for SessionMetadata {
    fn from(session: &QuizSession) -> Self {
        Self {
            id: session.id.clone(),
            quiz_name: session.quiz_name.clone(),
            progress_percentage: session.progress_percentage(),
            active_count: session.active_count(),
            total_count: session.questions.len(),
            updated_at: session.updated_at,
        }
    }
}
