#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::persistence::Storage;
    use crate::types::*;
    use std::path::PathBuf;

    fn create_test_question() -> Question {
        Question {
            tag: "test_q1".to_string(),
            content_type: ContentType::Text,
            content: QuestionContent::Text("Test question?".to_string()),
            question_type: QuestionType::Single,
            answers: vec![
                Answer {
                    id: 0,
                    answer_type: AnswerType::Text,
                    content: AnswerContent::Single {
                        content: "Answer 1".to_string(),
                    },
                    is_correct: Some(true),
                    correct_option_id: None,
                    options: None,
                },
                Answer {
                    id: 1,
                    answer_type: AnswerType::Text,
                    content: AnswerContent::Single {
                        content: "Answer 2".to_string(),
                    },
                    is_correct: Some(false),
                    correct_option_id: None,
                    options: None,
                },
            ],
        }
    }

    #[test]
    fn test_session_question_check_answer() {
        let question = create_test_question();
        let session_q = SessionQuestion::new(question, 2);

        // Test correct answer
        assert!(session_q.check_answer(&[0]));

        // Test incorrect answer
        assert!(!session_q.check_answer(&[1]));

        // Test no answer
        assert!(!session_q.check_answer(&[]));

        // Test both answers (should be incorrect since only answer 0 is correct)
        assert!(!session_q.check_answer(&[0, 1]));
    }

    #[test]
    fn test_session_question_record_correct() {
        let question = create_test_question();
        let mut session_q = SessionQuestion::new(question, 2);

        assert_eq!(session_q.repetitions_remaining, 2);
        assert_eq!(session_q.correct_count, 0);

        session_q.record_correct(1);

        assert_eq!(session_q.repetitions_remaining, 1);
        assert_eq!(session_q.correct_count, 1);

        session_q.record_correct(1);

        assert_eq!(session_q.repetitions_remaining, 0);
        assert_eq!(session_q.correct_count, 2);
    }

    #[test]
    fn test_session_question_record_incorrect() {
        let question = create_test_question();
        let mut session_q = SessionQuestion::new(question, 2);

        assert_eq!(session_q.repetitions_remaining, 2);
        assert_eq!(session_q.incorrect_count, 0);

        session_q.record_incorrect(1, 10);

        assert_eq!(session_q.repetitions_remaining, 3);
        assert_eq!(session_q.incorrect_count, 1);

        session_q.record_incorrect(1, 10);

        assert_eq!(session_q.repetitions_remaining, 4);
        assert_eq!(session_q.incorrect_count, 2);
    }

    #[test]
    fn test_quiz_session_active_questions() {
        let question1 = create_test_question();
        let mut session_q1 = SessionQuestion::new(question1, 2);

        let mut question2 = create_test_question();
        question2.tag = "test_q2".to_string();
        let mut session_q2 = SessionQuestion::new(question2, 0); // Already completed

        let questions = vec![session_q1.clone(), session_q2.clone()];

        let session = QuizSession::new(
            PathBuf::from("test.txt"),
            questions,
            RepetitionSettings::default(),
        );

        // Should only have 1 active question (session_q1)
        assert_eq!(session.active_count(), 1);
        assert!(!session.is_complete());

        // Mark the active question as complete
        session_q1.repetitions_remaining = 0;
        session_q2.repetitions_remaining = 0;

        let questions = vec![session_q1, session_q2];
        let session = QuizSession::new(
            PathBuf::from("test.txt"),
            questions,
            RepetitionSettings::default(),
        );

        assert_eq!(session.active_count(), 0);
        assert!(session.is_complete());
    }

    #[test]
    fn test_session_progress() {
        let question = create_test_question();
        let session_q = SessionQuestion::new(question, 2);

        let mut session = QuizSession::new(
            PathBuf::from("test.txt"),
            vec![session_q],
            RepetitionSettings::default(),
        );

        // Initial progress should be 0%
        assert_eq!(session.progress_percentage(), 0.0);

        // Update the question to have 1 repetition remaining
        session.questions[0].repetitions_remaining = 1;

        // Progress should be 50%
        assert_eq!(session.progress_percentage(), 50.0);

        // Complete the question
        session.questions[0].repetitions_remaining = 0;

        // Progress should be 100%
        assert_eq!(session.progress_percentage(), 100.0);
    }

    #[test]
    fn test_repetition_settings_default() {
        let settings = RepetitionSettings::default();
        assert_eq!(settings.initial_repetitions, 2);
        assert_eq!(settings.correct_decrease, 1);
        assert_eq!(settings.incorrect_increase, 1);
        assert_eq!(settings.max_repetitions, 10);
    }

    #[test]
    fn test_storage_creation() {
        // Just test that storage can be created
        let storage_result = Storage::new();
        assert!(storage_result.is_ok());
    }
}
