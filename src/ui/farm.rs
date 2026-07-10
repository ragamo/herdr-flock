use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::animation::sprites::{get_sprite, Px, SPRITE_CHAR_HEIGHT, SPRITE_CHAR_WIDTH, SPRITE_PX_HEIGHT, SPRITE_PX_WIDTH};
use crate::app::App;
use crate::model::sheep::SheepState;

const COLOR_BG: Color = Color::Rgb(34, 80, 34);
const COLOR_WHITE: Color = Color::Rgb(250, 250, 250);
const COLOR_BLACK: Color = Color::Rgb(20, 20, 20);
const COLOR_BEIGE: Color = Color::Rgb(210, 180, 140);

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let farm_area = chunks[0];
    let status_area = chunks[1];

    render_terrain(frame, farm_area);
    render_sheep(frame, app, farm_area);

    if let Some(idx) = app.selected_sheep {
        if let Some(sheep) = app.farm.sheep.get(idx) {
            render_tooltip(frame, sheep, farm_area);
        }
    }

    render_status_bar(frame, app, status_area);
}

fn render_terrain(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(80, 140, 80)))
        .style(Style::default().bg(COLOR_BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();
    for row in 0..inner.height {
        let mut spans = Vec::new();
        for col in 0..inner.width {
            let (ch, fg) = terrain_char(col, row, inner.width, inner.height);
            spans.push(Span::styled(
                ch,
                Style::default().fg(fg).bg(COLOR_BG),
            ));
        }
        lines.push(Line::from(spans));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn terrain_char(col: u16, row: u16, width: u16, height: u16) -> (&'static str, Color) {

    if col == width - 4 && row == height / 2 {
        return ("◎", Color::Cyan);
    }
    if (col == width - 5 || col == width - 3) && row == height / 2 {
        return ("~", Color::Rgb(100, 180, 220));
    }

    if row % 7 == 3 && col % 14 == 5 {
        return ("♣", Color::Rgb(50, 110, 50));
    }
    if row % 9 == 1 && col % 11 == 2 {
        return ("·", Color::Rgb(60, 100, 60));
    }

    (" ", COLOR_BG)
}

fn render_sheep(frame: &mut Frame, app: &App, area: Rect) {
    let inner_x = area.x + 1;
    let inner_y = area.y + 1;

    for (i, sheep) in app.farm.sheep.iter().enumerate().filter(|(_, s)| s.is_alive()) {
        let sprite = get_sprite(sheep.state, sheep.direction, sheep.anim_frame);
        let col = inner_x + sheep.display_col();
        let row = inner_y + sheep.display_row();

        if col + SPRITE_CHAR_WIDTH > area.right() || row + SPRITE_CHAR_HEIGHT > area.bottom() {
            continue;
        }

        let is_selected = app.selected_sheep == Some(i);
        let working_pulse = sheep.state == SheepState::Working && app.tick_count % 8 < 4;

        for char_row in 0..SPRITE_CHAR_HEIGHT as usize {
            let px_row_top = char_row * 2;
            let px_row_bot = char_row * 2 + 1;

            let mut spans = Vec::new();
            for px_col in 0..SPRITE_PX_WIDTH {
                let top_px = sprite.pixel_at(px_row_top, px_col);
                let bot_px = if px_row_bot < SPRITE_PX_HEIGHT {
                    sprite.pixel_at(px_row_bot, px_col)
                } else {
                    Px::T
                };

                let (ch, fg, bg) = half_block(top_px, bot_px, working_pulse);
                spans.push(Span::styled(ch, Style::default().fg(fg).bg(bg)));
            }

            if is_selected {
                let y = row + char_row as u16;
                let highlight = Line::from(spans);
                frame.render_widget(
                    Paragraph::new(vec![highlight]),
                    Rect::new(col, y, SPRITE_CHAR_WIDTH, 1),
                );
            } else {
                let y = row + char_row as u16;
                frame.render_widget(
                    Paragraph::new(vec![Line::from(spans)]),
                    Rect::new(col, y, SPRITE_CHAR_WIDTH, 1),
                );
            }
        }

        if is_selected {
            let name_y = row.saturating_sub(1);
            let name = &sheep.name;
            let name_x = col + (SPRITE_CHAR_WIDTH / 2).saturating_sub(name.len() as u16 / 2);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    name.clone(),
                    Style::default()
                        .fg(Color::White)
                        .bg(COLOR_BG)
                        .add_modifier(Modifier::BOLD),
                ))),
                Rect::new(name_x, name_y, name.len() as u16, 1),
            );
        }
    }
}

