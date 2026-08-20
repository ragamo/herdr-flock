use std::sync::mpsc;

use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use rand::Rng;

use crate::herdr::{HerdrEvent, SnapshotAgent};
use crate::mock;
use crate::model::farm::Farm;
use crate::model::sheep::{Direction, Sheep, SheepState};
use crate::storage;
use crate::ui;

// ─── Atmosphere ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherKind {
    Rain,
    Snow,
}

pub enum WeatherPhase {
    Clear   { countdown: u32 },
    FadeIn  { kind: WeatherKind, remaining: u32 },
    Active  { kind: WeatherKind, remaining: u32 },
    FadeOut { kind: WeatherKind, remaining: u32 },
}

pub struct Particle {
    pub x: f32,
    pub y: f32,
}

pub struct Atmosphere {
    pub time_of_day: u64,
    pub phase: WeatherPhase,
    pub precipitation: Vec<Particle>,
    pub fireflies: Vec<Firefly>,
    pub rainbow_timer: u32,
    pub wind_angle: f32,
}

pub struct Firefly {
    pub x: f32,
    pub y: f32,
    pub phase: f32,
}

const DAY_CYCLE: u64 = 2400;
const FADE_TICKS: u32 = 50;

impl Atmosphere {
    pub fn new() -> Self {
        Self {
            time_of_day: 0,
            phase: WeatherPhase::Clear { countdown: 800 },
            precipitation: Vec::new(),
            fireflies: Vec::new(),
            rainbow_timer: 0,
            wind_angle: 0.0,
        }
    }

    /// 0.0 = full day, 1.0 = full night
    pub fn night_factor(&self) -> f32 {
        let t = self.time_of_day % DAY_CYCLE;
        match t {
            0..=199 => 0.0,
            200..=1199 => 0.0,
            1200..=1399 => (t - 1200) as f32 / 200.0,
            1400..=2399 => 1.0,
            _ => 0.0,
        }
    }

    /// 0.0 = no precipitation, 1.0 = full
    pub fn weather_intensity(&self) -> f32 {
        match &self.phase {
            WeatherPhase::Clear { .. } => 0.0,
            WeatherPhase::FadeIn  { remaining, .. } => 1.0 - *remaining as f32 / FADE_TICKS as f32,
            WeatherPhase::Active  { .. } => 1.0,
            WeatherPhase::FadeOut { remaining, .. } => *remaining as f32 / FADE_TICKS as f32,
        }
    }

