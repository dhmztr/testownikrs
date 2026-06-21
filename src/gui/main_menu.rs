use crate::gui::styles::{
    session_card_style, background_style, surface_style,
    header_style, progress_bar_bg_style, progress_bar_fill_style, Colors,
};
use crate::gui::Message;
use testownik_rs::models::{AppSettings, SessionMetadata, Theme};
use testownik_rs::persistence::Storage;
use iced::widget::{button, column, container, horizontal_space, row, scrollable, text, vertical_space, Column, Space};
use iced::{Alignment,Font ,Element, Length};
use iced::theme::{Button as ButtonTheme};
const EMOJI_FONT: Font = iced::Font::with_name("Segoe UI Emoji");
/// State for the main menu screen
#[derive(Debug, Clone)]
pub struct MainMenuState {
    pub sessions: Vec<SessionMetadata>,
    pub show_settings: bool,
}

impl MainMenuState {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            show_settings: false,
        }
    }

    pub fn refresh_sessions(&mut self, storage: &Storage) {
        self.sessions = storage.list_sessions().unwrap_or_default();
    }

    pub fn view(&self, settings: &AppSettings, storage: &Storage) -> Element<'_, Message> {
        let is_dark = settings.theme == Theme::Dark;
        let sessions = storage.list_sessions().unwrap_or_default();

        // Header section with gradient-style appearance
        let header = self.create_header(settings, is_dark);
        
        // Action buttons section
        let action_section = self.create_action_section(is_dark);
        
        // Sessions section
        let sessions_section = self.create_sessions_section(&sessions, is_dark);
        
        // Settings info footer
        let settings_footer = self.create_settings_footer(settings, is_dark);

        // Main layout with modern spacing
        let content = column![
            header,
            vertical_space().height(20),
            action_section,
            vertical_space().height(24),
            sessions_section,
            settings_footer,
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

    fn create_header(&self, settings: &AppSettings, is_dark: bool) -> Element<'_, Message> {
        let title = text("✨ Testownik").font(EMOJI_FONT)
        
            .size(36)
            .style(Colors::PRIMARY);

        let subtitle = text("Nowoczesna Aplikacja do Nauki")
            .size(16)
            .style(if is_dark { Colors::DARK_TEXT_SECONDARY } else { Colors::LIGHT_TEXT_SECONDARY });

        // Theme toggle button with icon
        let theme_icon = match settings.theme {
            Theme::Light => "🌙",
            Theme::Dark => "🌞",
        };
        
        let theme_button = button(
            text(theme_icon).font(EMOJI_FONT).size(22)
        )
        .on_press(Message::ToggleTheme)
        .padding([10, 20])
        .style(ButtonTheme::Secondary);

        let title_section = column![
            title,
            subtitle,
        ]
        .spacing(4);

        container(
            row![
                title_section,
                horizontal_space(),
                theme_button,
            ]
            .align_items(Alignment::Center)
            .padding([24, 28])
        )
        .style(header_style(is_dark))
        .width(Length::Fill)
        .into()
    }

    fn create_action_section(&self, _is_dark: bool) -> Element<'_, Message> {
        // Main action: Open file picker
        let open_file_btn = button(
            row![
                text("📂").font(EMOJI_FONT).size(20),
                horizontal_space().width(12),
                column![
                    text("Otwórz Plik Quiz").size(17),
                    text("Przeglądaj i wybierz plik .txt lub .json").size(12)
                        .style(Colors::TEXT_ON_PRIMARY),
                ]
                .spacing(2),
            ]
            .align_items(Alignment::Center)
            .padding([8, 0])
        )
        .on_press(Message::OpenFilePicker)
        .padding([16, 24])
        .width(Length::Fill)
        .style(ButtonTheme::Primary);

        // Quick start with example
        let example_btn = button(
            row![
                text("🚀").font(EMOJI_FONT).size(18),
                horizontal_space().width(10),
                text("Szybki Start z Przykładem").size(15),
            ]
            .align_items(Alignment::Center)
        )
        .on_press(Message::QuizFileSelected("example_questions.txt".to_string()))
        .padding([14, 20])
        .width(Length::Fill)
        .style(ButtonTheme::Secondary);

        container(
            column![
                open_file_btn,
                vertical_space().height(12),
                example_btn,
            ]
            .width(Length::Fill)
        )
        .padding([0, 28])
        .width(Length::Fill)
        .into()
    }

    fn create_sessions_section(&self, sessions: &[SessionMetadata], is_dark: bool) -> Element<'_, Message> {
        let section_title = row![
            text("📚").font(EMOJI_FONT).size(20),
            horizontal_space().width(10),
            text("Ostatnie Sesje").size(20)
                .style(if is_dark { Colors::DARK_TEXT } else { Colors::LIGHT_TEXT }),
        ]
        .align_items(Alignment::Center);

        let mut sessions_list = Column::new().spacing(12);

        if sessions.is_empty() {
            let empty_state = container(
                column![
                    text("📭").font(EMOJI_FONT).size(48),
                    vertical_space().height(12),
                    text("Brak zapisanych sesji").size(18)
                        .style(if is_dark { Colors::DARK_TEXT_SECONDARY } else { Colors::LIGHT_TEXT_SECONDARY }),
                    vertical_space().height(4),
                    text("Otwórz plik quiz, aby rozpocząć!").size(14)
                        .style(if is_dark { Colors::DARK_TEXT_SECONDARY } else { Colors::LIGHT_TEXT_SECONDARY }),
                ]
                .align_items(Alignment::Center)
                .padding(32)
            )
            .width(Length::Fill)
            .style(session_card_style(is_dark));
            
            sessions_list = sessions_list.push(empty_state);
        } else {
            for session in sessions {
                let session_card = Self::create_session_card(session, is_dark);
                sessions_list = sessions_list.push(session_card);
            }
        }

        let sessions_scroll = scrollable(sessions_list)
            .height(Length::Fill);

        container(
            column![
                section_title,
                vertical_space().height(16),
                sessions_scroll,
            ]
            .width(Length::Fill)
        )
        .padding([0, 28])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn create_session_card(session: &SessionMetadata, is_dark: bool) -> Element<'static, Message> {
        let name = text(&session.quiz_name)
            .size(17)
            .style(if is_dark { Colors::DARK_TEXT } else { Colors::LIGHT_TEXT });

        // Progress bar
        let progress_pct = session.progress_percentage as f32 / 100.0;
        let progress_bar = container(
            container(Space::new(Length::Fill, 6))
                .width(Length::FillPortion((progress_pct * 100.0) as u16))
                .style(progress_bar_fill_style(is_dark))
        )
        .width(Length::Fixed(180.0))
        .height(6)
        .style(progress_bar_bg_style(is_dark));

        let progress_text = text(format!(
            "{:.0}% • {}/{} ukończone",
            session.progress_percentage,
            session.total_count - session.active_count,
            session.total_count
        ))
        .size(13)
        .style(if is_dark { Colors::DARK_TEXT_SECONDARY } else { Colors::LIGHT_TEXT_SECONDARY });

        let updated = text(format!(
            "🕐 {}",
            session.updated_at.format("%b %d, %Y at %H:%M")
        ))
        .font(EMOJI_FONT)
        .size(12)
        .style(if is_dark { Colors::DARK_TEXT_SECONDARY } else { Colors::LIGHT_TEXT_SECONDARY });

        let continue_btn = button(
            row![
                text("▶").font(EMOJI_FONT).size(12),
                horizontal_space().width(6),
                text("Kontynuuj").size(14),
            ]
            .align_items(Alignment::Center)
        )
        .on_press(Message::LoadSession(session.id.clone()))
        .padding([10, 16])
        .style(ButtonTheme::Primary);

        let delete_btn = button(text("🗑").font(EMOJI_FONT).size(14))
            .on_press(Message::DeleteSession(session.id.clone()))
            .padding([10, 12])
            .style(ButtonTheme::Destructive);

        let info_col = column![
            name,
            vertical_space().height(8),
            row![
                progress_bar,
                horizontal_space().width(12),
                progress_text,
            ]
            .align_items(Alignment::Center),
            vertical_space().height(6),
            updated,
        ]
        .width(Length::Fill);

        let buttons_row = row![continue_btn, delete_btn]
            .spacing(8)
            .align_items(Alignment::Center);

        container(
            row![info_col, buttons_row]
                .spacing(20)
                .align_items(Alignment::Center)
                .padding([18, 20]),
        )
        .style(session_card_style(is_dark))
        .width(Length::Fill)
        .into()
    }

    fn create_settings_footer(&self, settings: &AppSettings, is_dark: bool) -> Element<'_, Message> {
        if self.show_settings {
            return self.create_settings_panel(settings, is_dark);
        }

        let settings_text = text(format!(
            "⚙️ Powtórzenia: {} początkowe • -{} za poprawną • +{} za błędną • {} maks.",
            settings.repetition_settings.initial_repetitions,
            settings.repetition_settings.correct_decrease,
            settings.repetition_settings.incorrect_increase,
            settings.repetition_settings.max_repetitions
        ))
        .font(EMOJI_FONT)
        .size(12)
        .style(if is_dark { Colors::DARK_TEXT_SECONDARY } else { Colors::LIGHT_TEXT_SECONDARY });

        let edit_btn = button(text("Edytuj").size(12))
            .on_press(Message::ToggleSettingsPanel)
            .padding([6, 12])
            .style(ButtonTheme::Secondary);

        container(
            row![
                horizontal_space(),
                settings_text,
                horizontal_space().width(12),
                edit_btn,
                horizontal_space(),
            ]
            .align_items(Alignment::Center)
        )
        .padding([16, 28])
        .width(Length::Fill)
        .into()
    }

    fn create_settings_panel(&self, settings: &AppSettings, is_dark: bool) -> Element<'_, Message> {
        let rs = &settings.repetition_settings;

        let title = text("⚙️ Ustawienia powtórzeń")
            .font(EMOJI_FONT)
            .size(18)
            .style(if is_dark { Colors::DARK_TEXT } else { Colors::LIGHT_TEXT });

        let close_btn = button(text("Zamknij").size(13))
            .on_press(Message::ToggleSettingsPanel)
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
            vertical_space().height(12),
            initial,
            vertical_space().height(8),
            correct,
            vertical_space().height(8),
            incorrect,
            vertical_space().height(8),
            max,
        ]
        .width(Length::Fill);

        container(container(panel).padding(20).style(session_card_style(is_dark)).width(Length::Fill))
            .padding([16, 28])
            .width(Length::Fill)
            .into()
    }
}

