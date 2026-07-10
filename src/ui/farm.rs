use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::animation::sprites::{get_sprite, Px, SPRITE_CHAR_HEIGHT, SPRITE_CHAR_WIDTH, SPRITE_PX_HEIGHT, SPRITE_PX_WIDTH};
use crate::app::{App, Atmosphere, WeatherKind};
use crate::model::sheep::SheepState;

// ─── Color utilities ────────────────────────────────────────────────────────

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t.clamp(0.0, 1.0)) as u8
}

fn lerp_color(from: (u8, u8, u8), to: (u8, u8, u8), t: f32) -> Color {
    Color::Rgb(lerp_u8(from.0, to.0, t), lerp_u8(from.1, to.1, t), lerp_u8(from.2, to.2, t))
}

fn dim_color(r: u8, g: u8, b: u8, factor: f32) -> Color {
    let f = factor.clamp(0.0, 1.0);
    Color::Rgb((r as f32 * f) as u8, (g as f32 * f) as u8, (b as f32 * f) as u8)
}

fn cell_hash(col: u16, row: u16) -> u32 {
    let v = (col as u32)
        .wrapping_mul(2654435761)
        .wrapping_add((row as u32).wrapping_mul(2246822519));
    v ^ (v >> 16)
}

fn atmosphere_bg(night_t: f32) -> Color {
    lerp_color((34, 80, 34), (8, 18, 8), night_t)
}

fn fence_fg(night_t: f32) -> Color {
    lerp_color((160, 120, 80), (60, 45, 30), night_t)
}

// ─── TerrainContext ──────────────────────────────────────────────────────────

struct TerrainContext<'a> {
    width: u16,
    height: u16,
    tick: u64,
    night_t: f32,
    bg_color: Color,
    trees: &'a [(u16, u16)],
    river_row: u16,
}

// ─── Render entry point ─────────────────────────────────────────────────────

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let farm_area = chunks[0];
    let status_area = chunks[1];

    let night_t = app.atmosphere.night_factor();
    let bg_color = atmosphere_bg(night_t);

    render_terrain(frame, app, farm_area, bg_color, night_t);
    render_sheep(frame, app, farm_area, bg_color, night_t);

    if let Some(idx) = app.selected_sheep {
        if let Some(sheep) = app.farm.sheep.get(idx) {
            render_tooltip(frame, sheep, farm_area);
        }
    }

    render_status_bar(frame, app, status_area);
}

// ─── Terrain ────────────────────────────────────────────────────────────────

