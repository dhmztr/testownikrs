// Modern styling module for the Testownik-rs application
// Uses iced's built-in theming system with customizations

use iced::widget::container;
use iced::{Border, Color, Background};

/// Modern color palette
pub struct Colors;

impl Colors {
    // Primary colors - deep blue/purple gradient theme
    pub const PRIMARY: Color = Color::from_rgb(0.29, 0.47, 0.86);      // #4A78DB
    pub const PRIMARY_LIGHT: Color = Color::from_rgb(0.40, 0.60, 0.92); // #6699EB
    pub const PRIMARY_DARK: Color = Color::from_rgb(0.20, 0.35, 0.70);  // #3359B3
    
    // Accent colors
    pub const ACCENT: Color = Color::from_rgb(0.58, 0.33, 0.87);       // #9454DE - Purple accent
    pub const ACCENT_LIGHT: Color = Color::from_rgb(0.70, 0.50, 0.92); // #B380EB
    
    // Success/Error colors
    pub const SUCCESS: Color = Color::from_rgb(0.18, 0.75, 0.48);      // #2EBF7A
    pub const SUCCESS_LIGHT: Color = Color::from_rgba(0.18, 0.75, 0.48, 0.15);
    pub const ERROR: Color = Color::from_rgb(0.89, 0.28, 0.35);        // #E34759
    pub const ERROR_LIGHT: Color = Color::from_rgba(0.89, 0.28, 0.35, 0.15);
    
    // Warning/Info colors
    pub const WARNING: Color = Color::from_rgb(0.95, 0.70, 0.23);      // #F2B33B
    pub const INFO: Color = Color::from_rgb(0.18, 0.67, 0.87);         // #2EABDE

    // Hot streak colors (orange -> red -> magenta as the streak grows)
    pub const STREAK_HOT: Color = Color::from_rgb(0.98, 0.55, 0.13);    // #FA8C21
    pub const STREAK_HOTTER: Color = Color::from_rgb(0.95, 0.33, 0.16); // #F25429
    pub const STREAK_ON_FIRE: Color = Color::from_rgb(0.90, 0.20, 0.45); // #E63373
    
    // Background colors for dark mode
    pub const DARK_BG: Color = Color::from_rgb(0.08, 0.09, 0.12);      // #14171F
    pub const DARK_SURFACE: Color = Color::from_rgb(0.12, 0.14, 0.18); // #1F232E
    pub const DARK_CARD: Color = Color::from_rgb(0.16, 0.18, 0.23);    // #292E3B
    pub const DARK_BORDER: Color = Color::from_rgb(0.22, 0.25, 0.32);  // #383F51
    
    // Background colors for light mode
    pub const LIGHT_BG: Color = Color::from_rgb(0.96, 0.97, 0.98);     // #F5F7FA
    pub const LIGHT_SURFACE: Color = Color::from_rgb(1.0, 1.0, 1.0);   // #FFFFFF
    pub const LIGHT_CARD: Color = Color::from_rgb(0.98, 0.98, 0.99);   // #FAFAFC
    pub const LIGHT_BORDER: Color = Color::from_rgb(0.88, 0.90, 0.92); // #E0E5EB
    
    // Text colors
    pub const DARK_TEXT: Color = Color::from_rgb(0.93, 0.94, 0.96);    // #EDEFF4
    pub const DARK_TEXT_SECONDARY: Color = Color::from_rgb(0.65, 0.68, 0.75); // #A6ADBF
    pub const LIGHT_TEXT: Color = Color::from_rgb(0.12, 0.14, 0.20);   // #1F2433
    pub const LIGHT_TEXT_SECONDARY: Color = Color::from_rgb(0.45, 0.48, 0.55); // #737A8C
    
    // Special text overlay colors for buttons
    pub const TEXT_ON_PRIMARY: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.7); // Semi-transparent white for subtitles on primary buttons
}

/// Blends a translucent tint over the window background into an opaque
/// color, so panels stay readable when the hot streak fire burns behind them
pub fn opaque_over_bg(tint: Color, is_dark: bool) -> Color {
    let bg = if is_dark { Colors::DARK_BG } else { Colors::LIGHT_BG };
    Color::from_rgb(
        bg.r + (tint.r - bg.r) * tint.a,
        bg.g + (tint.g - bg.g) * tint.a,
        bg.b + (tint.b - bg.b) * tint.a,
    )
}

