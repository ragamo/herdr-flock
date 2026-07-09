use rand::Rng;

use super::sheep::{Direction, Sheep, SheepState};
use crate::animation::sprites::{SPRITE_CHAR_HEIGHT, SPRITE_CHAR_WIDTH};

pub struct Farm {
    pub sheep: Vec<Sheep>,
    pub width: u16,
    pub height: u16,
}

impl Farm {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            sheep: Vec::new(),
            width,
            height,
        }
    }

    pub fn tick(&mut self) {
        let mut rng = rand::thread_rng();
        let w = self.width as f32;
        let h = self.height as f32;
        let margin_x = SPRITE_CHAR_WIDTH as f32 + 2.0;
        let margin_y = SPRITE_CHAR_HEIGHT as f32 + 1.0;

        for sheep in self.sheep.iter_mut().filter(|s| s.is_alive()) {
            sheep.anim_tick = sheep.anim_tick.wrapping_add(1);
            sheep.state_timer = sheep.state_timer.saturating_sub(1);

            if sheep.state_timer == 0 {
                let new_state = match rng.gen_range(0..10) {
                    0..=5 => SheepState::Idle,
                    6..=7 => SheepState::Eating,
                    8 => SheepState::Sleeping,
                    _ => sheep.state,
                };
                sheep.state = new_state;
                sheep.state_timer = rng.gen_range(60..300);

                if new_state == SheepState::Idle {
                    sheep.target_x = rng.gen_range(2.0..w - margin_x);
                    sheep.target_y = rng.gen_range(2.0..h - margin_y);
                }
            }

            if sheep.state == SheepState::Idle || sheep.state == SheepState::Working {
                let dx = sheep.target_x - sheep.x;
                let dy = sheep.target_y - sheep.y;
                let speed = 0.08;

                if dx.abs() > 0.5 {
                    sheep.x += dx.signum() * speed;
                    sheep.direction = if dx > 0.0 {
                        Direction::Right
                    } else {
                        Direction::Left
                    };
                }
                if dy.abs() > 0.5 {
                    sheep.y += dy.signum() * speed;
                    sheep.direction = if dy > 0.0 {
                        Direction::Down
                    } else {
                        Direction::Up
                    };
                }
            }

            sheep.x = sheep.x.clamp(1.0, w - margin_x);
            sheep.y = sheep.y.clamp(1.0, h - margin_y);

            if sheep.anim_tick % 20 == 0 {
                sheep.anim_frame = (sheep.anim_frame + 1) % 4;
            }
        }
    }

    pub fn sheep_at(&self, col: u16, row: u16) -> Option<usize> {
        self.sheep
            .iter()
            .enumerate()
            .filter(|(_, s)| s.is_alive())
            .find(|(_, s)| {
                let sc = s.display_col();
                let sr = s.display_row();
                col >= sc
                    && col < sc + SPRITE_CHAR_WIDTH
                    && row >= sr
                    && row < sr + SPRITE_CHAR_HEIGHT
            })
            .map(|(i, _)| i)
    }
}
