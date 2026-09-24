use crate::gui::{MainMenuState, QuizState, CompletionState};
use testownik_rs::models::{
    AppSettings, QuizSession, RepetitionSettings, Theme, MAX_HOT_STREAK_THRESHOLD,
    MIN_HOT_STREAK_THRESHOLD,
};
use testownik_rs::types::QuestionType;
use testownik_rs::persistence::Storage;
use iced::{window, Application, Command, Element, Font, Settings as IcedSettings, Size};
use std::path::PathBuf;
use iced::keyboard;
use iced::event::{self, Event};
use iced::subscription::Subscription;

/// Default font that supports Polish characters (system will use built-in fallback fonts)
pub const DEFAULT_FONT: Font = Font::with_name("Segoe UI");

/// Main application state
pub struct TestownikApp {
    pub storage: Storage,
    pub settings: AppSettings,
    pub screen: Screen,
}

/// Different screens in the application
#[derive(Debug, Clone)]
pub enum Screen {
    MainMenu(MainMenuState),
    Quiz(QuizState),
    CompletionScreen(CompletionState),
}

/// Messages that can be sent in the application
#[derive(Debug, Clone)]
pub enum Message {
    // Main menu messages
    SelectQuizFile,
    OpenFilePicker,
    QuizFileSelected(String),
    FilePickerClosed,
    LoadSession(String),
    DeleteSession(String),
    ToggleTheme,
    UpdateSettings(AppSettings),
    ToggleSettingsPanel,
    SetInitialRepetitions(i32),
    SetCorrectDecrease(i32),
    SetIncorrectIncrease(i32),
    SetMaxRepetitions(i32),
    ResetRepetitionSettings,
    ToggleHotStreak,
    SetHotStreakThreshold(i32),
    DismissError,
    
    // Quiz messages
    SelectAnswer(usize),
    SelectOption(usize, usize), // (dropdown_index, option_index)
    SubmitAnswer,
    NextQuestion,
    SaveAndExit,
    
    // Keyboard input
    SelectAnswerByNumber(usize),
    SpacePressed,

    // Copy question text to clipboard
    CopyText(String),

    // Toggle quiz-specific settings panel
    ToggleQuizSettingsPanel,
    
    // Completion
    CompleteQuiz,
    
    // Navigation
    BackToMenu,
    None,
}

