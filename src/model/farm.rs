use rand::Rng;

use super::sheep::{Direction, Sheep, SheepState};
use crate::animation::sprites::{SPRITE_CHAR_HEIGHT, SPRITE_CHAR_WIDTH};

pub struct Farm {
    pub sheep: Vec<Sheep>,
    pub width: u16,
    pub height: u16,
    pub live_mode: bool,
    pub trees: Vec<(u16, u16)>,
    pub rocks: Vec<(u16, u16)>,
    /// Per-column row offset for the river (len == width)
    pub river_path: Vec<u16>,
}

impl Farm {
    pub fn new(width: u16, height: u16, live_mode: bool) -> Self {
        let river_path = generate_river_path(width, height);
        let trees = generate_trees(width, height, &river_path);
        let rocks = generate_rocks(width, height, &river_path, &trees);
        Self {
            sheep: Vec::new(),
            width,
            height,
            live_mode,
            trees,
            rocks,
            river_path,
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.river_path = generate_river_path(width, height);
        self.trees = generate_trees(width, height, &self.river_path);
        self.rocks = generate_rocks(width, height, &self.river_path, &self.trees);

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
                    0..=3 => SheepState::Idle,
                    4..=6 => SheepState::Eating,
                    7..=8 => SheepState::Sleeping,
                    _ => self.sheep[i].state,
                };
                self.sheep[i].state = new_state;
                self.sheep[i].state_timer = match new_state {
                    SheepState::Eating => rng.gen_range(150..400),
                    SheepState::Sleeping => rng.gen_range(200..500),
                    _ => rng.gen_range(60..250),
                };

                if new_state == SheepState::Idle {
                    self.sheep[i].target_x = rng.gen_range(2.0..w - margin_x);
                    self.sheep[i].target_y = rng.gen_range(2.0..h - margin_y);
                }
                // Bias direction toward Up/Down so front/back sprites show more
                self.sheep[i].direction = match rng.gen_range(0..6) {
                    0 | 1 => Direction::Down,
                    2 | 3 => Direction::Up,
                    4 => Direction::Left,
                    _ => Direction::Right,
                };
            }

