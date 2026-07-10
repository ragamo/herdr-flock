use rand::Rng;

use super::sheep::{Direction, Sheep, SheepState};
use crate::animation::sprites::{SPRITE_CHAR_HEIGHT, SPRITE_CHAR_WIDTH};

pub struct Farm {
    pub sheep: Vec<Sheep>,
    pub width: u16,
    pub height: u16,
    pub live_mode: bool,
}

impl Farm {
    pub fn new(width: u16, height: u16, live_mode: bool) -> Self {
        Self {
            sheep: Vec::new(),
            width,
            height,
            live_mode,
        }
    }

    pub fn tick(&mut self) {
        let mut rng = rand::thread_rng();
        let w = self.width as f32;
        let h = self.height as f32;
        let margin_x = SPRITE_CHAR_WIDTH as f32 + 2.0;
        let margin_y = SPRITE_CHAR_HEIGHT as f32 + 1.0;
        let sprite_w = SPRITE_CHAR_WIDTH as f32;
        let sprite_h = SPRITE_CHAR_HEIGHT as f32;

        let len = self.sheep.len();
        for i in 0..len {
            if !self.sheep[i].is_alive() {
                continue;
            }

            self.sheep[i].anim_tick = self.sheep[i].anim_tick.wrapping_add(1);
            self.sheep[i].state_timer = self.sheep[i].state_timer.saturating_sub(1);

            if !self.live_mode && self.sheep[i].state_timer == 0 {
                let new_state = match rng.gen_range(0..10) {
                    0..=5 => SheepState::Idle,
                    6..=7 => SheepState::Eating,
                    8 => SheepState::Sleeping,
                    _ => self.sheep[i].state,
                };
                self.sheep[i].state = new_state;
                self.sheep[i].state_timer = rng.gen_range(60..300);

                if new_state == SheepState::Idle {
                    self.sheep[i].target_x = rng.gen_range(2.0..w - margin_x);
                    self.sheep[i].target_y = rng.gen_range(2.0..h - margin_y);
                }
            }

            if self.live_mode && self.sheep[i].state_timer == 0 {
                self.sheep[i].target_x = rng.gen_range(2.0..w - margin_x);
                self.sheep[i].target_y = rng.gen_range(2.0..h - margin_y);
                self.sheep[i].state_timer = rng.gen_range(80..200);
            }

            let state = self.sheep[i].state;
            if state == SheepState::Idle || state == SheepState::Working {
                let dx = self.sheep[i].target_x - self.sheep[i].x;
                let dy = self.sheep[i].target_y - self.sheep[i].y;
                let speed = 0.08;

                let mut new_x = self.sheep[i].x;
                let mut new_y = self.sheep[i].y;

                if dx.abs() > 0.5 {
                    new_x += dx.signum() * speed;
                }
                if dy.abs() > 0.5 {
                    new_y += dy.signum() * speed;
                }

                new_x = new_x.clamp(1.0, w - margin_x);
                new_y = new_y.clamp(1.0, h - margin_y);

                let collides = (0..len).any(|j| {
                    if j == i || !self.sheep[j].is_alive() {
                        return false;
                    }
                    let ox = self.sheep[j].x;
                    let oy = self.sheep[j].y;
                    new_x < ox + sprite_w
                        && new_x + sprite_w > ox
                        && new_y < oy + sprite_h
                        && new_y + sprite_h > oy
                });

                if !collides {
                    if (new_x - self.sheep[i].x).abs() > 0.001 {
                        self.sheep[i].direction = if new_x > self.sheep[i].x {
                            Direction::Right
                        } else {
                            Direction::Left
                        };
                    } else if (new_y - self.sheep[i].y).abs() > 0.001 {
                        self.sheep[i].direction = if new_y > self.sheep[i].y {
                            Direction::Down
                        } else {
                            Direction::Up
                        };
                    }
                    self.sheep[i].x = new_x;
                    self.sheep[i].y = new_y;
                } else {
                    self.sheep[i].target_x = self.sheep[i].x;
                    self.sheep[i].target_y = self.sheep[i].y;
                }
            }

            self.sheep[i].x = self.sheep[i].x.clamp(1.0, w - margin_x);
            self.sheep[i].y = self.sheep[i].y.clamp(1.0, h - margin_y);

            if self.sheep[i].anim_tick % 20 == 0 {
                self.sheep[i].anim_frame = (self.sheep[i].anim_frame + 1) % 4;
            }
        }
    }

    pub fn sheep_at(&self, col: u16, row: u16, offset_x: u16, offset_y: u16) -> Option<usize> {
        self.sheep
            .iter()
            .enumerate()
            .filter(|(_, s)| s.is_alive())
            .find(|(_, s)| {
                let sc = offset_x + s.display_col();
                let sr = offset_y + s.display_row();
                col >= sc
                    && col < sc + SPRITE_CHAR_WIDTH
                    && row >= sr
                    && row < sr + SPRITE_CHAR_HEIGHT
            })
            .map(|(i, _)| i)
    }
}
