use crate::gui::styles::{
    success_container_style, error_container_style, answer_option_style,
    correct_answer_style, incorrect_answer_style, background_style, surface_style,
    header_style, progress_bar_bg_style, progress_bar_fill_style, badge_style,
    letter_badge_style, session_card_style, Colors,
};
use crate::gui::Message;
use crate::gui::main_menu::stepper_row;
use testownik_rs::models::{AppSettings, QuestionType, QuizSession, SessionQuestion, Theme};
use testownik_rs::types::{AnswerContent, AnswerType, ContentPart, QuestionContent};
use testownik_rs::utils;
use iced::widget::{button, column, container, horizontal_space, image as img_widget, pick_list, row, scrollable, text, vertical_space, Column, Space};
use iced::{Alignment, ContentFit, Font, Element, Length};
use iced::theme::{Button as ButtonTheme};
use std::path::{Path, PathBuf};

const EMOJI_FONT: Font = Font::with_name("Segoe UI Emoji");

/// Default option index for unselected dropdowns in Select-type questions
const DEFAULT_OPTION_INDEX: usize = 0;

/// State for the quiz screen
#[derive(Debug, Clone)]
pub struct QuizState {
    pub session: QuizSession,
    pub selected_answers: Vec<usize>,
    pub answer_submitted: bool,
    pub is_correct: Option<bool>,
    pub randomized_answer_indices: Vec<usize>,
    pub image_base_dir: PathBuf,
    // ponytail: defer session.update_question until next_question so current_question() does not jump after submit when reps hit 0
    pub pending_submission: Option<(String, bool)>,
    pub show_settings: bool,
}

impl QuizState {
    pub fn new(session: QuizSession, image_base_dir: PathBuf) -> Self {
        // Randomize answer order for the first question
        let answer_count = session
            .current_question()
            .map(|q| q.question.answers.len())
            .unwrap_or(0);

        let randomized_indices = utils::randomize_indices(answer_count);

        Self {
            session,
            selected_answers: Vec::new(),
            answer_submitted: false,
            is_correct: None,
            randomized_answer_indices: randomized_indices,
            image_base_dir,
            pending_submission: None,
            show_settings: false,
        }
    }

    pub fn toggle_answer(&mut self, index: usize) {
        if self.answer_submitted {
            return; // Don't allow changes after submission
        }

        if let Some(question) = self.session.current_question() {
            // Check how many correct answers this question has
            let correct_count = question.question.answers.iter()
                .filter(|a| a.is_correct == Some(true))
                .count();
            
            match question.question.question_type {
                QuestionType::Single => {
                    if correct_count > 1 {
                        // Multiple choice - allow multiple selections
                        if let Some(pos) = self.selected_answers.iter().position(|&x| x == index) {
                            self.selected_answers.remove(pos);
                        } else {
                            self.selected_answers.push(index);
                        }
                    } else {
                        // Single choice - only one selection
                        if self.selected_answers.contains(&index) {
                            self.selected_answers.clear();
                        } else {
                            self.selected_answers.clear();
                            self.selected_answers.push(index);
                        }
                    }
                }
                QuestionType::Select => {
                    // Toggle selection for select questions
                    if let Some(pos) = self.selected_answers.iter().position(|&x| x == index) {
                        self.selected_answers.remove(pos);
                    } else {
                        self.selected_answers.push(index);
                    }
                }
            }
        }
    }

    pub fn select_dropdown_option(&mut self, dropdown_idx: usize, option_idx: usize) {
        if self.answer_submitted {
            return; // Don't allow changes after submission
        }

        // Ensure we have enough space in selected_answers
        while self.selected_answers.len() <= dropdown_idx {
            self.selected_answers.push(DEFAULT_OPTION_INDEX);
        }
        
        // Set the selected option for this dropdown
        self.selected_answers[dropdown_idx] = option_idx;
    }

