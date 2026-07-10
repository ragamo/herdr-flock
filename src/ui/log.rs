use chrono::{Duration, Local};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};

use crate::app::{App, LogFilter};

fn format_duration(dur: Duration) -> String {
    let days = dur.num_days();
    let hours = dur.num_hours() % 24;
    let mins = dur.num_minutes() % 60;

    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else {
        format!("{mins}m")
    }
}

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(frame, app, chunks[0]);
    render_table(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let filter_text = match app.log_filter {
        LogFilter::All => "All",
        LogFilter::Alive => "Alive",
        LogFilter::Dead => "Dead",
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(" Flock Log ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(" | Filter: ", Style::default().fg(Color::DarkGray)),
        Span::styled(filter_text, Style::default().fg(Color::Cyan)),
        Span::styled(" [f] ", Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(header, area);
}

fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    let mut filtered: Vec<_> = app
        .farm
        .sheep
        .iter()
        .enumerate()
        .filter(|(_, s)| match app.log_filter {
            LogFilter::All => true,
            LogFilter::Alive => s.is_alive(),
            LogFilter::Dead => !s.is_alive(),
        })
        .collect();

    filtered.sort_by(|(_, a), (_, b)| {
        match (a.is_alive(), b.is_alive()) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (false, false) => b.died.cmp(&a.died),
            (true, true) => std::cmp::Ordering::Equal,
        }
    });

    let header = Row::new(vec!["", "Name", "Project", "Born", "Died", "Tasks", "Lifespan"])
        .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD));

    let visible_rows = (area.height.saturating_sub(3)) as usize;
    let max_scroll = filtered.len().saturating_sub(visible_rows);
    let scroll = (app.log_scroll as usize).min(max_scroll);

    let rows: Vec<Row> = filtered
        .iter()
        .skip(scroll)
        .take(visible_rows)
        .map(|(i, sheep)| {
            let status = if sheep.is_alive() { "●" } else { "○" };
            let status_color = if sheep.is_alive() {
                Color::Green
            } else {
                Color::DarkGray
            };

            let born = sheep.born.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string();
            let died = sheep
                .died
                .map(|d| d.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "—".to_string());

            let lifespan = if let Some(death) = sheep.died {
                let dur = death - sheep.born;
                format_duration(dur)
            } else {
                let dur = chrono::Utc::now() - sheep.born;
                format!("{}~", format_duration(dur))
            };

            let is_selected = app.selected_sheep == Some(*i);
            let style = if is_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            Row::new(vec![
                Span::styled(status, Style::default().fg(status_color)).to_string(),
                sheep.name.clone(),
                sheep.project.clone(),
                born,
                died,
                sheep.tasks_completed.to_string(),
                lifespan,
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(2),
        Constraint::Length(14),
        Constraint::Length(16),
        Constraint::Length(18),
        Constraint::Length(18),
        Constraint::Length(6),
        Constraint::Length(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(table, area);
}

fn render_status_bar(frame: &mut Frame, _app: &App, area: Rect) {
    let status = Line::from(vec![
        Span::styled(" [Tab]", Style::default().fg(Color::DarkGray)),
        Span::styled(" Farm ", Style::default().fg(Color::Cyan)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("[↑↓] Scroll ", Style::default().fg(Color::DarkGray)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("[f] Filter ", Style::default().fg(Color::DarkGray)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("[q] Quit", Style::default().fg(Color::DarkGray)),
    ]);

    frame.render_widget(Paragraph::new(status), area);
}
