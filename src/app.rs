use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind, KeyCode};

use crate::model::farm::Farm;
use crate::mock;
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFilter {
    All,
    Alive,
    Dead,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Farm,
            farm: mock::create_mock_farm(),
            tick_count: 0,
            selected_sheep: None,
            log_scroll: 0,
            log_filter: LogFilter::All,
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
        self.farm.tick();
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