    pub fn time_label(&self) -> &'static str {
        match self.time_of_day % DAY_CYCLE {
            0..=199   => "🌅 Dawn",
            200..=599 => "🌤 Morning",
            600..=1099 => "☀ Midday",
            1100..=1199 => "🌇 Afternoon",
            1200..=1399 => "🌆 Dusk",
            1400..=1799 => "🌙 Evening",
            1800..=2399 => "✦ Night",
            _           => "✦ Night",
        }
    }

    pub fn weather_label(&self) -> &'static str {
        match &self.phase {
            WeatherPhase::Clear { .. }                        => "☁ Clear",
            WeatherPhase::FadeIn  { kind: WeatherKind::Rain, .. } |
            WeatherPhase::Active  { kind: WeatherKind::Rain, .. } |
            WeatherPhase::FadeOut { kind: WeatherKind::Rain, .. } => "🌧 Rain",
            WeatherPhase::FadeIn  { kind: WeatherKind::Snow, .. } |
            WeatherPhase::Active  { kind: WeatherKind::Snow, .. } |
            WeatherPhase::FadeOut { kind: WeatherKind::Snow, .. } => "❄ Snow",
        }
    }

    pub fn active_weather(&self) -> Option<WeatherKind> {
        match &self.phase {
            WeatherPhase::FadeIn  { kind, .. } |
            WeatherPhase::Active  { kind, .. } |
            WeatherPhase::FadeOut { kind, .. } => Some(*kind),
            WeatherPhase::Clear { .. } => None,
        }
    }

    pub fn advance(&mut self, farm_w: u16, farm_h: u16, rng: &mut impl Rng) {
        self.time_of_day = (self.time_of_day + 1) % DAY_CYCLE;

        let w = farm_w as f32;
        let h = farm_h as f32;

        // Advance weather state machine
        let next_phase = match &mut self.phase {
            WeatherPhase::Clear { countdown } => {
                if *countdown == 0 {
                    let roll = rng.gen_range(0..10);
                    if roll < 2 {
                        let kind = WeatherKind::Rain;
                        self.seed_precipitation(kind, w, h, rng);
                        Some(WeatherPhase::FadeIn { kind, remaining: FADE_TICKS })
                    } else if roll < 3 {
                        let kind = WeatherKind::Snow;
                        self.seed_precipitation(kind, w, h, rng);
                        Some(WeatherPhase::FadeIn { kind, remaining: FADE_TICKS })
                    } else {
                        *countdown = rng.gen_range(600..1200);
                        None
                    }
                } else {
                    *countdown -= 1;
                    None
                }
            }
            WeatherPhase::FadeIn { kind, remaining } => {
                if *remaining == 0 {
                    let duration = rng.gen_range(300..=600u32);
                    Some(WeatherPhase::Active { kind: *kind, remaining: duration })
                } else {
                    *remaining -= 1;
                    None
                }
            }
            WeatherPhase::Active { kind, remaining } => {
                if *remaining == 0 {
                    Some(WeatherPhase::FadeOut { kind: *kind, remaining: FADE_TICKS })
                } else {
                    *remaining -= 1;
                    None
                }
            }
            WeatherPhase::FadeOut { remaining, .. } => {
                if *remaining == 0 {
                    self.precipitation.clear();
                    Some(WeatherPhase::Clear { countdown: rng.gen_range(600..1200) })
                } else {
                    *remaining -= 1;
                    None
                }
            }
        };

        if let Some(p) = next_phase {
            // Trigger rainbow when rain ends
            if matches!(&self.phase, WeatherPhase::FadeOut { kind: WeatherKind::Rain, .. })
                && matches!(&p, WeatherPhase::Clear { .. })
            {
                self.rainbow_timer = 80;
            }
            self.phase = p;
        }

        // Rainbow countdown
        self.rainbow_timer = self.rainbow_timer.saturating_sub(1);

        // Wind angle drifts slowly
        let tod = self.time_of_day as f32;
        self.wind_angle = (tod * 0.005).sin() * 0.6;

        // Fireflies at night
        self.update_fireflies(w, h, rng);

        // Move particles
        self.move_particles(w, h);
    }

    fn seed_precipitation(&mut self, _kind: WeatherKind, w: f32, h: f32, rng: &mut impl Rng) {
        self.precipitation.clear();
        let count = ((w * h) / 80.0) as usize;
        for _ in 0..count {
            self.precipitation.push(Particle {
                x: rng.gen_range(0.0..w),
                y: rng.gen_range(0.0..h),
            });
        }
    }

    fn move_particles(&mut self, w: f32, h: f32) {
        let weather = self.active_weather();
        let wind = self.wind_angle;
        for p in &mut self.precipitation {
            match weather {
                Some(WeatherKind::Rain) => {
                    p.y += 0.8;
                    p.x += wind;
                    if p.y >= h { p.y = 0.0; }
                    if p.x < 0.0 { p.x += w; }
                    if p.x >= w { p.x -= w; }
                }
                Some(WeatherKind::Snow) => {
                    p.y += 0.3;
                    p.x += wind * 0.5 + (p.y * 0.05).sin() * 0.3;
                    if p.y >= h { p.y = 0.0; }
                    if p.x < 0.0 { p.x += w; }
                    if p.x >= w { p.x -= w; }
                }
                None => {}
            }
        }
    }

    fn update_fireflies(&mut self, w: f32, h: f32, rng: &mut impl Rng) {
        let night = self.night_factor();
        let target_count = if night > 0.5 {
            ((w * h) / 800.0 * (night - 0.5) * 2.0) as usize
        } else {
            0
        };

        // Spawn/despawn to match target
        while self.fireflies.len() < target_count {
            self.fireflies.push(Firefly {
                x: rng.gen_range(2.0..w - 2.0),
                y: rng.gen_range(2.0..h - 2.0),
                phase: rng.gen_range(0.0..std::f32::consts::TAU),
            });
        }
        self.fireflies.truncate(target_count);

        // Move fireflies gently
        for f in &mut self.fireflies {
            f.phase += 0.12;
            f.x += (f.phase).sin() * 0.15;
            f.y += (f.phase * 0.7).cos() * 0.1;
            f.x = f.x.clamp(1.0, w - 2.0);
            f.y = f.y.clamp(1.0, h - 2.0);
        }
    }
}