    pub fn submit_answer(&mut self) {
        // Get the question tag first to avoid borrowing issues
        let (question_tag, is_correct) = if let Some(question) = self.session.current_question() {
            let is_correct = question.check_answer(&self.selected_answers);
            (question.question.tag.clone(), is_correct)
        } else {
            return;
        };

        self.is_correct = Some(is_correct);
        self.answer_submitted = true;

        // ponytail: defer session update until advance so current_question stays put while feedback shows
        self.pending_submission = Some((question_tag, is_correct));
    }

    pub fn next_question(&mut self) {
        if let Some((tag, is_correct)) = self.pending_submission.take() {
            self.session.update_question(&tag, is_correct);
        }
        self.session.next_question();
        self.selected_answers.clear();
        self.answer_submitted = false;
        self.is_correct = None;

        // Randomize answers for the new question
        let answer_count = self
            .session
            .current_question()
            .map(|q| q.question.answers.len())
            .unwrap_or(0);
        self.randomized_answer_indices = utils::randomize_indices(answer_count);
    }

    pub fn view(&self, settings: &AppSettings) -> Element<'_, Message> {
        let is_dark = settings.theme == Theme::Dark;
        
        if self.session.is_complete() {
            return self.view_completion(is_dark);
        }

        let Some(current_q) = self.session.current_question() else {
            return container(
                column![
                    text("📭").font(EMOJI_FONT).size(48),
                    vertical_space().height(12),
                    text("Brak dostępnych pytań").size(20)
                        .style(if is_dark { Colors::DARK_TEXT } else { Colors::LIGHT_TEXT }),
                ]
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(background_style(is_dark))
            .into();
        };

        // Header with progress
        let header = self.create_header(current_q, is_dark);

        // Question content
        let question_card = self.create_question_card(current_q, is_dark);

        // Feedback message
        let feedback = self.create_feedback(is_dark);

        // Answers section
        let answers_section = self.create_answers_section(current_q, is_dark);

        // Action buttons
        let action_buttons = self.create_action_buttons(is_dark);

        // Main content
        let content = column![
            header,
            vertical_space().height(16),
            question_card,
            answers_section,
            feedback,
            action_buttons,
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
            .style(background_style(is_dark))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(0)
            .into()
    }

    fn create_header(&self, current_q: &SessionQuestion, is_dark: bool) -> Element<'_, Message> {
        let active_questions = self.session.active_questions();
        let progress_pct = self.session.progress_percentage() as f32 / 100.0;
        
        // Calculate total remaining answers (sum of all repetitions remaining)
        let total_remaining: i32 = active_questions.iter()
            .map(|q| q.repetitions_remaining)
            .sum();

        // Polish pluralization: 1 = "odpowiedź", else "odpowiedzi"
        let answer_word = if total_remaining == 1 { "odpowiedź" } else { "odpowiedzi" };

        // Question counter - shows total remaining answers
        let question_counter = text(format!(
            "Pytanie {} z {} (pozostało {} {})",
            self.session.current_index + 1,
            active_questions.len(),
            total_remaining,
            answer_word
        ))
        .size(14)
        .style(if is_dark { Colors::DARK_TEXT_SECONDARY } else { Colors::LIGHT_TEXT_SECONDARY });

        // Repetitions badge - shows how many times THIS question still needs to be answered
        let reps_badge = container(
            text(format!("🔄 {} powt. dla tego pytania", current_q.repetitions_remaining)).font(EMOJI_FONT).size(12)
        )
        .padding([4, 10])
        .style(badge_style(is_dark));

        // Progress percentage + finished/total questions
        let total_q = self.session.questions.len();
        let finished_q = self.session.questions.iter().filter(|q| q.repetitions_remaining == 0).count();
        let progress_text = text(format!(
            "{}/{} ukończone • {:.0}%",
            finished_q,
            total_q,
            self.session.progress_percentage()
        ))
            .size(16)
            .style(Colors::PRIMARY);

        // Progress bar
        let progress_bar = container(
            container(Space::new(Length::Fill, 8))
                .width(Length::FillPortion((progress_pct * 100.0) as u16))
                .style(progress_bar_fill_style(is_dark))
        )
        .width(Length::Fill)
        .height(8)
        .style(progress_bar_bg_style(is_dark));

        container(
            column![
                row![
                    question_counter,
                    horizontal_space(),
                    reps_badge,
                    horizontal_space().width(12),
                    progress_text,
                ]
                .align_items(Alignment::Center),
                vertical_space().height(12),
                progress_bar,
            ]
            .padding([20, 28])
        )
        .style(header_style(is_dark))
        .width(Length::Fill)
        .into()
    }