impl Application for TestownikApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    /// Optional path of a quiz file to open right away (first CLI argument)
    type Flags = Option<String>;

    fn new(quiz_path: Self::Flags) -> (Self, Command<Self::Message>) {
        let storage = Storage::new().expect("Failed to create storage");
        let settings = storage.load_settings().unwrap_or_default();

        let screen = Screen::MainMenu(MainMenuState::load(&storage));
        let mut app = Self {
            storage,
            settings,
            screen,
        };

        if let Some(path) = quiz_path {
            app.load_quiz_from_file(&path);
        }

        (app, Command::none())
    }

    fn title(&self) -> String {
        match &self.screen {
            Screen::MainMenu(_) => "Testownik-rs - Aplikacja do Nauki".to_string(),
            Screen::Quiz(state) => format!("Testownik-rs - {}", state.session.quiz_name),
            Screen::CompletionScreen(_) => "Testownik-rs - Quiz Ukończony".to_string(),
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::ToggleTheme => {
                self.settings.theme = match self.settings.theme {
                    Theme::Light => Theme::Dark,
                    Theme::Dark => Theme::Light,
                };
                let _ = self.storage.save_settings(&self.settings);
                Command::none()
            }
            
            Message::BackToMenu => {
                self.go_to_menu();
                Command::none()
            }

            Message::ToggleHotStreak => {
                self.settings.hot_streak_enabled = !self.settings.hot_streak_enabled;
                let _ = self.storage.save_settings(&self.settings);
                Command::none()
            }

            Message::SetHotStreakThreshold(v) => {
                self.settings.hot_streak_threshold = (v.max(0) as u32)
                    .clamp(MIN_HOT_STREAK_THRESHOLD, MAX_HOT_STREAK_THRESHOLD);
                let _ = self.storage.save_settings(&self.settings);
                Command::none()
            }

            Message::DismissError => {
                if let Screen::MainMenu(ref mut state) = self.screen {
                    state.error = None;
                }
                Command::none()
            }
            
            Message::SelectQuizFile => {
                // Load the example file directly
                Command::perform(async { "example_questions.txt".to_string() }, Message::QuizFileSelected)
            }
            
            Message::OpenFilePicker => {
                // Open native file picker dialog
                Command::perform(
                    async {
                        let file = rfd::AsyncFileDialog::new()
                            .add_filter("Pliki Quiz", &["txt", "json", "quiz"])
                            .add_filter("Pliki Tekstowe", &["txt"])
                            .add_filter("Pliki JSON", &["json"])
                            .add_filter("Wszystkie Pliki", &["*"])
                            .set_title("Wybierz Plik Quiz")
                            .pick_file()
                            .await;
                        
                        file.map(|f| f.path().to_string_lossy().to_string())
                    },
                    |result| {
                        match result {
                            Some(path) => Message::QuizFileSelected(path),
                            None => Message::FilePickerClosed,
                        }
                    }
                )
            }
            
            Message::FilePickerClosed => {
                // User cancelled the file picker, do nothing
                Command::none()
            }
            
            Message::QuizFileSelected(path) => {
                // Load quiz and create new session
                self.load_quiz_from_file(&path);
                Command::none()
            }
            
            Message::LoadSession(session_id) => {
                match self.storage.load_session(&session_id) {
                    Ok(session) => {
                        let image_base_dir = image_base_dir(&session);
                        self.screen = Screen::Quiz(QuizState::new(session, image_base_dir));
                    }
                    Err(e) => {
                        self.screen = Screen::MainMenu(MainMenuState::with_error(
                            &self.storage,
                            format!("Nie udało się wczytać sesji: {e:#}"),
                        ));
                    }
                }
                Command::none()
            }
            
            Message::DeleteSession(session_id) => {
                let _ = self.storage.delete_session(&session_id);
                if let Screen::MainMenu(ref mut state) = self.screen {
                    state.refresh_sessions(&self.storage);
                }
                Command::none()
            }
            
            Message::UpdateSettings(new_settings) => {
                self.settings = new_settings;
                let _ = self.storage.save_settings(&self.settings);
                Command::none()
            }

            Message::ToggleSettingsPanel => {
                if let Screen::MainMenu(ref mut state) = self.screen {
                    state.show_settings = !state.show_settings;
                }
                Command::none()
            }

            Message::ToggleQuizSettingsPanel => {
                if let Screen::Quiz(ref mut state) = self.screen {
                    state.show_settings = !state.show_settings;
                }
                Command::none()
            }

            Message::CopyText(s) => iced::clipboard::write(s),

            Message::SetInitialRepetitions(v) => {
                // ponytail: Quiz screen edits this session's settings; MainMenu edits global defaults
                if let Screen::Quiz(ref mut state) = self.screen {
                    let max = state.session.settings.max_repetitions;
                    let clamped = v.clamp(1, max);
                    let old_initial = state.session.settings.initial_repetitions;
                    state.session.settings.initial_repetitions = clamped;
                    let delta = clamped - old_initial;
                    if delta != 0 {
                        // Only questions still in play are adjusted: mastered ones stay
                        // mastered and active ones never silently drop out of the quiz
                        // (that would shift the current question under the user)
                        for q in state.session.questions.iter_mut() {
                            if q.repetitions_remaining > 0 {
                                q.repetitions_remaining = (q.repetitions_remaining + delta).clamp(1, max);
                            }
                        }
                    }
                    let _ = self.storage.save_session(&state.session);
                } else {
                    self.settings.repetition_settings.initial_repetitions =
                        v.clamp(1, self.settings.repetition_settings.max_repetitions);
                    let _ = self.storage.save_settings(&self.settings);
                }
                Command::none()
            }

            Message::SetCorrectDecrease(v) => {
                if let Screen::Quiz(ref mut state) = self.screen {
                    state.session.settings.correct_decrease = v.clamp(1, 10);
                    let _ = self.storage.save_session(&state.session);
                } else {
                    self.settings.repetition_settings.correct_decrease = v.clamp(1, 10);
                    let _ = self.storage.save_settings(&self.settings);
                }
                Command::none()
            }

            Message::SetIncorrectIncrease(v) => {
                if let Screen::Quiz(ref mut state) = self.screen {
                    state.session.settings.incorrect_increase = v.clamp(0, 20);
                    let _ = self.storage.save_session(&state.session);
                } else {
                    self.settings.repetition_settings.incorrect_increase = v.clamp(0, 20);
                    let _ = self.storage.save_settings(&self.settings);
                }
                Command::none()
            }

            Message::SetMaxRepetitions(v) => {
                if let Screen::Quiz(ref mut state) = self.screen {
                    let min = state.session.settings.initial_repetitions.max(1);
                    let max = v.clamp(min, 50);
                    state.session.settings.max_repetitions = max;
                    for q in state.session.questions.iter_mut() {
                        q.repetitions_remaining = q.repetitions_remaining.min(max);
                    }
                    let _ = self.storage.save_session(&state.session);
                } else {
                    let min = self.settings.repetition_settings.initial_repetitions.max(1);
                    self.settings.repetition_settings.max_repetitions = v.clamp(min, 50);
                    let _ = self.storage.save_settings(&self.settings);
                }
                Command::none()
            }

            Message::ResetRepetitionSettings => {
                if let Screen::Quiz(ref mut state) = self.screen {
                    let defaults = RepetitionSettings::default();
                    for q in state.session.questions.iter_mut() {
                        q.repetitions_remaining = q.repetitions_remaining.min(defaults.max_repetitions);
                    }
                    state.session.settings = defaults;
                    let _ = self.storage.save_session(&state.session);
                } else {
                    self.settings.repetition_settings = RepetitionSettings::default();
                    let _ = self.storage.save_settings(&self.settings);
                }
                Command::none()
            }
            
            Message::SelectAnswer(index) => {
                if let Screen::Quiz(ref mut state) = self.screen {
                    state.toggle_answer(index);
                }
                Command::none()
            }
            
            Message::SelectOption(dropdown_idx, option_idx) => {
                if let Screen::Quiz(ref mut state) = self.screen {
                    state.select_dropdown_option(dropdown_idx, option_idx);
                }
                Command::none()
            }
            
            Message::SubmitAnswer => {
                self.submit_answer();
                Command::none()
            }
            
            Message::NextQuestion => {
                self.advance_question();
                Command::none()
            }
            
            Message::SaveAndExit => {
                if let Screen::Quiz(ref mut state) = self.screen {
                    // An answer that was checked but not yet applied must not be lost
                    if state.pending_submission.is_some() {
                        state.next_question();
                    }
                    let _ = self.storage.save_session(&state.session);
                }
                self.go_to_menu();
                Command::none()
            }
            
            Message::SelectAnswerByNumber(num) => {
                if let Screen::Quiz(ref mut state) = self.screen {
                    // Only allow keyboard selection for Single-type questions
                    if let Some(question) = state.session.current_question() {
                        if question.question.question_type == QuestionType::Single {
                            // Get the actual answer index from randomized indices
                            if let Some(&original_idx) = state.randomized_answer_indices.get(num) {
                                state.toggle_answer(original_idx);
                            }
                        }
                    }
                }
                Command::none()
            }
            
            Message::SpacePressed => {
                match self.screen {
                    Screen::Quiz(ref state) if state.answer_submitted => self.advance_question(),
                    Screen::Quiz(_) => self.submit_answer(),
                    _ => {}
                }
                Command::none()
            }
            
            Message::CompleteQuiz => {
                if let Screen::CompletionScreen(ref state) = self.screen {
                    // Delete the completed session
                    let _ = self.storage.delete_session(&state.session.id);
                }
                self.go_to_menu();
                Command::none()
            }
            
            Message::None => Command::none(),
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match &self.screen {
            Screen::MainMenu(state) => state.view(&self.settings),
            Screen::Quiz(state) => state.view(&self.settings),
            Screen::CompletionScreen(state) => state.view(&self.settings),
        }
    }

    fn theme(&self) -> Self::Theme {
        match self.settings.theme {
            Theme::Light => iced::Theme::Light,
            Theme::Dark => iced::Theme::Dark,
        }
    }
    
    fn subscription(&self) -> Subscription<Self::Message> {
        event::listen().map(|event| {
            if let Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event {
                match key.as_ref() {
                    keyboard::Key::Character("1") => Message::SelectAnswerByNumber(0),
                    keyboard::Key::Character("2") => Message::SelectAnswerByNumber(1),
                    keyboard::Key::Character("3") => Message::SelectAnswerByNumber(2),
                    keyboard::Key::Character("4") => Message::SelectAnswerByNumber(3),
                    keyboard::Key::Character("5") => Message::SelectAnswerByNumber(4),
                    keyboard::Key::Character("6") => Message::SelectAnswerByNumber(5),
                    keyboard::Key::Character("7") => Message::SelectAnswerByNumber(6),
                    keyboard::Key::Character("8") => Message::SelectAnswerByNumber(7),
                    keyboard::Key::Character("9") => Message::SelectAnswerByNumber(8),
                    keyboard::Key::Named(keyboard::key::Named::Space)
                    | keyboard::Key::Named(keyboard::key::Named::Enter) => Message::SpacePressed,
                    _ => Message::None,
                }
            } else {
                Message::None
            }
        })
    }
}