            if self.live_mode && self.sheep[i].state_timer == 0 {
                let base_state = self.sheep[i].state;
                if base_state == SheepState::Idle {
                    self.sheep[i].state = match rng.gen_range(0..4) {
                        0 | 1 => SheepState::Eating,
                        2 => SheepState::Sleeping,
                        _ => SheepState::Idle,
                    };
                }
                self.sheep[i].target_x = rng.gen_range(2.0..w - margin_x);
                self.sheep[i].target_y = rng.gen_range(2.0..h - margin_y);
                self.sheep[i].state_timer = rng.gen_range(80..200);
                self.sheep[i].direction = match rng.gen_range(0..6) {
                    0 | 1 => Direction::Down,
                    2 | 3 => Direction::Up,
                    4 => Direction::Left,
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

                let collides = collides_sheep;

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

pub fn generate_river_path(width: u16, height: u16) -> Vec<u16> {
    let mut rng = rand::thread_rng();
    let h = height as f32;
    let w = width as f32;
    let min_row = 3.0f32;
    let max_row = (h - 5.0).max(min_row + 1.0);

    // Random start and end rows — river can cross the full height
    let start_row = rng.gen_range(min_row..max_row);
    let end_row   = rng.gen_range(min_row..max_row);

    // Sinusoidal wobble on top of the linear drift
    let amplitude = rng.gen_range(2.0f32..5.0);
    let freq1  = rng.gen_range(0.03f32..0.09);
    let freq2  = rng.gen_range(0.01f32..0.04);
    let phase1 = rng.gen_range(0.0f32..std::f32::consts::TAU);
    let phase2 = rng.gen_range(0.0f32..std::f32::consts::TAU);

    (0..width)
        .map(|col| {
            let t = col as f32 / w.max(1.0);
            let linear = start_row + (end_row - start_row) * t;
            let wobble = amplitude * (freq1 * col as f32 + phase1).sin()
                + (amplitude * 0.4) * (freq2 * col as f32 + phase2).sin();
            (linear + wobble).clamp(min_row, max_row) as u16
        })
        .collect()
}

pub fn generate_trees(width: u16, height: u16, river_path: &[u16]) -> Vec<(u16, u16)> {
    if width < 20 || height < 14 {
        return Vec::new();
    }

    let target = ((width as u32 * height as u32) / 700).min(10) as usize;
    let mut trees: Vec<(u16, u16)> = Vec::with_capacity(target);

    let mut rng = rand::thread_rng();
    let entropy: u64 = rng.gen_range(0..u64::MAX);
    let mut seed: u64 = (width as u64)
        .wrapping_mul(48271)
        .wrapping_add((height as u64).wrapping_mul(16807))
        .wrapping_add(entropy);

    let lcg = |s: u64| -> u64 {
        s.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407)
    };

    let col_min: u16 = 3;
    let col_max: u16 = width.saturating_sub(6);
    let row_min: u16 = 3;
    let row_max: u16 = height.saturating_sub(6);

    let mut attempts = 0usize;
    while trees.len() < target && attempts < target * 40 {
        attempts += 1;
        seed = lcg(seed);
        let col = col_min + ((seed >> 33) as u16 % (col_max - col_min + 1));
        seed = lcg(seed);
        let row = row_min + ((seed >> 33) as u16 % (row_max - row_min + 1));
        // Exclusion: river overlaps any of the 5 cols × 6 rows the tree occupies
        let river_overlap = (0..5u16).any(|dc| {
            let c = col.saturating_add(dc);
            let rr = river_path.get(c as usize).copied().unwrap_or(0);
            row < rr + 3 && row + 6 > rr.saturating_sub(1)
        });
        if river_overlap { continue; }
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

pub fn generate_rocks(width: u16, height: u16, river_path: &[u16], trees: &[(u16, u16)]) -> Vec<(u16, u16)> {
    if width < 20 || height < 14 {
        return Vec::new();
    }

    let target = ((width as u32 * height as u32) / 2000).min(5) as usize;
    let mut rocks: Vec<(u16, u16)> = Vec::with_capacity(target);

    let mut rng = rand::thread_rng();
    let entropy: u64 = rng.gen_range(0..u64::MAX);
    let mut seed: u64 = (width as u64)
        .wrapping_mul(73856093)
        .wrapping_add((height as u64).wrapping_mul(19349663))
        .wrapping_add(entropy);

    let lcg = |s: u64| -> u64 {
        s.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407)
    };

    let col_min: u16 = 3;
    let col_max: u16 = width.saturating_sub(6);
    let row_min: u16 = 3;
    let row_max: u16 = height.saturating_sub(6);

    let mut attempts = 0usize;
    while rocks.len() < target && attempts < target * 40 {
        attempts += 1;
        seed = lcg(seed);
        let col = col_min + ((seed >> 33) as u16 % (col_max - col_min + 1));
        seed = lcg(seed);
        let row = row_min + ((seed >> 33) as u16 % (row_max - row_min + 1));

        // Exclusion: river overlaps any of the 6 cols × 4 rows the rock occupies
        let river_overlap = (0..6u16).any(|dc| {
            let c = col.saturating_add(dc);
            let rr = river_path.get(c as usize).copied().unwrap_or(0);
            row < rr + 3 && row + 4 > rr.saturating_sub(1)
        });
        if river_overlap { continue; }
        // Exclusion: too close to a tree
        let near_tree = trees.iter().any(|&(tc, tr)| {
            (col as i32 - tc as i32).abs() < 6 && (row as i32 - tr as i32).abs() < 7
        });
        if near_tree { continue; }
        // Exclusion: too close to another rock
        let near_rock = rocks.iter().any(|&(rc, rr)| {
            (col as i32 - rc as i32).abs() < 5 && (row as i32 - rr as i32).abs() < 4
        });
        if near_rock { continue; }

        rocks.push((col, row));
    }

    rocks
}