// ─── Screen / Filter ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Farm,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFilter {
    All,
    Alive,
    Dead,
}

#[derive(Debug, Clone, Copy)]
pub struct DragState {
    pub sheep_idx: usize,
    pub grab_offset_x: f32,
    pub grab_offset_y: f32,
}

// ─── App ────────────────────────────────────────────────────────────────────

pub struct App {
    pub screen: Screen,
    pub farm: Farm,
    pub tick_count: u64,
    pub selected_sheep: Option<usize>,
    pub drag: Option<DragState>,
    pub log_scroll: u16,
    pub log_filter: LogFilter,
    pub herdr_rx: Option<mpsc::Receiver<HerdrEvent>>,
    pub connected: bool,
    pub atmosphere: Atmosphere,
}

impl App {
    pub fn new(herdr_rx: Option<mpsc::Receiver<HerdrEvent>>) -> Self {
        let connected = herdr_rx.is_some();
        let mut farm = if connected {
            Farm::new(100, 40, true)
        } else {
            mock::create_mock_farm()
        };

        if connected {
            let history = storage::load_flock();
            let mut rng = rand::thread_rng();
            let w = farm.width as f32;
            let h = farm.height as f32;
            let sprite_w = crate::animation::sprites::SPRITE_CHAR_WIDTH as f32;
            let sprite_h = crate::animation::sprites::SPRITE_CHAR_HEIGHT as f32;

            for mut sheep in history {
                let (x, y) = find_free_spawn_in(&farm, &mut rng, w, h, sprite_w, sprite_h);
                sheep.x = x;
                sheep.y = y;
                sheep.target_x = x;
                sheep.target_y = y;
                farm.sheep.push(sheep);
            }
        }

        Self {
            screen: Screen::Farm,
            farm,
            tick_count: 0,
            selected_sheep: None,
            drag: None,
            log_scroll: 0,
            log_filter: LogFilter::All,
            herdr_rx,
            connected,
            atmosphere: Atmosphere::new(),
        }
    }

    pub fn save(&self) {
        if self.connected {
            let _ = storage::save_flock(&self.farm.sheep);
        }
    }

    pub fn toggle_screen(&mut self) {
        self.screen = match self.screen {
            Screen::Farm => Screen::Log,
            Screen::Log => Screen::Farm,
        };
        self.selected_sheep = None;
        self.drag = None;
    }

    pub fn handle_key(&mut self, key: &KeyEvent) {
        match self.screen {
            Screen::Farm => self.handle_farm_key(key),
            Screen::Log => self.handle_log_key(key),
        }
    }

    pub fn handle_mouse(&mut self, mouse: &MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(_) => {
                if mouse.row < ui::TAB_HEIGHT {
                    if let Some(screen) = ui::tab_click(mouse.column) {
                        if self.screen != screen {
                            self.screen = screen;
                            self.selected_sheep = None;
                            self.drag = None;
                        }
                    }
                    return;
                }
            }
            _ => {}
        }