/// Images referenced by a quiz are looked up relative to the quiz file
fn image_base_dir(session: &QuizSession) -> PathBuf {
    session
        .quiz_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

impl TestownikApp {
    fn go_to_menu(&mut self) {
        self.screen = Screen::MainMenu(MainMenuState::load(&self.storage));
    }

    fn submit_answer(&mut self) {
        if let Screen::Quiz(ref mut state) = self.screen {
            state.submit_answer();
            let _ = self.storage.save_session(&state.session);
        }
    }

    fn advance_question(&mut self) {
        if let Screen::Quiz(ref mut state) = self.screen {
            if !state.answer_submitted {
                return;
            }
            state.next_question();
            let _ = self.storage.save_session(&state.session);

            if state.session.is_complete() {
                self.screen = Screen::CompletionScreen(CompletionState::new(state.session.clone()));
            }
        }
    }

    fn load_quiz_from_file(&mut self, path: &str) {
        use testownik_rs::models::SessionQuestion;
        use testownik_rs::parser;
        use testownik_rs::utils;

        let questions = match parser::read_questions_from_file(path) {
            Ok(q) if !q.is_empty() => q,
            Ok(_) => {
                self.screen = Screen::MainMenu(MainMenuState::with_error(
                    &self.storage,
                    format!("Plik {path} nie zawiera żadnych poprawnych pytań."),
                ));
                return;
            }
            Err(e) => {
                self.screen = Screen::MainMenu(MainMenuState::with_error(
                    &self.storage,
                    format!("Nie udało się wczytać pliku {path}: {e:#}"),
                ));
                return;
            }
        };

        let session_questions: Vec<SessionQuestion> = questions
            .into_iter()
            .map(|q| SessionQuestion::new(q, self.settings.repetition_settings.initial_repetitions))
            .collect();

        // Randomize question order
        let shuffled_questions = utils::randomize_order(&session_questions);

        let mut session = QuizSession::new(
            PathBuf::from(path),
            shuffled_questions,
            self.settings.repetition_settings.clone(),
        );

        // Use the title from JSON quiz files when available
        if let Some(name) = parser::get_json_quiz_title(path) {
            session.quiz_name = name;
        }

        let _ = self.storage.save_session(&session);

        let image_base_dir = image_base_dir(&session);
        self.screen = Screen::Quiz(QuizState::new(session, image_base_dir));
    }
}

/// Run the application
pub fn run_app() -> iced::Result {
    TestownikApp::run(IcedSettings {
        window: window::Settings {
            size: Size::new(900.0, 700.0),
            min_size: Some(Size::new(640.0, 480.0)),
            ..window::Settings::default()
        },
        default_font: DEFAULT_FONT,
        ..IcedSettings::with_flags(std::env::args().nth(1))
    })
}
