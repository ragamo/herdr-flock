use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};

use crate::app::{App, LogFilter};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

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
    let filtered: Vec<_> = app
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

    let header = Row::new(vec!["", "Name", "Project", "Born", "Died", "Tasks"])
        .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD));

    let scroll = app.log_scroll as usize;
    let visible_rows = (area.height.saturating_sub(3)) as usize;

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

            let born = sheep.born.format("%Y-%m-%d").to_string();
            let died = sheep
                .died
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "—".to_string());

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
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(2),
        Constraint::Length(12),
        Constraint::Length(16),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(6),
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
