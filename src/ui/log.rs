use chrono::{Duration, Local};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table},
    Frame,
};

use crate::app::{App, LogFilter};
use crate::model::farm::Farm;
use crate::model::sheep::Sheep;

/// Returns the sorted indices of sheep as they appear in the log table.
/// `None` entries represent separator rows (not clickable).
pub fn sorted_log_indices(farm: &Farm, filter: LogFilter) -> Vec<Option<usize>> {
    let mut filtered: Vec<(usize, &Sheep)> = farm
        .sheep
        .iter()
        .enumerate()
        .filter(|(_, s)| match filter {
            LogFilter::All => true,
            LogFilter::Alive => s.is_alive(),
            LogFilter::Dead => !s.is_alive(),
        })
        .collect();

    filtered.sort_by(|(_, a), (_, b)| match (a.is_alive(), b.is_alive()) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        (false, false) => b.died.cmp(&a.died),
        (true, true) => std::cmp::Ordering::Equal,
    });

    let separator_pos = filtered.iter().position(|(_, s)| !s.is_alive());
    let mut result = Vec::new();

    for (pos, (idx, _)) in filtered.iter().enumerate() {
        if filter != LogFilter::Alive {
            if let Some(sep) = separator_pos {
                if pos == sep {
                    result.push(None); // separator row
                }
            }
        }
        result.push(Some(*idx));
    }

    result
}

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

// Fade dead rows — older deaths get darker
fn dead_color(sheep: &Sheep) -> Color {
    let Some(died) = sheep.died else { return Color::White };
    let days_ago = (chrono::Utc::now() - died).num_days();
    match days_ago {
        0 => Color::Rgb(180, 180, 180),
        1..=3 => Color::Rgb(140, 140, 140),
        4..=7 => Color::Rgb(100, 100, 100),
        _ => Color::Rgb(70, 70, 70),
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

    if let Some(idx) = app.selected_sheep {
        if let Some(sheep) = app.farm.sheep.get(idx) {
            if !sheep.is_alive() {
                if area.width >= 100 {
                    // Wide: epitaph on the right
                    let h_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Min(1), Constraint::Length(26)])
                        .split(chunks[1]);
                    render_table(frame, app, h_chunks[0]);
                    render_epitaph(frame, sheep, h_chunks[1]);
                } else {
                    // Narrow: epitaph below the table
                    let v_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Min(1), Constraint::Length(14)])
                        .split(chunks[1]);
                    render_table(frame, app, v_chunks[0]);
                    render_epitaph(frame, sheep, v_chunks[1]);
                }
                render_status_bar(frame, app, chunks[2]);
                return;
            }
        }
    }

    render_table(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let dead_count = app.farm.sheep.iter().filter(|s| !s.is_alive()).count();
    let alive_count = app.farm.sheep.iter().filter(|s| s.is_alive()).count();

    let filter_text = match app.log_filter {
        LogFilter::All => "All",
        LogFilter::Alive => "Alive",
        LogFilter::Dead => "Dead",
    };

    let epitaph = if dead_count == 0 {
        format!(" No souls lost yet ")
    } else if dead_count == 1 {
        format!(" In memory of 1 brave soul ")
    } else {
        format!(" In memory of {dead_count} brave souls ")
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(epitaph, Style::default().fg(Color::Rgb(160, 160, 160)).add_modifier(Modifier::ITALIC)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("🐑 {alive_count}"), Style::default().fg(Color::Green)),
        Span::styled("  ", Style::default()),
        Span::styled("Filter: ", Style::default().fg(Color::DarkGray)),
        Span::styled(filter_text, Style::default().fg(Color::Cyan)),
        Span::styled(" [f]", Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(60, 60, 60))),
    );

    frame.render_widget(header, area);
}

fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    let indices = sorted_log_indices(&app.farm, app.log_filter);

    let header = Row::new(vec!["", "Name", "Project", "Born", "Died", "Agent", "Lifespan"])
        .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD));

    let visible_rows = (area.height.saturating_sub(3)) as usize;
    let max_scroll = indices.len().saturating_sub(visible_rows);
    let scroll = (app.log_scroll as usize).min(max_scroll);

    let rows: Vec<Row> = indices
        .iter()
        .skip(scroll)
        .take(visible_rows)
        .map(|maybe_idx| {
            let Some(i) = maybe_idx else {
                return Row::new(vec![
                    "".to_string(),
                    "─── departed ───".to_string(),
                    "".to_string(), "".to_string(),
                    "".to_string(), "".to_string(), "".to_string(),
                ])
                .style(Style::default().fg(Color::Rgb(60, 60, 60)));
            };

            let sheep = &app.farm.sheep[*i];
            let is_selected = app.selected_sheep == Some(*i);
            let fg = if sheep.is_alive() { Color::White } else { dead_color(sheep) };
            let style = if is_selected {
                Style::default().bg(Color::Rgb(50, 50, 60)).fg(Color::White)
            } else {
                Style::default().fg(fg)
            };

            let status = if sheep.is_alive() { "●" } else { "🪦" };
            let born = sheep.born.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string();
            let died = sheep
                .died
                .map(|d| d.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "—".to_string());
            let lifespan = if let Some(death) = sheep.died {
                format_duration(death - sheep.born)
            } else {
                format!("{}~", format_duration(chrono::Utc::now() - sheep.born))
            };

            Row::new(vec![
                status.to_string(),
                sheep.name.clone(),
                sheep.project.clone(),
                born,
                died,
                sheep.agent.clone(),
                lifespan,
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(3),
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
                .border_style(Style::default().fg(Color::Rgb(40, 40, 40))),
        );

    frame.render_widget(table, area);
}

fn render_epitaph(frame: &mut Frame, sheep: &Sheep, area: Rect) {
    let lifespan = sheep
        .died
        .map(|d| format_duration(d - sheep.born))
        .unwrap_or_else(|| "?".to_string());
    let born = sheep.born.with_timezone(&Local).format("%Y-%m-%d").to_string();
    let died = sheep
        .died
        .map(|d| d.with_timezone(&Local).format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "?".to_string());

    // Truncate text to exactly 11 chars to fit inside the gravestone
    let fit = |s: &str| -> String {
        if s.len() > 11 { format!("{:.10}…", s) } else { format!("{:^11}", s) }
    };

    let name_fit = fit(&sheep.name);
    let life_fit = fit(&lifespan);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("   .-----------.  ", Style::default().fg(Color::Gray))),
        Line::from(Span::styled("  /             \\ ", Style::default().fg(Color::Gray))),
        Line::from(Span::styled("  |     RIP     | ", Style::default().fg(Color::Rgb(120, 120, 120)))),
        Line::from(vec![
            Span::styled("  | ", Style::default().fg(Color::Gray)),
            Span::styled(name_fit, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" | ", Style::default().fg(Color::Gray)),
        ]),
        Line::from(Span::styled("  |             | ", Style::default().fg(Color::Gray))),
        Line::from(vec![
            Span::styled("  | ", Style::default().fg(Color::Gray)),
            Span::styled(life_fit, Style::default().fg(Color::Rgb(150, 150, 150))),
            Span::styled(" | ", Style::default().fg(Color::Gray)),
        ]),
        Line::from(Span::styled("  |_____________| ", Style::default().fg(Color::Gray))),
        Line::from(Span::styled("  |             | ", Style::default().fg(Color::Rgb(80, 60, 40)))),
        Line::from(Span::styled("  |_____________| ", Style::default().fg(Color::Rgb(80, 60, 40)))),
        Line::from(""),
        Line::from(Span::styled(format!("  Born:  {born}"), Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled(format!("  Died:  {died}"), Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled(format!("  Agent: {}", sheep.agent), Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled(format!("  Proj:  {}", sheep.project), Style::default().fg(Color::DarkGray))),
    ];

    let epitaph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(60, 60, 60))),
    );

    frame.render_widget(Clear, area);
    frame.render_widget(epitaph, area);
}

fn render_status_bar(frame: &mut Frame, _app: &App, area: Rect) {
    let status = Line::from(vec![
        Span::styled(" [Tab] Flock ", Style::default().fg(Color::DarkGray)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("[↑↓] Scroll ", Style::default().fg(Color::DarkGray)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("[f] Filter ", Style::default().fg(Color::DarkGray)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("[q] Quit", Style::default().fg(Color::DarkGray)),
    ]);

    frame.render_widget(Paragraph::new(status), area);
}
