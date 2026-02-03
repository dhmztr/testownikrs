#[cfg(test)]
mod next_question_tests {
    use crate::models::*;
    use crate::types::*;
    use std::path::PathBuf;

    fn create_test_question_with_tag(tag: &str) -> Question {
        Question {
            tag: tag.to_string(),
            content_type: ContentType::Text,
            content: QuestionContent::Text(format!("Question {}?", tag)),
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
            ],
        }
    }

    #[test]
    fn test_next_question_after_mastering_current() {
        // Create a session with 2 questions, each with 1 repetition
        let q1 = create_test_question_with_tag("q1");
        let q2 = create_test_question_with_tag("q2");
        
        let sq1 = SessionQuestion::new(q1, 1);
        let sq2 = SessionQuestion::new(q2, 1);
        
        let mut session = QuizSession::new(
            PathBuf::from("test.txt"),
            vec![sq1, sq2],
            RepetitionSettings::default(),
        );

        // Start at index 0 (Q1)
        assert_eq!(session.current_index, 0);
        let current = session.current_question().unwrap();
        assert_eq!(current.question.tag, "q1");

        // Move to next question (Q2)
        session.next_question();
        assert_eq!(session.current_index, 1);
        let current = session.current_question().unwrap();
        assert_eq!(current.question.tag, "q2");

        // Answer Q2 correctly (it will be mastered)
        session.update_question("q2", true);
        
        // Q2 should now have 0 repetitions
        assert_eq!(session.questions[1].repetitions_remaining, 0);
        
        // Active questions should only be [Q1]
        assert_eq!(session.active_count(), 1);
        
        // current_index is still 1, but there's only 1 active question
        assert_eq!(session.current_index, 1);
        
        // Call next_question - this should handle the out-of-bounds index
        session.next_question();
        
        // After next_question, we should see Q1 (the only remaining question)
        let current = session.current_question().unwrap();
        assert_eq!(current.question.tag, "q1");
        
        // The index should be valid
        assert!(session.current_index < session.active_count());
    }

    #[test]
    fn test_next_question_normal_progression() {
        // Create a session with 3 questions, each with 2 repetitions
        let q1 = create_test_question_with_tag("q1");
        let q2 = create_test_question_with_tag("q2");
        let q3 = create_test_question_with_tag("q3");
        
        let sq1 = SessionQuestion::new(q1, 2);
        let sq2 = SessionQuestion::new(q2, 2);
        let sq3 = SessionQuestion::new(q3, 2);
        
        let mut session = QuizSession::new(
            PathBuf::from("test.txt"),
            vec![sq1, sq2, sq3],
            RepetitionSettings::default(),
        );

        // Progress through questions without mastering any
        assert_eq!(session.current_question().unwrap().question.tag, "q1");
        
        session.next_question();
        assert_eq!(session.current_question().unwrap().question.tag, "q2");
        
        session.next_question();
        assert_eq!(session.current_question().unwrap().question.tag, "q3");
        
        session.next_question();
        // Should wrap around to q1
        assert_eq!(session.current_question().unwrap().question.tag, "q1");
    }

    #[test]
    fn test_next_question_with_middle_question_mastered() {
        // Create session with 3 questions
        let q1 = create_test_question_with_tag("q1");
        let q2 = create_test_question_with_tag("q2");
        let q3 = create_test_question_with_tag("q3");
        
        let sq1 = SessionQuestion::new(q1, 1);
        let sq2 = SessionQuestion::new(q2, 1);
        let sq3 = SessionQuestion::new(q3, 1);
        
        let mut session = QuizSession::new(
            PathBuf::from("test.txt"),
            vec![sq1, sq2, sq3],
            RepetitionSettings::default(),
        );

        // At Q1
        assert_eq!(session.current_question().unwrap().question.tag, "q1");
        
        // Move to Q2
        session.next_question();
        assert_eq!(session.current_question().unwrap().question.tag, "q2");
        
        // Master Q2
        session.update_question("q2", true);
        assert_eq!(session.active_count(), 2); // Only Q1 and Q3 remain
        
        // Move to next - should show Q3 (not Q1)
        session.next_question();
        let current = session.current_question().unwrap();
        println!("After mastering Q2 and calling next_question, current question is: {}", current.question.tag);
        
        // This is the key test: after mastering the middle question,
        // next_question should show Q3, not wrap back to Q1
        assert_eq!(current.question.tag, "q3", 
            "After mastering Q2 at index 1, next_question should show Q3, not Q1");
    }
}