    fn create_question_card(&self, current_q: &SessionQuestion, is_dark: bool) -> Element<'_, Message> {
        let text_color = if is_dark { Colors::DARK_TEXT } else { Colors::LIGHT_TEXT };

        let question_body: Element<'_, Message> = match &current_q.question.content {
            QuestionContent::Text(t) => {
                let mut col = Column::new().spacing(12);
                for raw_line in t.split('\n') {
                    let line = raw_line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    col = push_text_with_images(&self.image_base_dir, col, line, text_color);
                }
                col.into()
            }
            QuestionContent::Image(path) => {
                image_element(&self.image_base_dir, path.trim())
            }
            QuestionContent::Select(parts) => {
                let mut content = String::new();
                for part in parts {
                    match part {
                        ContentPart::Text(t) => content.push_str(t),
                        ContentPart::SelectPlaceholder {
                            select_id,
                            visible_content,
                        } => {
                            if visible_content.is_empty() {
                                content.push_str(&format!("[Wybór {}]", select_id + 1));
                            } else {
                                content.push_str(visible_content);
                            }
                        }
                    }
                }
                text(content)
                    .size(20)
                    .style(if is_dark { Colors::DARK_TEXT } else { Colors::LIGHT_TEXT })
                    .into()
            }
        };

        container(
            container(
                column![
                    text("❓").font(EMOJI_FONT).size(28),
                    vertical_space().height(12),
                    question_body,
                ]
            )
            .padding([24, 28])
            .width(Length::Fill)
            .style(surface_style(is_dark))
        )
        .padding([0, 28])
        .width(Length::Fill)
        .into()
    }

    fn create_feedback(&self, is_dark: bool) -> Element<'_, Message> {
        if let Some(is_correct) = self.is_correct {
            let (icon, message) = if is_correct {
                ("✅", "Poprawnie! Świetna robota!")
            } else {
                ("❌", "Niepoprawnie. Prawidłowa odpowiedź jest podświetlona powyżej.")
            };

            let inner_container = if is_correct {
                container(
                    row![
                        text(icon).font(EMOJI_FONT).size(24),
                        horizontal_space().width(12),
                        text(message).size(16),
                    ]
                    .align_items(Alignment::Center)
                    .padding([14, 20])
                )
                .width(Length::Fill)
                .style(success_container_style(is_dark))
            } else {
                container(
                    row![
                        text(icon).font(EMOJI_FONT).size(24),
                        horizontal_space().width(12),
                        text(message).size(16),
                    ]
                    .align_items(Alignment::Center)
                    .padding([14, 20])
                )
                .width(Length::Fill)
                .style(error_container_style(is_dark))
            };

            container(inner_container)
            .padding([16, 28])
            .width(Length::Fill)
            .into()
        } else {
            container(vertical_space().height(8))
                .width(Length::Fill)
                .into()
        }
    }

    fn create_answers_section(&self, question: &SessionQuestion, is_dark: bool) -> Element<'_, Message> {
        let mut answers_col = Column::new().spacing(10);

        match question.question.question_type {
            QuestionType::Single => {
                for &original_idx in &self.randomized_answer_indices {
                    if let Some(answer) = question.question.answers.get(original_idx) {
                        let answer_text = match &answer.content {
                            AnswerContent::Single { content } => content.clone(),
                            _ => String::new(),
                        };

                        let is_selected = self.selected_answers.contains(&original_idx);
                        let is_correct_answer = answer.is_correct.unwrap_or(false);

                        // Create the answer option
                        let option_letter = (b'A' + original_idx as u8) as char;
                        
                        let letter_container = container(
                            text(format!("{}", option_letter)).size(14)
                        )
                        .padding([6, 10])
                        .style(letter_badge_style(is_dark, is_selected));

                        let answer_content: Element<'_, Message> = match answer.answer_type {
                            AnswerType::Image => img_widget(
                                resolve_image_path(&self.image_base_dir, answer_text.trim())
                            )
                            .width(Length::Fixed(200.0))
                            .height(Length::Fixed(140.0))
                            .content_fit(ContentFit::Contain)
                            .into(),
                            AnswerType::Text => text(&answer_text)
                                .size(16)
                                .style(if is_dark { Colors::DARK_TEXT } else { Colors::LIGHT_TEXT })
                                .into(),
                        };

                        let answer_row = row![
                            letter_container,
                            horizontal_space().width(14),
                            answer_content,
                        ]
                        .align_items(Alignment::Center);

                        // Build the styled container based on submission state
                        let styled_container = if self.answer_submitted && is_correct_answer {
                            container(answer_row)
                                .padding([14, 18])
                                .width(Length::Fill)
                                .style(correct_answer_style(is_dark))
                        } else if self.answer_submitted && is_selected && !is_correct_answer {
                            container(answer_row)
                                .padding([14, 18])
                                .width(Length::Fill)
                                .style(incorrect_answer_style(is_dark))
                        } else if is_selected {
                            container(answer_row)
                                .padding([14, 18])
                                .width(Length::Fill)
                                .style(answer_option_style(is_dark, true))
                        } else {
                            container(answer_row)
                                .padding([14, 18])
                                .width(Length::Fill)
                                .style(answer_option_style(is_dark, false))
                        };

                        let answer_container = button(styled_container)
                        .on_press(Message::SelectAnswer(original_idx))
                        .padding(0)
                        .width(Length::Fill)
                        .style(ButtonTheme::Text);

                        answers_col = answers_col.push(answer_container);
                    }
                }
            }
            QuestionType::Select => {
                // For each answer (dropdown), create a pick list
                for (answer_idx, answer) in question.question.answers.iter().enumerate() {
                    if let Some(options) = &answer.options {
                        let label = format!("Wybór {}:", answer_idx + 1);
                        
                        // Create list of option contents for the dropdown
                        let options_list: Vec<String> = options.iter()
                            .map(|opt| opt.content.clone())
                            .collect();
                        
                        // Get currently selected option index
                        let selected_idx = self.selected_answers.get(answer_idx).cloned();
                        let selected_option = selected_idx.and_then(|idx| options_list.get(idx).cloned());
                        
                        // Create the pick list
                        let pick = pick_list(
                            options_list.clone(),
                            selected_option,
                            move |selected_text| {
                                // Find the index of the selected option
                                let opt_idx = options_list.iter()
                                    .position(|opt| opt == &selected_text)
                                    .unwrap_or(DEFAULT_OPTION_INDEX);
                                Message::SelectOption(answer_idx, opt_idx)
                            }
                        );
                        
                        // Style the dropdown based on submission state
                        let dropdown_container = if self.answer_submitted {
                            // Show correct/incorrect styling after submission
                            let is_correct_choice = selected_idx == answer.correct_option_id;
                            
                            let feedback_icon = if is_correct_choice { "✓" } else { "✗" };
                            let feedback_color = if is_correct_choice { 
                                Colors::SUCCESS 
                            } else { 
                                Colors::ERROR 
                            };
                            
                            let feedback_text = if is_correct_choice { 
                                "Poprawnie".to_string()
                            } else { 
                                format!("Niepoprawnie (poprawna: {})", 
                                    options.get(answer.correct_option_id.unwrap_or(DEFAULT_OPTION_INDEX))
                                        .map(|o| o.content.as_str())
                                        .unwrap_or("?"))
                            };
                            
                            column![
                                text(label.clone()).size(14)
                                    .style(if is_dark { Colors::DARK_TEXT } else { Colors::LIGHT_TEXT }),
                                pick,
                                text(format!("{} {}", feedback_icon, feedback_text))
                                    .size(12).style(feedback_color)
                            ].spacing(6)
                        } else {
                            column![
                                text(label).size(14)
                                    .style(if is_dark { Colors::DARK_TEXT } else { Colors::LIGHT_TEXT }),
                                pick,
                            ].spacing(6)
                        };
                        
                        answers_col = answers_col.push(
                            container(dropdown_container)
                                .padding([12, 16])
                                .width(Length::Fill)
                                .style(surface_style(is_dark))
                        );
                    }
                }
            }
        }

        container(
            scrollable(answers_col)
        )
        .padding([0, 28])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn create_action_buttons(&self, is_dark: bool) -> Element<'_, Message> {
        let (primary_btn_text, primary_btn_msg) = if self.answer_submitted {
            ("Następne Pytanie →", Message::NextQuestion)
        } else {
            ("Sprawdź Odpowiedź ✓", Message::SubmitAnswer)
        };

        let primary_btn = button(
            text(primary_btn_text).size(16)
        )
        .on_press(primary_btn_msg)
        .padding([14, 28])
        .style(ButtonTheme::Primary);

        let exit_btn = button(
            row![
                text("💾").font(EMOJI_FONT).size(14),
                horizontal_space().width(8),
                text("Zapisz i Wyjdź").size(14),
            ]
            .align_items(Alignment::Center)
        )
        .on_press(Message::SaveAndExit)
        .padding([14, 20])
        .style(ButtonTheme::Secondary);

        let settings_btn = button(
            row![
                text("⚙️").font(EMOJI_FONT).size(14),
                horizontal_space().width(6),
                text("Ustawienia").size(14),
            ]
            .align_items(Alignment::Center)
        )
        .on_press(Message::ToggleQuizSettingsPanel)
        .padding([14, 16])
        .style(ButtonTheme::Secondary);

        let copy_btn = button(
            row![
                text("📋").font(EMOJI_FONT).size(14),
                horizontal_space().width(6),
                text("Kopiuj pytanie").size(14),
            ]
            .align_items(Alignment::Center)
        )
        .on_press(Message::CopyText(self.question_copy_text()))
        .padding([14, 16])
        .style(ButtonTheme::Secondary);

        // Keyboard shortcuts help text
        let shortcuts_text = text("⌨️ 1-9: Wybierz • Spacja: Zatwierdź/Dalej").font(EMOJI_FONT)
            .size(12)
            .style(if is_dark { Colors::DARK_TEXT_SECONDARY } else { Colors::LIGHT_TEXT_SECONDARY });

        let settings_panel: Element<'_, Message> = if self.show_settings {
            self.create_settings_panel(is_dark)
        } else {
            vertical_space().height(0).into()
        };

        column![
            settings_panel,
            container(
                row![
                    exit_btn,
                    horizontal_space().width(8),
                    settings_btn,
                    horizontal_space().width(8),
                    copy_btn,
                    horizontal_space(),
                    primary_btn,
                ]
                .padding([20, 28])
                .align_items(Alignment::Center)
            )
            .width(Length::Fill),
            container(shortcuts_text)
                .padding([0, 28, 12, 28])
                .width(Length::Fill)
                .center_x()
        ]
        .width(Length::Fill)
        .into()
    }

    fn question_copy_text(&self) -> String {
        let Some(q) = self.session.current_question() else {
            return String::new();
        };
        let mut out = String::new();
        match &q.question.content {
            QuestionContent::Text(t) => out.push_str(t),
            QuestionContent::Image(p) => out.push_str(p),
            QuestionContent::Select(parts) => {
                for part in parts {
                    match part {
                        ContentPart::Text(t) => out.push_str(t),
                        ContentPart::SelectPlaceholder { select_id, visible_content } => {
                            if visible_content.is_empty() {
                                out.push_str(&format!("[Wybór {}]", select_id + 1));
                            } else {
                                out.push_str(visible_content);
                            }
                        }
                    }
                }
            }
        }
        // Append answers so user can paste full question
        out.push_str("\n\n");
        for (i, a) in q.question.answers.iter().enumerate() {
            let prefix = format!("{}. ", i + 1);
            match &a.content {
                AnswerContent::Single { content } => out.push_str(&format!("{prefix}{content}\n")),
                AnswerContent::Select { options, .. } => {
                    out.push_str(&prefix);
                    for opt in options {
                        out.push_str(&format!("[{}] ", opt.content));
                    }
                    out.push('\n');
                }
            }
        }
        out
    }

    fn create_settings_panel(&self, is_dark: bool) -> Element<'_, Message> {
        let rs = &self.session.settings;
        let title = text("⚙️ Ustawienia powtórzeń (ten quiz)")
            .font(EMOJI_FONT)
            .size(16)
            .style(if is_dark { Colors::DARK_TEXT } else { Colors::LIGHT_TEXT });

        let close_btn = button(text("Zamknij").size(13))
            .on_press(Message::ToggleQuizSettingsPanel)
            .padding([8, 14])
            .style(ButtonTheme::Secondary);

        let reset_btn = button(text("Reset").size(13))
            .on_press(Message::ResetRepetitionSettings)
            .padding([8, 14])
            .style(ButtonTheme::Destructive);

        let header_row = row![title, horizontal_space(), reset_btn, horizontal_space().width(8), close_btn]
            .align_items(Alignment::Center);

        let initial = stepper_row(
            "Początkowe powtórzenia",
            "Ile razy każde pytanie pojawi się na początku",
            rs.initial_repetitions,
            Message::SetInitialRepetitions(rs.initial_repetitions - 1),
            Message::SetInitialRepetitions(rs.initial_repetitions + 1),
            is_dark,
        );
        let correct = stepper_row(
            "Za poprawną odpowiedź (−)",
            "O ile zmniejszyć licznik po poprawnej odpowiedzi",
            rs.correct_decrease,
            Message::SetCorrectDecrease(rs.correct_decrease - 1),
            Message::SetCorrectDecrease(rs.correct_decrease + 1),
            is_dark,
        );
        let incorrect = stepper_row(
            "Kara za błędną odpowiedź (+)",
            "O ile zwiększyć licznik po błędnej odpowiedzi",
            rs.incorrect_increase,
            Message::SetIncorrectIncrease(rs.incorrect_increase - 1),
            Message::SetIncorrectIncrease(rs.incorrect_increase + 1),
            is_dark,
        );
        let max = stepper_row(
            "Maksymalna liczba powtórzeń",
            "Górny limit licznika powtórzeń",
            rs.max_repetitions,
            Message::SetMaxRepetitions(rs.max_repetitions - 1),
            Message::SetMaxRepetitions(rs.max_repetitions + 1),
            is_dark,
        );

        let panel = column![
            header_row,
            vertical_space().height(10),
            initial,
            vertical_space().height(6),
            correct,
            vertical_space().height(6),
            incorrect,
            vertical_space().height(6),
            max,
        ]
        .width(Length::Fill);

        container(container(panel).padding(16).style(session_card_style(is_dark)).width(Length::Fill))
            .padding([8, 28])
            .width(Length::Fill)
            .into()
    }

    fn view_completion(&self, is_dark: bool) -> Element<'_, Message> {
        let completion_card = container(
            column![
                text("🎉").font(EMOJI_FONT).size(64),
                vertical_space().height(20),
                text("Quiz Ukończony!")
                    .size(32)
                    .style(Colors::SUCCESS),
                vertical_space().height(12),
                text("Gratulacje! Ukończyłeś wszystkie pytania w")
                    .size(16)
                    .style(if is_dark { Colors::DARK_TEXT_SECONDARY } else { Colors::LIGHT_TEXT_SECONDARY }),
                text(&self.session.quiz_name)
                    .size(18)
                    .style(if is_dark { Colors::DARK_TEXT } else { Colors::LIGHT_TEXT }),
                vertical_space().height(24),
                container(
                    column![
                        text("📚").font(EMOJI_FONT).size(24),
                        text(format!("{}", self.session.questions.len())).size(28)
                            .style(Colors::PRIMARY),
                        text("Pytań").size(12)
                            .style(if is_dark { Colors::DARK_TEXT_SECONDARY } else { Colors::LIGHT_TEXT_SECONDARY }),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(4)
                )
                .padding(20),
                vertical_space().height(24),
                button(
                    row![
                        text("🏠").font(EMOJI_FONT).size(16),
                        horizontal_space().width(10),
                        text("Powrót do Menu").size(16),
                    ]
                    .align_items(Alignment::Center)
                )
                .on_press(Message::BackToMenu)
                .padding([14, 28])
                .style(ButtonTheme::Primary),
            ]
            .spacing(8)
            .padding(40)
            .align_items(Alignment::Center)
        )
        .style(surface_style(is_dark));

        container(completion_card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(background_style(is_dark))
            .into()
    }
}

