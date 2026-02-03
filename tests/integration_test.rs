use testownik_rs::{models::*, parser, persistence::Storage};
use std::path::PathBuf;

#[test]
fn test_load_example_questions_and_create_session() {
    // Load the example questions
    let questions = parser::read_questions_from_file("example_questions.txt")
        .expect("Failed to load example questions");

    assert!(!questions.is_empty(), "Should have loaded questions");
    println!("Loaded {} questions", questions.len());

    // Create session questions with initial repetitions
    let settings = RepetitionSettings::default();
    let session_questions: Vec<SessionQuestion> = questions
        .into_iter()
        .map(|q| SessionQuestion::new(q, settings.initial_repetitions))
        .collect();

    // Create a quiz session
    let session = QuizSession::new(
        PathBuf::from("example_questions.txt"),
        session_questions,
        settings.clone(),
    );

    assert!(!session.is_complete(), "New session should not be complete");
    assert_eq!(
        session.active_count(),
        session.questions.len(),
        "All questions should be active initially"
    );
    assert_eq!(session.progress_percentage(), 0.0, "Progress should be 0%");

    println!("Session created: {}", session.quiz_name);
    println!("Active questions: {}", session.active_count());
}

#[test]
fn test_session_persistence() {
    // Create a storage instance
    let storage = Storage::new().expect("Failed to create storage");

    // Load example questions
    let questions = parser::read_questions_from_file("example_questions.txt")
        .expect("Failed to load example questions");

    let settings = RepetitionSettings::default();
    let session_questions: Vec<SessionQuestion> = questions
        .into_iter()
        .map(|q| SessionQuestion::new(q, settings.initial_repetitions))
        .collect();

    let mut session = QuizSession::new(
        PathBuf::from("example_questions.txt"),
        session_questions,
        settings.clone(),
    );

    // Save the session
    storage
        .save_session(&session)
        .expect("Failed to save session");

    // Load the session back
    let loaded_session = storage
        .load_session(&session.id)
        .expect("Failed to load session");

    assert_eq!(loaded_session.id, session.id);
    assert_eq!(loaded_session.quiz_name, session.quiz_name);
    assert_eq!(loaded_session.questions.len(), session.questions.len());

    // Clean up - delete the test session
    storage
        .delete_session(&session.id)
        .expect("Failed to delete test session");

    println!("Session persistence test passed");
}

#[test]
fn test_quiz_workflow() {
    // Simulate a complete quiz workflow
    let questions = parser::read_questions_from_file("example_questions.txt")
        .expect("Failed to load example questions");

    let settings = RepetitionSettings::default();
    let session_questions: Vec<SessionQuestion> = questions
        .into_iter()
        .map(|q| SessionQuestion::new(q, settings.initial_repetitions))
        .collect();

    let mut session = QuizSession::new(
        PathBuf::from("example_questions.txt"),
        session_questions,
        settings.clone(),
    );

    let initial_count = session.active_count();
    assert!(initial_count > 0, "Should have active questions");

    // Get first question and answer it correctly
    if let Some(question) = session.current_question() {
        let tag = question.question.tag.clone();
        
        // Simulate correct answer
        session.update_question(&tag, true);
        
        // Move to next question
        session.next_question();
    }

    // Progress should have changed
    assert!(
        session.progress_percentage() > 0.0,
        "Progress should increase after correct answer"
    );

    println!("Quiz workflow test passed");
    println!("Initial questions: {}", initial_count);
    println!("Progress: {:.1}%", session.progress_percentage());
}
