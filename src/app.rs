use std::sync::mpsc;

use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use rand::Rng;

use crate::herdr::{HerdrEvent, SnapshotAgent};
use crate::mock;
use crate::model::farm::Farm;
use crate::model::sheep::{Direction, Sheep, SheepState};
use crate::storage;
use crate::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Farm,
    Log,
}

pub struct App {
    pub screen: Screen,
    pub farm: Farm,
    pub tick_count: u64,
    pub selected_sheep: Option<usize>,
    pub log_scroll: u16,
    pub log_filter: LogFilter,
    pub herdr_rx: Option<mpsc::Receiver<HerdrEvent>>,
    pub connected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFilter {
    All,
    Alive,
    Dead,
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
            for sheep in history {
                if !sheep.is_alive() {
                    farm.sheep.push(sheep);
                }
            }
        }

        Self {
            screen: Screen::Farm,
            farm,
            tick_count: 0,
            selected_sheep: None,
            log_scroll: 0,
            log_filter: LogFilter::All,
            herdr_rx,
            connected,
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
        self.process_herdr_events();
        self.farm.tick();
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
                        if !active_panes.contains(&sheep.id) {
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
        let exists = self.farm.sheep.iter().any(|s| s.id == agent.pane_id && s.is_alive());
        if exists {
            if let Some(sheep) = self
                .farm
                .sheep
                .iter_mut()
                .find(|s| s.id == agent.pane_id && s.is_alive())
            {
                sheep.state = map_agent_status(&agent.agent_status);
            }
            return;
        }

        let mut rng = rand::thread_rng();
        let w = self.farm.width as f32;
        let h = self.farm.height as f32;
        let sprite_w = crate::animation::sprites::SPRITE_CHAR_WIDTH as f32;
        let sprite_h = crate::animation::sprites::SPRITE_CHAR_HEIGHT as f32;

        let (x, y) = self.find_free_spawn(&mut rng, w, h, sprite_w, sprite_h);

        let project = agent
            .cwd
            .as_ref()
            .and_then(|p| p.split('/').last().map(String::from))
            .unwrap_or_else(|| agent.workspace_id.clone());

        let name = project.clone();

        let sheep = Sheep {
            id: agent.pane_id.clone(),
            name,
            born: Utc::now(),
            died: None,
            project,
            tasks_completed: 0,
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

    fn find_free_spawn(
        &self,
        rng: &mut impl rand::Rng,
        w: f32,
        h: f32,
        sprite_w: f32,
        sprite_h: f32,
    ) -> (f32, f32) {
        let max_x = (w - sprite_w - 2.0).max(3.0);
        let max_y = (h - sprite_h - 1.0).max(3.0);

        for _ in 0..50 {
            let x = rng.gen_range(3.0..max_x);
            let y = rng.gen_range(3.0..max_y);

            let collides = self.farm.sheep.iter().filter(|s| s.is_alive()).any(|s| {
                x < s.x + sprite_w
                    && x + sprite_w > s.x
                    && y < s.y + sprite_h
                    && y + sprite_h > s.y
            });

            if !collides {
                return (x, y);
            }
        }

        // Fallback: allow overlap in extreme cases
        (rng.gen_range(3.0..max_x), rng.gen_range(3.0..max_y))
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
        match mouse.kind {
            MouseEventKind::Down(_) => {
                let (col, row) = (mouse.column, mouse.row);
                let offset_x = 1;
                let offset_y = ui::TAB_HEIGHT + 1;
                self.selected_sheep = self.farm.sheep_at(col, row, offset_x, offset_y);
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
                let header_offset = (ui::TAB_HEIGHT + 3) as usize;
                if row >= header_offset {
                    let index = (row - header_offset) + self.log_scroll as usize;
                    if index < self.farm.sheep.len() {
                        self.selected_sheep = Some(index);
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