        match self.screen {
            Screen::Farm => self.handle_farm_mouse(mouse),
            Screen::Log => self.handle_log_mouse(mouse),
        }
    }

    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
        self.tick_atmosphere();
        self.process_herdr_events();
        let dragged_idx = self.drag.map(|d| d.sheep_idx);
        self.farm.tick(dragged_idx);
    }

    fn tick_atmosphere(&mut self) {
        let w = self.farm.width;
        let h = self.farm.height;
        let mut rng = rand::thread_rng();
        self.atmosphere.advance(w, h, &mut rng);
    }

    fn process_herdr_events(&mut self) {
        let events: Vec<HerdrEvent> = match &self.herdr_rx {
            Some(rx) => rx.try_iter().collect(),
            None => return,
        };

        if events.is_empty() {
            return;
        }

        for event in events {
            match event {
                HerdrEvent::AgentList { agents } => {
                    for agent in &agents {
                        self.upsert_sheep_from_agent(agent);
                    }
                    let active_panes: Vec<String> =
                        agents.iter().map(|a| a.pane_id.clone()).collect();
                    for sheep in self.farm.sheep.iter_mut().filter(|s| s.is_alive()) {
                        if !active_panes.contains(&sheep.pane_id) {
                            sheep.died = Some(Utc::now());
                            sheep.state = SheepState::Sleeping;
                        }
                    }
                }
            }
        }

        self.save();
    }

    fn upsert_sheep_from_agent(&mut self, agent: &SnapshotAgent) {
        if let Some(sheep) = self
            .farm
            .sheep
            .iter_mut()
            .find(|s| s.is_alive() && s.pane_id == agent.pane_id)
        {
            sheep.state = map_agent_status(&agent.agent_status);
            return;
        }

        let mut rng = rand::thread_rng();
        let (x, y) = self.find_free_spawn_pos(&mut rng);

        let project = agent
            .cwd
            .as_ref()
            .and_then(|p| p.split('/').last().map(String::from))
            .unwrap_or_else(|| agent.workspace_id.clone());

        let name = random_sheep_name(&mut rng);

        let sheep = Sheep {
            id: format!("{}:{}", agent.pane_id, name),
            pane_id: agent.pane_id.clone(),
            name,
            born: Utc::now(),
            died: None,
            project,
            agent: agent.agent.clone().unwrap_or_else(|| "claude".to_string()),
            state: map_agent_status(&agent.agent_status),
            direction: match rng.gen_range(0..4) {
                0 => Direction::Up,
                1 => Direction::Down,
                2 => Direction::Left,
                _ => Direction::Right,
            },
            x,
            y,
            target_x: x,
            target_y: y,
            anim_frame: 0,
            anim_tick: 0,
            state_timer: rng.gen_range(60..200),
        };

        self.farm.sheep.push(sheep);
    }

    fn find_free_spawn_pos(&self, rng: &mut impl rand::Rng) -> (f32, f32) {
        let w = self.farm.width as f32;
        let h = self.farm.height as f32;
        let sprite_w = crate::animation::sprites::SPRITE_CHAR_WIDTH as f32;
        let sprite_h = crate::animation::sprites::SPRITE_CHAR_HEIGHT as f32;
        find_free_spawn_in(&self.farm, rng, w, h, sprite_w, sprite_h)
    }

    fn handle_farm_key(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Esc => self.selected_sheep = None,
            _ => {}
        }
    }

    fn handle_log_key(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Up => self.log_scroll = self.log_scroll.saturating_sub(1),
            KeyCode::Down => self.log_scroll = self.log_scroll.saturating_add(1),
            KeyCode::Char('f') => {
                self.log_filter = match self.log_filter {
                    LogFilter::All => LogFilter::Alive,
                    LogFilter::Alive => LogFilter::Dead,
                    LogFilter::Dead => LogFilter::All,
                };
            }
            _ => {}
        }
    }

    fn handle_farm_mouse(&mut self, mouse: &MouseEvent) {
        let (col, row) = (mouse.column, mouse.row);
        let offset_x = 1;
        let offset_y = ui::TAB_HEIGHT + 1;

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(idx) = self.farm.sheep_at(col, row, offset_x, offset_y) {
                    self.selected_sheep = Some(idx);
                    let sheep = &self.farm.sheep[idx];
                    let grab_offset_x = (col as f32) - (offset_x as f32) - sheep.x;
                    let grab_offset_y = (row as f32) - (offset_y as f32) - sheep.y;
                    self.drag = Some(DragState {
                        sheep_idx: idx,
                        grab_offset_x,
                        grab_offset_y,
                    });
                } else {
                    self.selected_sheep = None;
                    self.drag = None;
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if let Some(drag) = self.drag {
                    if drag.sheep_idx < self.farm.sheep.len() && self.farm.sheep[drag.sheep_idx].is_alive() {
                        let margin_x = crate::animation::sprites::SPRITE_CHAR_WIDTH as f32 + 2.0;
                        let margin_y = crate::animation::sprites::SPRITE_CHAR_HEIGHT as f32 + 1.0;
                        let max_x = (self.farm.width as f32 - margin_x).max(1.0);
                        let max_y = (self.farm.height as f32 - margin_y).max(1.0);

                        let new_x = ((col as f32) - (offset_x as f32) - drag.grab_offset_x).clamp(1.0, max_x);
                        let new_y = ((row as f32) - (offset_y as f32) - drag.grab_offset_y).clamp(1.0, max_y);

                        let sheep = &mut self.farm.sheep[drag.sheep_idx];
                        if (new_x - sheep.x).abs() > 0.1 {
                            sheep.direction = if new_x > sheep.x {
                                Direction::Right
                            } else {
                                Direction::Left
                            };
                        } else if (new_y - sheep.y).abs() > 0.1 {
                            sheep.direction = if new_y > sheep.y {
                                Direction::Down
                            } else {
                                Direction::Up
                            };
                        }

                        sheep.x = new_x;
                        sheep.y = new_y;
                        sheep.target_x = new_x;
                        sheep.target_y = new_y;
                    }
                }
            }
            MouseEventKind::Up(_) => {
                self.drag = None;
            }
            _ => {}
        }
    }

    fn handle_log_mouse(&mut self, mouse: &MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => self.log_scroll = self.log_scroll.saturating_sub(3),
            MouseEventKind::ScrollDown => self.log_scroll = self.log_scroll.saturating_add(3),
            MouseEventKind::Down(_) => {
                let row = mouse.row as usize;
                let header_offset = (ui::TAB_HEIGHT + 5) as usize;
                if row >= header_offset {
                    let display_row = (row - header_offset) + self.log_scroll as usize;
                    let indices = ui::log::sorted_log_indices(&self.farm, self.log_filter);
                    if let Some(maybe_idx) = indices.get(display_row) {
                        self.selected_sheep = *maybe_idx;
                    }
                }
            }
            _ => {}
        }
    }
}

