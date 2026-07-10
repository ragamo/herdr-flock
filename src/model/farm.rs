use rand::Rng;

use super::sheep::{Direction, Sheep, SheepState};
use crate::animation::sprites::{SPRITE_CHAR_HEIGHT, SPRITE_CHAR_WIDTH};

pub struct Farm {
    pub sheep: Vec<Sheep>,
    pub width: u16,
    pub height: u16,
    pub live_mode: bool,
    pub trees: Vec<(u16, u16)>,
    pub river_row: u16,
}

impl Farm {
    pub fn new(width: u16, height: u16, live_mode: bool) -> Self {
        let river_row = compute_river_row(height);
        let trees = generate_trees(width, height, river_row);
        Self {
            sheep: Vec::new(),
            width,
            height,
            live_mode,
            trees,
            river_row,
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.river_row = compute_river_row(height);
        self.trees = generate_trees(width, height, self.river_row);

        let margin_x = SPRITE_CHAR_WIDTH as f32 + 2.0;
        let margin_y = SPRITE_CHAR_HEIGHT as f32 + 1.0;
        let max_x = (width as f32 - margin_x).max(1.0);
        let max_y = (height as f32 - margin_y).max(1.0);

        for sheep in self.sheep.iter_mut().filter(|s| s.is_alive()) {
            if sheep.x > max_x || sheep.y > max_y {
                sheep.x = sheep.x.clamp(1.0, max_x);
                sheep.y = sheep.y.clamp(1.0, max_y);
                sheep.target_x = sheep.x;
                sheep.target_y = sheep.y;
            }
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
                let base_state = self.sheep[i].state;
                if base_state == SheepState::Idle {
                    self.sheep[i].state = match rng.gen_range(0..6) {
                        0 => SheepState::Eating,
                        1 => SheepState::Sleeping,
                        _ => SheepState::Idle,
                    };
                }
                self.sheep[i].target_x = rng.gen_range(2.0..w - margin_x);
                self.sheep[i].target_y = rng.gen_range(2.0..h - margin_y);
                self.sheep[i].state_timer = rng.gen_range(80..200);
                self.sheep[i].direction = match rng.gen_range(0..4) {
                    0 => Direction::Up,
                    1 => Direction::Down,
                    2 => Direction::Left,
                    _ => Direction::Right,
                };
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

                let collides_sheep = (0..len).any(|j| {
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

                let collides_tree = self.trees.iter().any(|&(tc, tr)| {
                    new_x < tc as f32 + 6.0
                        && new_x + sprite_w > tc as f32 - 1.0
                        && new_y < tr as f32 + 6.0
                        && new_y + sprite_h > tr as f32
                });

                let collides = collides_sheep || collides_tree;

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

pub fn compute_river_row(height: u16) -> u16 {
    (height / 3).clamp(4, height.saturating_sub(6))
}

pub fn generate_trees(width: u16, height: u16, river_row: u16) -> Vec<(u16, u16)> {
    if width < 20 || height < 14 {
        return Vec::new();
    }

    let target = ((width as u32 * height as u32) / 700).min(10) as usize;
    let mut trees: Vec<(u16, u16)> = Vec::with_capacity(target);

    let mut seed: u64 = (width as u64)
        .wrapping_mul(48271)
        .wrapping_add((height as u64).wrapping_mul(16807));

    let lcg = |s: u64| -> u64 {
        s.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407)
    };

    let col_min: u16 = 3;
    let col_max: u16 = width.saturating_sub(4);
    let row_min: u16 = 3;
    let row_max: u16 = height.saturating_sub(4);
    let spawn_zone_max_col = width / 3;

    let mut attempts = 0usize;
    while trees.len() < target && attempts < target * 40 {
        attempts += 1;
        seed = lcg(seed);
        let col = col_min + ((seed >> 33) as u16 % (col_max - col_min + 1));
        seed = lcg(seed);
        let row = row_min + ((seed >> 33) as u16 % (row_max - row_min + 1));

        // Exclusion: left-third spawn zone
        if col < spawn_zone_max_col + 2 {
            continue;
        }
        // Exclusion: river band ±1
        if row >= river_row.saturating_sub(1) && row <= river_row + 3 {
            continue;
        }
        // Exclusion: too close to another tree
        let too_close = trees.iter().any(|&(tc, tr)| {
            (col as i32 - tc as i32).abs() < 8 && (row as i32 - tr as i32).abs() < 8
        });
        if too_close {
            continue;
        }

        trees.push((col, row));
    }

    trees
}
