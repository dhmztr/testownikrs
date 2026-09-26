use crate::gui::Message;
use crate::gui::styles::{surface_style, background_style, Colors};
use testownik_rs::models::{AppSettings, QuizSession, Theme};
use iced::widget::{button, column, container, text, vertical_space};
use iced::{Alignment, Font, Element, Length};
use iced::theme::Button as ButtonTheme;

const EMOJI_FONT: Font = Font::with_name("Segoe UI Emoji");

#[derive(Debug, Clone)]
pub struct CompletionState {
    pub session: QuizSession,
}

impl CompletionState {
    pub fn new(session: QuizSession) -> Self {
        Self { session }
    }

    pub fn view(&self, settings: &AppSettings) -> Element<'_, Message> {
        let is_dark = settings.theme == Theme::Dark;
        
        let total_questions = self.session.questions.len();
        let completed_questions = self.session.questions.iter()
            .filter(|q| q.repetitions_remaining == 0)
            .count();
        
        let total_correct = self.session.questions.iter()
            .map(|q| q.correct_count as usize)
            .sum::<usize>();
        
        let total_incorrect = self.session.questions.iter()
            .map(|q| q.incorrect_count as usize)
            .sum::<usize>();
        
        let accuracy = if total_correct + total_incorrect > 0 {
            total_correct as f32 / (total_correct + total_incorrect) as f32 * 100.0
        } else {
            0.0
        };

        let content = column![
            vertical_space().height(60),
            text("🎉 Quiz Ukończony!").font(EMOJI_FONT).size(32),
            vertical_space().height(32),
            container(
                column![
                    text(format!("Quiz: {}", self.session.quiz_name)).size(18),
                    vertical_space().height(16),
                    text(format!("Opanowane pytania: {} / {}", completed_questions, total_questions)).size(16),
                    text(format!("Suma poprawnych odpowiedzi: {}", total_correct)).size(16),
                    text(format!("Suma błędnych odpowiedzi: {}", total_incorrect)).size(16),
                    text(format!("Dokładność: {:.1}%", accuracy)).size(16),
                    text(format!("🔥 Najdłuższa seria poprawnych: {}", self.session.best_streak))
                        .font(EMOJI_FONT)
                        .size(16)
                        .style(Colors::STREAK_HOT),
                ]
                .spacing(8)
                .padding(24)
            )
            .style(surface_style(is_dark))
            .width(Length::Fixed(500.0)),
            vertical_space().height(32),
            button(text("Powrót do Menu Głównego").size(16))
                .on_press(Message::CompleteQuiz)
                .padding([14, 28])
                .style(ButtonTheme::Primary),
        ]
        .align_items(Alignment::Center)
        .width(Length::Fill);

        container(content)
            .style(background_style(is_dark))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(0)
            .center_x()
            .center_y()
            .into()
    }
}