/// Style for a modern card container
pub fn card_style(is_dark: bool) -> impl Fn(&iced::Theme) -> container::Appearance {
    move |_theme: &iced::Theme| {
        container::Appearance {
            background: Some(Background::Color(
                if is_dark { Colors::DARK_CARD } else { Colors::LIGHT_CARD }
            )),
            border: Border {
                color: if is_dark { Colors::DARK_BORDER } else { Colors::LIGHT_BORDER },
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    }
}

/// Style for the main background
pub fn background_style(is_dark: bool) -> impl Fn(&iced::Theme) -> container::Appearance {
    move |_theme: &iced::Theme| {
        container::Appearance {
            background: Some(Background::Color(
                if is_dark { Colors::DARK_BG } else { Colors::LIGHT_BG }
            )),
            ..Default::default()
        }
    }
}

/// Style for surface containers
pub fn surface_style(is_dark: bool) -> impl Fn(&iced::Theme) -> container::Appearance {
    move |_theme: &iced::Theme| {
        container::Appearance {
            background: Some(Background::Color(
                if is_dark { Colors::DARK_SURFACE } else { Colors::LIGHT_SURFACE }
            )),
            border: Border {
                radius: 16.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// Style for success feedback
pub fn success_container_style(is_dark: bool) -> impl Fn(&iced::Theme) -> container::Appearance {
    move |_theme: &iced::Theme| {
        container::Appearance {
            background: Some(Background::Color(opaque_over_bg(Colors::SUCCESS_LIGHT, is_dark))),
            text_color: Some(Colors::SUCCESS),
            border: Border {
                color: Colors::SUCCESS,
                width: 2.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    }
}

/// Style for error feedback
pub fn error_container_style(is_dark: bool) -> impl Fn(&iced::Theme) -> container::Appearance {
    move |_theme: &iced::Theme| {
        container::Appearance {
            background: Some(Background::Color(opaque_over_bg(Colors::ERROR_LIGHT, is_dark))),
            text_color: Some(Colors::ERROR),
            border: Border {
                color: Colors::ERROR,
                width: 2.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    }
}

/// Style for answer options
pub fn answer_option_style(is_dark: bool, is_selected: bool) -> impl Fn(&iced::Theme) -> container::Appearance {
    move |_theme: &iced::Theme| {
        let bg_color = if is_selected {
            if is_dark {
                opaque_over_bg(Color::from_rgba(0.29, 0.47, 0.86, 0.2), is_dark)
            } else {
                opaque_over_bg(Color::from_rgba(0.29, 0.47, 0.86, 0.1), is_dark)
            }
        } else {
            if is_dark { Colors::DARK_CARD } else { Colors::LIGHT_CARD }
        };
        
        let border_color = if is_selected {
            Colors::PRIMARY
        } else {
            if is_dark { Colors::DARK_BORDER } else { Colors::LIGHT_BORDER }
        };
        
        container::Appearance {
            background: Some(Background::Color(bg_color)),
            border: Border {
                color: border_color,
                width: if is_selected { 2.0 } else { 1.0 },
                radius: 10.0.into(),
            },
            ..Default::default()
        }
    }
}

/// Style for correct answer highlighting
pub fn correct_answer_style(is_dark: bool) -> impl Fn(&iced::Theme) -> container::Appearance {
    move |_theme: &iced::Theme| {
        container::Appearance {
            background: Some(Background::Color(opaque_over_bg(Colors::SUCCESS_LIGHT, is_dark))),
            border: Border {
                color: Colors::SUCCESS,
                width: 2.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        }
    }
}

/// Style for incorrect answer highlighting
pub fn incorrect_answer_style(is_dark: bool) -> impl Fn(&iced::Theme) -> container::Appearance {
    move |_theme: &iced::Theme| {
        container::Appearance {
            background: Some(Background::Color(opaque_over_bg(Colors::ERROR_LIGHT, is_dark))),
            border: Border {
                color: Colors::ERROR,
                width: 2.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        }
    }
}

/// Style for the header bar
pub fn header_style(is_dark: bool) -> impl Fn(&iced::Theme) -> container::Appearance {
    move |_theme: &iced::Theme| {
        container::Appearance {
            background: Some(Background::Color(
                if is_dark { Colors::DARK_SURFACE } else { Colors::LIGHT_SURFACE }
            )),
            border: Border {
                color: if is_dark { Colors::DARK_BORDER } else { Colors::LIGHT_BORDER },
                width: 0.0,
                radius: [0.0, 0.0, 16.0, 16.0].into(),
            },
            ..Default::default()
        }
    }
}

/// Style for progress bar background
pub fn progress_bar_bg_style(is_dark: bool) -> impl Fn(&iced::Theme) -> container::Appearance {
    move |_theme: &iced::Theme| {
        container::Appearance {
            background: Some(Background::Color(
                if is_dark { Colors::DARK_BORDER } else { Colors::LIGHT_BORDER }
            )),
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// Style for progress bar fill
pub fn progress_bar_fill_style(_is_dark: bool) -> impl Fn(&iced::Theme) -> container::Appearance {
    move |_theme: &iced::Theme| {
        container::Appearance {
            background: Some(Background::Color(Colors::PRIMARY)),
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// Style for session cards on the main menu
pub fn session_card_style(is_dark: bool) -> impl Fn(&iced::Theme) -> container::Appearance {
    move |_theme: &iced::Theme| {
        container::Appearance {
            background: Some(Background::Color(
                if is_dark { Colors::DARK_CARD } else { Colors::LIGHT_CARD }
            )),
            border: Border {
                color: if is_dark { Colors::DARK_BORDER } else { Colors::LIGHT_BORDER },
                width: 1.0,
                radius: 14.0.into(),
            },
            ..Default::default()
        }
    }
}

/// Badge style for repetition counts
pub fn badge_style(is_dark: bool) -> impl Fn(&iced::Theme) -> container::Appearance {
    move |_theme: &iced::Theme| {
        container::Appearance {
            background: Some(Background::Color(
                if is_dark { Colors::DARK_CARD } else { Colors::LIGHT_CARD }
            )),
            border: Border {
                radius: 12.0.into(),
                color: if is_dark { Colors::DARK_BORDER } else { Colors::LIGHT_BORDER },
                width: 1.0,
            },
            ..Default::default()
        }
    }
}

/// Letter badge style for answer options
pub fn letter_badge_style(is_dark: bool, is_selected: bool) -> impl Fn(&iced::Theme) -> container::Appearance {
    move |_theme: &iced::Theme| {
        container::Appearance {
            background: Some(Background::Color(
                if is_selected { Colors::PRIMARY } else {
                    if is_dark { Colors::DARK_BORDER } else { Colors::LIGHT_BORDER }
                }
            )),
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            text_color: Some(if is_selected { Color::WHITE } else {
                if is_dark { Colors::DARK_TEXT_SECONDARY } else { Colors::LIGHT_TEXT_SECONDARY }
            }),
            ..Default::default()
        }
    }
}

/// Frame drawn around images so they stand out from the card background
pub fn image_frame_style() -> impl Fn(&iced::Theme) -> container::Appearance {
    move |_theme: &iced::Theme| container::Appearance {
        background: Some(Background::Color(Color::WHITE)),
        border: Border {
            color: Color::from_rgba(0.5, 0.5, 0.5, 0.35),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

/// Placeholder shown when an image file cannot be found
pub fn missing_image_style() -> impl Fn(&iced::Theme) -> container::Appearance {
    move |_theme: &iced::Theme| container::Appearance {
        background: Some(Background::Color(Color::from_rgba(0.95, 0.70, 0.23, 0.12))),
        border: Border {
            color: Colors::WARNING,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

/// Accent color for a hot streak level (1 = hot, 2 = hotter, 3 = on fire)
pub fn hot_streak_color(level: u32) -> Color {
    match level {
        0 | 1 => Colors::STREAK_HOT,
        2 => Colors::STREAK_HOTTER,
        _ => Colors::STREAK_ON_FIRE,
    }
}

/// Badge / banner style for an active hot streak
pub fn hot_streak_style(level: u32, is_dark: bool) -> impl Fn(&iced::Theme) -> container::Appearance {
    move |_theme: &iced::Theme| {
        let accent = hot_streak_color(level);
        let alpha = 0.10 + 0.05 * level.min(3) as f32;
        container::Appearance {
            background: Some(Background::Color(opaque_over_bg(Color { a: alpha, ..accent }, is_dark))),
            text_color: Some(accent),
            border: Border {
                color: accent,
                width: if level >= 3 { 2.0 } else { 1.5 },
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    }
}
