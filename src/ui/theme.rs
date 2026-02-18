use ratatui::style::{Color, Modifier, Style};

// Catppuccin Mocha-inspired palette
pub const BASE: Color = Color::Rgb(30, 30, 46);
pub const SURFACE0: Color = Color::Rgb(49, 50, 68);
pub const SURFACE1: Color = Color::Rgb(69, 71, 90);
pub const TEXT: Color = Color::Rgb(205, 214, 244);
pub const SUBTEXT: Color = Color::Rgb(166, 173, 200);
pub const BLUE: Color = Color::Rgb(137, 180, 250);
pub const GREEN: Color = Color::Rgb(166, 227, 161);
pub const RED: Color = Color::Rgb(243, 139, 168);
pub const YELLOW: Color = Color::Rgb(249, 226, 175);
pub const PEACH: Color = Color::Rgb(250, 179, 135);
pub const MAUVE: Color = Color::Rgb(203, 166, 247);
pub const TEAL: Color = Color::Rgb(148, 226, 213);
pub const SKY: Color = Color::Rgb(137, 220, 235);

pub fn title_style() -> Style {
    Style::default().fg(BLUE).add_modifier(Modifier::BOLD)
}

pub fn active_tab_style() -> Style {
    Style::default()
        .fg(BASE)
        .bg(BLUE)
        .add_modifier(Modifier::BOLD)
}

pub fn inactive_tab_style() -> Style {
    Style::default().fg(SUBTEXT).bg(SURFACE0)
}

pub fn header_style() -> Style {
    Style::default().fg(TEXT).bg(SURFACE0)
}

pub fn footer_style() -> Style {
    Style::default().fg(SUBTEXT).bg(SURFACE0)
}

pub fn key_hint_style() -> Style {
    Style::default().fg(BLUE)
}

pub fn label_style() -> Style {
    Style::default().fg(SUBTEXT)
}

pub fn value_style() -> Style {
    Style::default().fg(TEXT)
}

pub fn gauge_style(percent: f64) -> Style {
    let color = if percent > 90.0 {
        RED
    } else if percent > 70.0 {
        YELLOW
    } else if percent > 50.0 {
        PEACH
    } else {
        GREEN
    };
    Style::default().fg(color).bg(BASE)
}

pub fn border_style() -> Style {
    Style::default().fg(SURFACE1)
}

pub fn highlight_style() -> Style {
    Style::default().fg(BASE).bg(BLUE)
}

pub fn process_state_color(state: crate::metrics::process::ProcessState) -> Color {
    use crate::metrics::process::ProcessState;
    match state {
        ProcessState::Run => GREEN,
        ProcessState::Sleep => SUBTEXT,
        ProcessState::Idle => SURFACE1,
        ProcessState::Zombie => RED,
        ProcessState::Stop => YELLOW,
        ProcessState::Unknown => SURFACE1,
    }
}
