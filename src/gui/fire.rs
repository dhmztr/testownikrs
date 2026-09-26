// Animated "burning screen" effect shown during a hot streak.
//
// iced 0.12 has no stack widget, so `FireOverlay` wraps the screen content and
// delegates layout/events to it. It paints the window background and the
// flames first and the content on top, so cards, buttons and text stay
// readable while the fire burns in every gap between them.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{self, tree, Tree, Widget};
use iced::advanced::{mouse, overlay, Clipboard, Renderer as _, Shell};
use iced::gradient::Linear;
use iced::{event, Background, Border, Color, Element, Event, Length, Rectangle, Size, Vector};

/// Maximum share of the window height the flames may cover
const MAX_HEIGHT_FRACTION: f32 = 0.5;

/// Animation state of the fire effect
#[derive(Debug, Clone, Copy, Default)]
pub struct FireState {
    /// Animation clock in seconds
    pub time: f32,
    /// Displayed strength 0.0..=1.0, eased towards `target` for smooth
    /// growth and a gentle fade-out when the streak breaks
    pub level: f32,
    /// Strength the effect is moving towards
    pub target: f32,
}

impl FireState {
    /// Strength for a streak: 0 below the threshold, then grows from 0.25
    /// at the threshold to 1.0 at four times the threshold.
    pub fn target_for(streak: u32, threshold: u32, enabled: bool) -> f32 {
        let threshold = threshold.max(1);
        if !enabled || streak < threshold {
            return 0.0;
        }
        let growth = (streak - threshold) as f32 / (3 * threshold) as f32;
        0.25 + 0.75 * growth.clamp(0.0, 1.0)
    }

    pub fn tick(&mut self, dt: f32) {
        let dt = dt.clamp(0.0, 0.1);
        self.time = (self.time + dt) % 10_000.0;
        // Grow faster than fade so a new streak level is felt immediately
        let speed = if self.target > self.level { 4.0 } else { 1.5 };
        self.level += (self.target - self.level) * (dt * speed).min(1.0);
        if self.target == 0.0 && self.level < 0.005 {
            self.level = 0.0;
        }
    }

    /// Whether frames are needed (effect visible or still fading)
    pub fn is_animating(&self) -> bool {
        self.target > 0.0 || self.level > 0.0
    }
}

pub struct FireOverlay<'a, Message> {
    content: Element<'a, Message>,
    state: FireState,
    background: Color,
}

impl<'a, Message> FireOverlay<'a, Message> {
    /// `content` must have a transparent background; `background` is painted
    /// underneath the flames instead.
    pub fn new(content: impl Into<Element<'a, Message>>, state: FireState, background: Color) -> Self {
        Self { content: content.into(), state, background }
    }
}

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for FireOverlay<'a, Message> {
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(&self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        self.content.as_widget().layout(&mut tree.children[0], renderer, limits)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        self.content.as_widget().operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.content.as_widget_mut().on_event(
            &mut tree.children[0], event, layout, cursor, renderer, clipboard, shell, viewport,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(&mut tree.children[0], layout, renderer, translation)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        renderer.fill_quad(
            Quad { bounds, ..Default::default() },
            Background::Color(self.background),
        );
        if self.state.level > 0.0 {
            draw_fire(renderer, bounds, self.state);
        }

        self.content
            .as_widget()
            .draw(&tree.children[0], renderer, theme, style, layout, cursor, viewport);
    }
}

impl<'a, Message: 'a> From<FireOverlay<'a, Message>> for Element<'a, Message> {
    fn from(overlay: FireOverlay<'a, Message>) -> Self {
        Element::new(overlay)
    }
}

