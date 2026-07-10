pub mod farm;
pub mod log;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, Screen};

pub const TAB_HEIGHT: u16 = 1;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(TAB_HEIGHT), Constraint::Min(1)])
        .split(area);

    render_tabs(frame, app, chunks[0]);

    match app.screen {
        Screen::Farm => farm::render(frame, app, chunks[1]),
        Screen::Log => log::render(frame, app, chunks[1]),
    }
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let farm_style = if app.screen == Screen::Farm {
        Style::default().fg(Color::White).bg(Color::Rgb(60, 60, 60)).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray).bg(Color::Rgb(30, 30, 30))
    };

    let log_style = if app.screen == Screen::Log {
        Style::default().fg(Color::White).bg(Color::Rgb(60, 60, 60)).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray).bg(Color::Rgb(30, 30, 30))
    };

    let tabs = Line::from(vec![
        Span::styled(" Flock ", farm_style),
        Span::styled(" ", Style::default().bg(Color::Rgb(20, 20, 20))),
        Span::styled(" Graveyard ", log_style),
        Span::styled(
            " ".repeat(area.width.saturating_sub(20) as usize),
            Style::default().bg(Color::Rgb(20, 20, 20)),
        ),
    ]);

    frame.render_widget(
        Paragraph::new(tabs).style(Style::default().bg(Color::Rgb(20, 20, 20))),
        area,
    );
}

pub fn tab_click(col: u16) -> Option<Screen> {
    if col < 7 {
        Some(Screen::Farm)
    } else if col >= 8 && col < 19 {
        Some(Screen::Log)
    } else {
        None
    }
}