fn map_agent_status(status: &str) -> SheepState {
    match status {
        "working" => SheepState::Working,
        "blocked" => SheepState::Eating,
        "done" => SheepState::Sleeping,
        _ => SheepState::Idle,
    }
}

pub fn find_free_spawn_in(
    farm: &Farm,
    rng: &mut impl rand::Rng,
    w: f32,
    h: f32,
    sprite_w: f32,
    sprite_h: f32,
) -> (f32, f32) {
    let max_x = (w - sprite_w - 2.0).max(4.0);
    let max_y = (h - sprite_h - 1.0).max(4.0);

    for _ in 0..100 {
        let x = rng.gen_range(3.0..max_x);
        let y = rng.gen_range(3.0..max_y);

        let collides_sheep = farm.sheep.iter().filter(|s| s.is_alive()).any(|s| {
            x < s.x + sprite_w + 1.0
                && x + sprite_w + 1.0 > s.x
                && y < s.y + sprite_h
                && y + sprite_h > s.y
        });

        if !collides_sheep {
            return (x, y);
        }
    }

    (rng.gen_range(3.0..max_x), rng.gen_range(3.0..max_y))
}

const SHEEP_NAMES: &[&str] = &[
    "Dolly", "Woolma", "Clover", "Nimbus", "Cotton",
    "Patches", "Fleecy", "Misty", "Pepper", "Daisy",
    "Luna", "Maple", "Olive", "Hazel", "Sunny",
    "Pebble", "Willow", "Biscuit", "Mochi", "Truffle",
    "Sage", "Cinnamon", "Nutmeg", "Cocoa", "Velvet",
    "Pearl", "Ember", "Frost", "Bramble", "Thistle",
    "Poppy", "Fern", "Ivy", "Basil", "Rosie",
    "Ginger", "Toffee", "Pudding", "Crumble", "Scone",
    "Baa-rbara", "Shearlock", "Lambchop", "Wooly",
    "Churro", "Marshmallow", "Puffin", "Snowball",
    "Buttercup", "Honey", "Caramel", "Waffles",
    "Nube", "Algodón", "Canela", "Merino",
    "Cloud", "Stormy", "Thunder", "Breeze",
    "Cashew", "Pretzel", "Dumpling", "Tofu",
    "Muffin", "Cookie", "Brownie", "Meringue",
    "Orbit", "Comet", "Nova", "Pixel",
];

pub fn random_sheep_name(rng: &mut impl rand::Rng) -> String {
    SHEEP_NAMES[rng.gen_range(0..SHEEP_NAMES.len())].to_string()
}
