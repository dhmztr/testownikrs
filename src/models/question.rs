use serde::{Deserialize, Serialize};

/// Re-export the question types from the root types module
pub use crate::types::{
    Answer, AnswerContent, AnswerOption, AnswerType, ContentPart, ContentType, Question,
    QuestionContent, QuestionType,
};

/// Extended question data with session tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionQuestion {
    pub question: Question,
    /// Number of times this question should still appear
    pub repetitions_remaining: i32,
    /// How many times answered correctly
    pub correct_count: u32,
    /// How many times answered incorrectly
    pub incorrect_count: u32,
    /// Track user's last answer for this question
    pub last_answer: Option<Vec<usize>>,
}

impl SessionQuestion {
    pub fn new(question: Question, initial_repetitions: i32) -> Self {
        Self {
            question,
            repetitions_remaining: initial_repetitions,
            correct_count: 0,
            incorrect_count: 0,
            last_answer: None,
        }
    }

    /// Check if a given answer is correct for this question
    pub fn check_answer(&self, selected_indices: &[usize]) -> bool {
        match &self.question.question_type {
            QuestionType::Single => {
                let correct_indices: Vec<usize> = self
                    .question
                    .answers
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| a.is_correct.unwrap_or(false))
                    .map(|(i, _)| i)
                    .collect();

                let mut selected_sorted = selected_indices.to_vec();
                selected_sorted.sort_unstable();
                let mut correct_sorted = correct_indices;
                correct_sorted.sort_unstable();

                selected_sorted == correct_sorted
            }
            QuestionType::Select => {
                // For select questions, check if each dropdown has correct selection
                if selected_indices.len() != self.question.answers.len() {
                    return false;
                }

                self.question
                    .answers
                    .iter()
                    .enumerate()
                    .all(|(idx, answer)| {
                        if let Some(&selected_option) = selected_indices.get(idx) {
                            answer.correct_option_id == Some(selected_option)
                        } else {
                            false
                        }
                    })
            }
        }
    }

    /// Record a correct answer
    pub fn record_correct(&mut self, repetition_decrease: i32) {
        self.correct_count += 1;
        self.repetitions_remaining = (self.repetitions_remaining - repetition_decrease).max(0);
    }

    /// Record an incorrect answer
    pub fn record_incorrect(&mut self, repetition_increase: i32, max_repetitions: i32) {
        self.incorrect_count += 1;
        self.repetitions_remaining =
            (self.repetitions_remaining + repetition_increase).min(max_repetitions);
    }
}
