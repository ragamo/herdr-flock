use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SheepState {
    Idle,
    Eating,
    Sleeping,
    Working,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sheep {
    pub id: String,
    pub pane_id: String,
    pub name: String,
    pub born: DateTime<Utc>,
    pub died: Option<DateTime<Utc>>,
    pub project: String,
    pub agent: String,
    pub state: SheepState,
    pub direction: Direction,
    #[serde(skip)]
    pub x: f32,
    #[serde(skip)]
    pub y: f32,
    #[serde(skip)]
    pub target_x: f32,
    #[serde(skip)]
    pub target_y: f32,
    #[serde(skip)]
    pub anim_frame: u8,
    #[serde(skip)]
    pub anim_tick: u16,
    #[serde(skip)]
    pub state_timer: u16,
}

impl Sheep {
    pub fn is_alive(&self) -> bool {
        self.died.is_none()
    }

    pub fn display_col(&self) -> u16 {
        self.x as u16
    }

    pub fn display_row(&self) -> u16 {
        self.y as u16
    }
}