fn half_block(top: Px, bot: Px, working_pulse: bool) -> (&'static str, Color, Color) {
    if top == Px::Z && bot == Px::Z {
        return ("z", Color::Rgb(180, 180, 220), COLOR_BG);
    }
    if top == Px::Z {
        let bot_color = px_color(bot, working_pulse);
        let bg = if bot == Px::T { COLOR_BG } else { bot_color };
        return ("z", Color::Rgb(180, 180, 220), bg);
    }
    if bot == Px::Z {
        let top_color = px_color(top, working_pulse);
        let bg = if top == Px::T { COLOR_BG } else { top_color };
        return ("z", Color::Rgb(180, 180, 220), bg);
    }

    let top_color = px_color(top, working_pulse);
    let bot_color = px_color(bot, working_pulse);

    match (top, bot) {
        (Px::T, Px::T) => (" ", COLOR_BG, COLOR_BG),
        (_, Px::T) => ("▀", top_color, COLOR_BG),
        (Px::T, _) => ("▄", bot_color, COLOR_BG),
        _ if top_color == bot_color => ("█", top_color, top_color),
        _ => ("▀", top_color, bot_color),
    }
}

fn px_color(px: Px, working_pulse: bool) -> Color {
    match px {
        Px::T => COLOR_BG,
        Px::K => COLOR_BLACK,
        Px::W => {
            if working_pulse {
                Color::Rgb(255, 240, 100)
            } else {
                COLOR_WHITE
            }
        }
        Px::B => COLOR_BEIGE,
        Px::Z => COLOR_BG,
        Px::M => Color::Rgb(180, 40, 40),
    }
}

fn render_tooltip(frame: &mut Frame, sheep: &crate::model::sheep::Sheep, area: Rect) {
    let text = format!(
        " {} | {} | tasks: {} ",
        sheep.name, sheep.project, sheep.tasks_completed
    );
    let width = text.len() as u16 + 2;
    let x = (area.width / 2).saturating_sub(width / 2) + area.x;
    let y = area.bottom().saturating_sub(4);

    let tooltip = Paragraph::new(Line::from(Span::styled(
        text,
        Style::default().fg(Color::Black).bg(Color::White),
    )))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White).bg(Color::Rgb(40, 40, 40)))
            .style(Style::default().bg(Color::Rgb(40, 40, 40))),
    );

    frame.render_widget(tooltip, Rect::new(x, y, width, 3));
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let alive = app.farm.sheep.iter().filter(|s| s.is_alive()).count();
    let dead = app.farm.sheep.iter().filter(|s| !s.is_alive()).count();

    let conn_indicator = if app.connected {
        Span::styled(" ● herdr ", Style::default().fg(Color::Green))
    } else {
        Span::styled(" ○ demo ", Style::default().fg(Color::DarkGray))
    };

    let status = Line::from(vec![
        conn_indicator,
        Span::styled("| ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("🐑 {alive}"),
            Style::default().fg(Color::Green),
        ),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("🪦 {dead}"),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Tab] Switch", Style::default().fg(Color::DarkGray)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("[q] Quit", Style::default().fg(Color::DarkGray)),
    ]);

    frame.render_widget(Paragraph::new(status), area);
}