fn stepper_row<'a>(
    label: &'a str,
    hint: &'a str,
    value: i32,
    on_minus: Message,
    on_plus: Message,
    is_dark: bool,
) -> Element<'a, Message> {
    let label_text = text(label)
        .size(14)
        .style(if is_dark { Colors::DARK_TEXT } else { Colors::LIGHT_TEXT });
    let hint_text = text(hint)
        .size(11)
        .style(if is_dark { Colors::DARK_TEXT_SECONDARY } else { Colors::LIGHT_TEXT_SECONDARY });

    let minus = button(text("−").size(18))
        .on_press(on_minus)
        .padding([4, 14])
        .style(ButtonTheme::Secondary);
    let plus = button(text("+").size(18))
        .on_press(on_plus)
        .padding([4, 14])
        .style(ButtonTheme::Secondary);
    let value_text = text(format!("{}", value))
        .size(16)
        .style(Colors::PRIMARY);

    let stepper = row![minus, horizontal_space().width(12), value_text, horizontal_space().width(12), plus]
        .align_items(Alignment::Center);

    row![
        column![label_text, hint_text].spacing(2).width(Length::Fill),
        stepper,
    ]
    .align_items(Alignment::Center)
    .into()
}

impl Default for MainMenuState {
    fn default() -> Self {
        Self::new()
    }
}