fn render_terrain(frame: &mut Frame, app: &App, area: Rect, bg_color: Color, night_t: f32) {
    let border_color = lerp_color((80, 140, 80), (25, 50, 25), night_t);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(bg_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let ctx = TerrainContext {
        width: inner.width,
        height: inner.height,
        tick: app.tick_count,
        night_t,
        bg_color,
        trees: &app.farm.trees,
        river_row: app.farm.river_row,
    };

    let mut lines: Vec<Line> = (0..inner.height)
        .map(|row| {
            Line::from(
                (0..inner.width)
                    .map(|col| {
                        let (ch, fg, bg) = terrain_cell(col, row, &ctx);
                        Span::styled(ch, Style::default().fg(fg).bg(bg))
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();

    overlay_precipitation(&mut lines, &app.atmosphere, inner.width, inner.height);

    frame.render_widget(Paragraph::new(lines), inner);
}

fn terrain_cell(col: u16, row: u16, ctx: &TerrainContext) -> (&'static str, Color, Color) {
    let w = ctx.width;
    let h = ctx.height;
    let n = ctx.night_t;

    // 1. Perimeter fence
    if col == 0 || col == w - 1 || row == 0 || row == h - 1 {
        let ch = match (col == 0, col == w - 1, row == 0, row == h - 1) {
            (true, _, true, _) => "┌",
            (_, true, true, _) => "┐",
            (true, _, _, true) => "└",
            (_, true, _, true) => "┘",
            (true, _, _, _) | (_, true, _, _) => "│",
            _ => "─",
        };
        return (ch, fence_fg(n), ctx.bg_color);
    }

    // 2. Trees (4 wide × 6 tall)
    //    rows 0-3: canopy, rows 4-5: trunk (cols 1-2 of 4)
    for &(tc, tr) in ctx.trees {
        let lc = col.wrapping_sub(tc);
        let lr = row.wrapping_sub(tr);
        if lc >= 5 || lr >= 6 {
            continue;
        }
        let g_dark  = lerp_color((18, 62, 18),  (7, 20, 7),  n);
        let g_light = lerp_color((38, 95, 38),  (14, 35, 14), n);
        let brown   = lerp_color((110, 75, 40), (44, 30, 16), n);
        let bg = ctx.bg_color;
        // Use tree+col hash to scatter a few light highlight pixels
        let h = cell_hash(tc + lc as u16, tr + lr as u16);
        let canopy = if h % 10 == 0 { ("█", g_light, g_light) } else { ("█", g_dark, g_dark) };
        // 5 wide × 6 tall:
        //   rows 0-3: canopy (corners transparent)
        //   rows 4-5: trunk cols 1-3
        return match (lc, lr) {
            (0, 0) | (4, 0) | (0, 3) | (4, 3) => (" ", bg, bg),
            (_, 0) | (_, 1) | (_, 2) | (_, 3) => canopy,
            (1, 4) | (2, 4) | (3, 4) => ("█", brown, brown),
            (0, 4) | (4, 4) => (" ", bg, bg),
            (1, 5) | (2, 5) | (3, 5) => ("█", brown, brown),
            (0, 5) | (4, 5) => (" ", bg, bg),
            _ => (" ", bg, bg),
        };
    }

    // 3. River
    if row == ctx.river_row || row == ctx.river_row + 1 {
        let phase = (ctx.tick / 4 + col as u64) % 3;
        let water_fg = match phase {
            0 => lerp_color((80, 160, 220), (30, 60, 110), n),
            1 => lerp_color((70, 140, 200), (25, 52, 100), n),
            _ => lerp_color((90, 170, 230), (35, 65, 115), n),
        };
        let water_bg = lerp_color((20, 60, 110), (8, 22, 45), n);
        return ("≈", water_fg, water_bg);
    }

    // 4. Pond (existing)
    if col == w - 4 && row == h / 2 {
        return ("◎", lerp_color((0, 210, 210), (0, 80, 100), n), ctx.bg_color);
    }
    if (col == w - 5 || col == w - 3) && row == h / 2 {
        return ("~", lerp_color((100, 180, 220), (40, 70, 110), n), ctx.bg_color);
    }

    // 5. Stars at night
    if n > 0.5 {
        let h_val = cell_hash(col, row);
        let star_threshold = ((n - 0.5) * 24.0) as u32;
        if h_val % 60 < star_threshold {
            let brightness = lerp_u8(0, 200, n - 0.5);
            let star_ch = if h_val % 3 == 0 { "·" } else { "*" };
            return (star_ch, Color::Rgb(brightness, brightness, brightness + 20), ctx.bg_color);
        }
    }

    // 6. Grass texture + clover/dots (day-scaled)
    let grass_factor = 1.0 - n * 0.7;
    let h_val = cell_hash(col, row);
    let patch = (h_val % 4) as f32 / 4.0;
    let grass_bg = dim_color(
        lerp_u8(28, 40, patch),
        lerp_u8(72, 90, patch),
        lerp_u8(28, 40, patch),
        1.0 - n * 0.75,
    );

    // Clover: use a darker bg patch so the char is visibly darker than surroundings
    if h_val % 80 < 1 {
        let clover_bg = dim_color(20, 55, 20, 1.0 - n * 0.75);
        return (" ", clover_bg, clover_bg);
    }
    // Dots: slightly lighter green spot
    if h_val % 30 < 1 {
        let dot_bg = dim_color(45, 100, 45, 1.0 - n * 0.75);
        return (" ", dot_bg, dot_bg);
    }

    (" ", grass_bg, grass_bg)
}

fn overlay_precipitation(lines: &mut Vec<Line>, atm: &Atmosphere, w: u16, h: u16) {
    let intensity = atm.weather_intensity();
    if intensity < 0.05 {
        return;
    }

    let (ch, base_r, base_g, base_b) = match atm.active_weather() {
        None => return,
        Some(WeatherKind::Rain) => {
            if intensity > 0.6 { ("|", 100u8, 140u8, 200u8) } else { ("'", 130, 165, 215) }
        }
        Some(WeatherKind::Snow) => ("*", 210u8, 220u8, 255u8),
    };

    let alpha = intensity;
    let fg = Color::Rgb(
        (base_r as f32 * alpha) as u8,
        (base_g as f32 * alpha) as u8,
        (base_b as f32 * alpha) as u8,
    );

    for p in &atm.precipitation {
        let px = p.x as u16;
        let py = p.y as u16;
        if px >= w || py >= h {
            continue;
        }
        if let Some(line) = lines.get_mut(py as usize) {
            if let Some(span) = line.spans.get_mut(px as usize) {
                let existing_bg = span.style.bg.unwrap_or(Color::Reset);
                *span = Span::styled(ch, Style::default().fg(fg).bg(existing_bg));
            }
        }
    }
}

// ─── Sheep ──────────────────────────────────────────────────────────────────

fn render_sheep(frame: &mut Frame, app: &App, area: Rect, bg_color: Color, night_t: f32) {
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

                let (ch, fg, bg) = half_block(top_px, bot_px, working_pulse, bg_color, night_t);
                spans.push(Span::styled(ch, Style::default().fg(fg).bg(bg)));
            }

            let y = row + char_row as u16;
            frame.render_widget(
                Paragraph::new(vec![Line::from(spans)]),
                Rect::new(col, y, SPRITE_CHAR_WIDTH, 1),
            );
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
                        .bg(bg_color)
                        .add_modifier(Modifier::BOLD),
                ))),
                Rect::new(name_x, name_y, name.len() as u16, 1),
            );
        }
    }
}

