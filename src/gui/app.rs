use crate::gui::{MainMenuState, QuizState, CompletionState};
use testownik_rs::models::{AppSettings, RepetitionSettings, Theme};
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
    
    // Quiz messages
    SelectAnswer(usize),
    SelectOption(usize, usize), // (dropdown_index, option_index)
    SubmitAnswer,
    NextQuestion,
    SaveAndExit,
    
    // Keyboard input
    SelectAnswerByNumber(usize),
    SpacePressed,
    
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
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let storage = Storage::new().expect("Failed to create storage");
        let settings = storage.load_settings().unwrap_or_default();

        let app = Self {
            storage,
            settings,
            screen: Screen::MainMenu(MainMenuState::new()),
        };

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
                self.screen = Screen::MainMenu(MainMenuState::new());
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
                if let Ok(session) = self.storage.load_session(&session_id) {
                    let image_base_dir = session.quiz_path
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| PathBuf::from("."));
                    self.screen = Screen::Quiz(QuizState::new(session, image_base_dir));
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

            Message::SetInitialRepetitions(v) => {
                self.settings.repetition_settings.initial_repetitions =
                    v.clamp(1, self.settings.repetition_settings.max_repetitions);
                let _ = self.storage.save_settings(&self.settings);
                Command::none()
            }

            Message::SetCorrectDecrease(v) => {
                self.settings.repetition_settings.correct_decrease = v.clamp(1, 10);
                let _ = self.storage.save_settings(&self.settings);
                Command::none()
            }

            Message::SetIncorrectIncrease(v) => {
                self.settings.repetition_settings.incorrect_increase = v.clamp(0, 20);
                let _ = self.storage.save_settings(&self.settings);
                Command::none()
            }

            Message::SetMaxRepetitions(v) => {
                let min = self.settings.repetition_settings.initial_repetitions.max(1);
                self.settings.repetition_settings.max_repetitions = v.clamp(min, 50);
                let _ = self.storage.save_settings(&self.settings);
                Command::none()
            }

            Message::ResetRepetitionSettings => {
                self.settings.repetition_settings = RepetitionSettings::default();
                let _ = self.storage.save_settings(&self.settings);
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
                if let Screen::Quiz(ref mut state) = self.screen {
                    state.submit_answer();
                    let _ = self.storage.save_session(&state.session);
                }
                Command::none()
            }
            
            Message::NextQuestion => {
                if let Screen::Quiz(ref mut state) = self.screen {
                    state.next_question();
                    let _ = self.storage.save_session(&state.session);
                    
                    // Check if quiz is complete
                    if state.session.is_complete() {
                        self.screen = Screen::CompletionScreen(CompletionState::new(state.session.clone()));
                    }
                }
                Command::none()
            }
            
            Message::SaveAndExit => {
                if let Screen::Quiz(ref state) = self.screen {
                    let _ = self.storage.save_session(&state.session);
                }
                self.screen = Screen::MainMenu(MainMenuState::new());
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
                if let Screen::Quiz(ref mut state) = self.screen {
                    if state.answer_submitted {
                        // Space pressed after submission = next question
                        state.next_question();
                        let _ = self.storage.save_session(&state.session);
                        
                        if state.session.is_complete() {
                            self.screen = Screen::CompletionScreen(CompletionState::new(state.session.clone()));
                        }
                    } else {
                        // Space pressed before submission = submit answer
                        state.submit_answer();
                        let _ = self.storage.save_session(&state.session);
                    }
                }
                Command::none()
            }
            
            Message::CompleteQuiz => {
                if let Screen::CompletionScreen(ref state) = self.screen {
                    // Delete the completed session
                    let _ = self.storage.delete_session(&state.session.id);
                }
                self.screen = Screen::MainMenu(MainMenuState::new());
                Command::none()
            }
            
            Message::None => Command::none(),
        }
    }

    fn view(&self) -> Element<Self::Message> {
        match &self.screen {
            Screen::MainMenu(state) => state.view(&self.settings, &self.storage),
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
                    keyboard::Key::Named(keyboard::key::Named::Space) => Message::SpacePressed,
                    _ => Message::None,
                }
            } else {
                Message::None
            }
        })
    }
}

impl TestownikApp {
    fn load_quiz_from_file(&mut self, path: &str) {
        use testownik_rs::models::{QuizSession, SessionQuestion};
        use testownik_rs::parser;
        use testownik_rs::utils;
        use std::path::PathBuf;

        // Load questions from file
        if let Ok(questions) = parser::read_questions_from_file(path) {
            if questions.is_empty() {
                return; // Don't create empty sessions
            }
            
            let session_questions: Vec<SessionQuestion> = questions
                .into_iter()
                .map(|q| {
                    SessionQuestion::new(
                        q,
                        self.settings.repetition_settings.initial_repetitions,
                    )
                })
                .collect();

            // Randomize question order
            let shuffled_questions = utils::randomize_order(&session_questions);

            // Try to get custom quiz name from JSON files
            let quiz_name = parser::get_json_quiz_title(path);
            
            let mut session = QuizSession::new(
                PathBuf::from(path),
                shuffled_questions,
                self.settings.repetition_settings.clone(),
            );
            
            // Override quiz name if we got one from JSON
            if let Some(name) = quiz_name {
                session.quiz_name = name;
            }

            // Save the new session
            let _ = self.storage.save_session(&session);

            // Switch to quiz screen
            let image_base_dir = PathBuf::from(path)
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            self.screen = Screen::Quiz(QuizState::new(session, image_base_dir));
        }
    }
}

/// Run the application
pub fn run_app() -> iced::Result {
    TestownikApp::run(IcedSettings {
        window: window::Settings {
            size: Size::new(900.0, 700.0),
            ..window::Settings::default()
        },
        default_font: DEFAULT_FONT,
        ..IcedSettings::default()
    })
}