/// Scans `line` for `[img]...[/img]` markers and appends the text segments and
/// image widgets to `col` in source order, so images always render directly
/// under the surrounding question text (also handles tags inline with text).
fn push_text_with_images<'a>(
    base_dir: &Path,
    mut col: Column<'a, Message>,
    line: &str,
    text_color: iced::Color,
) -> Column<'a, Message> {
    let mut rest = line;
    loop {
        let Some(open) = rest.find("[img]") else {
            let tail = rest.trim();
            if !tail.is_empty() {
                col = col.push(text(tail.to_string()).size(20).style(text_color));
            }
            return col;
        };

        let pre = rest[..open].trim();
        if !pre.is_empty() {
            col = col.push(text(pre.to_string()).size(20).style(text_color));
        }

        let after_open = &rest[open + "[img]".len()..];
        let Some(close) = after_open.find("[/img]") else {
            let tail = rest[open..].trim();
            if !tail.is_empty() {
                col = col.push(text(tail.to_string()).size(20).style(text_color));
            }
            return col;
        };

        let img_path = after_open[..close].trim();
        if !img_path.is_empty() {
            col = col.push(image_element(base_dir, img_path));
        }
        rest = &after_open[close + "[/img]".len()..];
    }
}

/// Wraps the image in a centered, height-constrained container so it scales
/// sensibly and stays visible inside the question card.
fn image_element(base_dir: &Path, img_path: &str) -> Element<'static, Message> {
    container(
        img_widget(resolve_image_path(base_dir, img_path))
            .width(Length::Fill)
            .height(Length::Fixed(280.0))
            .content_fit(ContentFit::Contain),
    )
    .width(Length::Fill)
    .center_x()
    .into()
}

/// Resolves an image filename to a full path.
/// Tries `base_dir/filename` first; if not found, searches one level of subdirectories.
fn resolve_image_path(base_dir: &Path, filename: &str) -> PathBuf {
    let direct = base_dir.join(filename);
    if direct.exists() {
        return direct;
    }
    if let Ok(entries) = std::fs::read_dir(base_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let candidate = entry.path().join(filename);
                if candidate.exists() {
                    return candidate;
                }
            }
        }
    }
    direct
}