fn half_block(top: Px, bot: Px, working_pulse: bool, bg_color: Color, night_t: f32) -> (&'static str, Color, Color) {
    if top == Px::Z || bot == Px::Z {
        let zzz = Color::Rgb(
            lerp_u8(180, 80, night_t),
            lerp_u8(180, 80, night_t),
            lerp_u8(220, 120, night_t),
        );
        let other_px = if top == Px::Z { bot } else { top };
        let other_color = px_color(other_px, working_pulse, bg_color, night_t);
        let bg = if other_px == Px::T { bg_color } else { other_color };
        return ("z", zzz, bg);
    }

    let top_color = px_color(top, working_pulse, bg_color, night_t);
    let bot_color = px_color(bot, working_pulse, bg_color, night_t);

    match (top, bot) {
        (Px::T, Px::T) => (" ", bg_color, bg_color),
        (_, Px::T) => ("▀", top_color, bg_color),
        (Px::T, _) => ("▄", bot_color, bg_color),
        _ if top_color == bot_color => ("█", top_color, top_color),
        _ => ("▀", top_color, bot_color),
    }
}

fn px_color(px: Px, working_pulse: bool, bg_color: Color, night_t: f32) -> Color {
    let dim = 1.0 - night_t * 0.60;
    match px {
        Px::T | Px::Z => bg_color,
        Px::K => dim_color(20, 20, 20, dim),
        Px::W => {
            if working_pulse {
                dim_color(255, 240, 100, dim)
            } else {
                dim_color(250, 250, 250, dim)
            }
        }
        Px::B => dim_color(210, 180, 140, dim),
        Px::M => dim_color(180, 40, 40, dim),
    }
}

// ─── Tooltip ────────────────────────────────────────────────────────────────

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
        Style::default().fg(Color::White),
    )))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray).bg(Color::Rgb(40, 40, 40)))
            .style(Style::default().bg(Color::Rgb(40, 40, 40))),
    );

    frame.render_widget(tooltip, Rect::new(x, y, width, 3));
}

// ─── Status bar ─────────────────────────────────────────────────────────────

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
        Span::styled(format!("🐑 {alive}"), Style::default().fg(Color::Green)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("🪦 {dead}"), Style::default().fg(Color::Gray)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Tab] Switch", Style::default().fg(Color::DarkGray)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("[q] Quit", Style::default().fg(Color::DarkGray)),
    ]);

    frame.render_widget(Paragraph::new(status), area);
}