fn draw_fire(renderer: &mut iced::Renderer, bounds: Rectangle, state: FireState) {
    let strength = state.level.clamp(0.0, 1.0);
    let t = state.time;
    // Heat 0 at the start of a streak, 1 at full blaze: drives color shift to red
    let heat = ((strength - 0.25) / 0.75).clamp(0.0, 1.0);
    let flame_height = bounds.height * MAX_HEIGHT_FRACTION * strength;
    let bottom = bounds.y + bounds.height;

    let outer = lerp_color(Color::from_rgb(1.0, 0.55, 0.08), Color::from_rgb(0.85, 0.04, 0.02), heat);
    let inner = lerp_color(Color::from_rgb(1.0, 0.85, 0.30), Color::from_rgb(1.0, 0.40, 0.05), heat);
    let base_alpha = 0.30 + 0.35 * strength;

    // Glow along the whole bottom edge
    fill_gradient(
        renderer,
        Rectangle::new(
            iced::Point::new(bounds.x, bottom - flame_height * 1.25),
            Size::new(bounds.width, flame_height * 1.25),
        ),
        0.0,
        &[(0.0, with_alpha(outer, base_alpha * 0.9)), (0.45, with_alpha(outer, base_alpha * 0.35)), (1.0, with_alpha(outer, 0.0))],
    );

    // Tongues of flame: two layers of flickering rounded columns
    let columns = ((bounds.width / 22.0) as usize).max(16);
    let column_width = bounds.width / columns as f32;
    for (layer_scale, width_scale, color, alpha) in [
        (1.0, 1.9, outer, base_alpha * 0.85),
        (0.6, 1.2, inner, base_alpha * 0.9),
    ] {
        for i in 0..=columns {
            let seed = hash(i as u32 + (layer_scale * 100.0) as u32);
            let phase = seed * std::f32::consts::TAU;
            let flicker = 0.5
                + 0.3 * (t * (2.1 + seed * 1.3) + phase).sin()
                + 0.2 * (t * (5.3 + seed * 2.0) + i as f32 * 0.8).sin();
            // Uneven heights and widths so the tongues read as flames, not bars
            let height = flame_height * layer_scale * (0.3 + 0.95 * flicker.clamp(0.0, 1.0)) * (0.75 + 0.5 * seed);
            let width = column_width * width_scale * (0.7 + 0.7 * hash(i as u32 * 3 + 17));
            let sway = (t * 1.6 + phase).sin() * column_width * 0.25;
            let x = bounds.x + i as f32 * column_width - width / 2.0 + sway;

            fill_gradient_rounded(
                renderer,
                Rectangle::new(iced::Point::new(x, bottom - height), Size::new(width, height)),
                width / 2.0,
                &[(0.0, with_alpha(color, alpha)), (0.55, with_alpha(color, alpha * 0.6)), (1.0, with_alpha(color, 0.0))],
            );
        }
    }

    // Embers rising above the flames; more of them as the streak grows
    let embers = (8.0 + 40.0 * strength) as u32;
    for k in 0..embers {
        let seed = hash(k * 7 + 3);
        let speed = 0.25 + 0.35 * hash(k * 13 + 1);
        let life = (t * speed + seed).fract();
        let x = bounds.x + bounds.width * hash(k * 31 + 5) + (t * 2.0 + seed * 9.0).sin() * 12.0;
        let y = bottom - life * flame_height * 2.0;
        let size = 2.0 + 3.0 * hash(k * 17 + 11) * (1.0 - life * 0.5);
        let alpha = (1.0 - life) * (0.5 + 0.4 * strength);
        renderer.fill_quad(
            Quad {
                bounds: Rectangle::new(iced::Point::new(x, y), Size::new(size, size)),
                border: Border { radius: (size / 2.0).into(), ..Default::default() },
                ..Default::default()
            },
            Background::Color(with_alpha(inner, alpha)),
        );
    }
}

/// Fills `rect` with a vertical gradient; stop offset 0.0 is the bottom edge
fn fill_gradient(renderer: &mut iced::Renderer, rect: Rectangle, radius: f32, stops: &[(f32, Color)]) {
    let gradient = stops
        .iter()
        .fold(Linear::new(0.0), |g, &(offset, color)| g.add_stop(offset, color));
    renderer.fill_quad(
        Quad {
            bounds: rect,
            border: Border { radius: radius.into(), ..Default::default() },
            ..Default::default()
        },
        Background::Gradient(gradient.into()),
    );
}

/// Like `fill_gradient`, with only the top corners rounded (flame tip)
fn fill_gradient_rounded(renderer: &mut iced::Renderer, rect: Rectangle, radius: f32, stops: &[(f32, Color)]) {
    let gradient = stops
        .iter()
        .fold(Linear::new(0.0), |g, &(offset, color)| g.add_stop(offset, color));
    renderer.fill_quad(
        Quad {
            bounds: rect,
            border: Border { radius: [radius, radius, 0.0, 0.0].into(), ..Default::default() },
            ..Default::default()
        },
        Background::Gradient(gradient.into()),
    );
}

fn hash(i: u32) -> f32 {
    ((i as f32 * 12.9898 + 78.233).sin() * 43_758.547).fract().abs()
}

fn with_alpha(color: Color, a: f32) -> Color {
    Color { a: a.clamp(0.0, 1.0), ..color }
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::from_rgb(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_grows_with_streak() {
        assert_eq!(FireState::target_for(2, 3, true), 0.0);
        assert_eq!(FireState::target_for(3, 3, true), 0.25);
        assert!(FireState::target_for(6, 3, true) > 0.25);
        assert_eq!(FireState::target_for(12, 3, true), 1.0);
        assert_eq!(FireState::target_for(50, 3, true), 1.0);
        assert_eq!(FireState::target_for(50, 3, false), 0.0);
    }

    #[test]
    fn fades_out_after_streak_breaks() {
        let mut s = FireState { target: 1.0, ..Default::default() };
        for _ in 0..100 {
            s.tick(0.033);
        }
        assert!(s.level > 0.9);
        s.target = 0.0;
        for _ in 0..300 {
            s.tick(0.033);
        }
        assert_eq!(s.level, 0.0);
        assert!(!s.is_animating());
    }
}
