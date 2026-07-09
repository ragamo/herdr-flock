use crate::model::sheep::{Direction, SheepState};

pub const SPRITE_PX_WIDTH: usize = 11;
pub const SPRITE_PX_HEIGHT: usize = 10;
pub const SPRITE_CHAR_WIDTH: u16 = 11;
pub const SPRITE_CHAR_HEIGHT: u16 = 5;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Px {
    T,
    K,
    W,
    B,
}

pub struct SheepSprite {
    pub rows: &'static [&'static str],
}

impl SheepSprite {
    pub fn pixel_at(&self, row: usize, col: usize) -> Px {
        self.rows
            .get(row)
            .and_then(|r| r.as_bytes().get(col))
            .map(|&b| match b {
                b'#' => Px::K,
                b'W' => Px::W,
                b'B' => Px::B,
                _ => Px::T,
            })
            .unwrap_or(Px::T)
    }
}

pub fn get_sprite(state: SheepState, direction: Direction, frame: u8) -> SheepSprite {
    match state {
        SheepState::Working => idle_sprite(direction, frame),
        SheepState::Sleeping => sleeping_sprite(frame),
        SheepState::Eating => eating_sprite(frame),
        SheepState::Idle => idle_sprite(direction, frame),
    }
}

fn idle_sprite(direction: Direction, frame: u8) -> SheepSprite {
    match direction {
        Direction::Down => {
            if frame % 4 < 2 {
                SheepSprite { rows: &FRONT_A }
            } else {
                SheepSprite { rows: &FRONT_B }
            }
        }
        Direction::Up => {
            if frame % 4 < 2 {
                SheepSprite { rows: &BACK_A }
            } else {
                SheepSprite { rows: &BACK_B }
            }
        }
        Direction::Left => {
            if frame % 4 < 2 {
                SheepSprite { rows: &LEFT_A }
            } else {
                SheepSprite { rows: &LEFT_B }
            }
        }
        Direction::Right => {
            if frame % 4 < 2 {
                SheepSprite { rows: &RIGHT_A }
            } else {
                SheepSprite { rows: &RIGHT_B }
            }
        }
    }
}

fn eating_sprite(frame: u8) -> SheepSprite {
    if frame % 4 < 2 {
        SheepSprite { rows: &EAT_A }
    } else {
        SheepSprite { rows: &EAT_B }
    }
}

fn sleeping_sprite(frame: u8) -> SheepSprite {
    if frame % 4 < 2 {
        SheepSprite { rows: &SLEEP_A }
    } else {
        SheepSprite { rows: &SLEEP_B }
    }
}

// Front facing (Down) - frame A: legs spread
const FRONT_A: [&str; 10] = [
    "...#WWW#...",
    "..#WWWWW#..",
    ".#WW#B#WW#.",
    ".#WWBBBWW#.",
    ".#WWBBBWW#.",
    ".#WWWWWWW#.",
    "..#WWWWW#..",
    "...#WWW#...",
    "..##...##..",
    "..##...##..",
];

// Front facing (Down) - frame B: legs together
const FRONT_B: [&str; 10] = [
    "...#WWW#...",
    "..#WWWWW#..",
    ".#WWWWWWW#.",
    ".#WW#B#WW#.",
    ".#WWBBBWW#.",
    ".#WWBBBWW#.",
    "..#WWWWW#..",
    "...#WWW#...",
    "...##.##...",
    "...##.##...",
];

// Back facing (Up) - frame A
const BACK_A: [&str; 10] = [
    "...........",
    "..##WWW##..",
    ".#WWWWWWW#.",
    ".#WWWWWWW#.",
    ".#WWWWWWW#.",
    ".#WWW#WWW#.",
    ".#WWWWWWW#.",
    "..#WWWWW#..",
    "..##...##..",
    "..##...##..",
];

// Back facing (Up) - frame B
const BACK_B: [&str; 10] = [
    "...........",
    "..##WWW##..",
    ".#WWWWWWW#.",
    ".#WWWWWWW#.",
    ".#WWWWWWW#.",
    ".#WWW#WWW#.",
    ".#WWWWWWW#.",
    "..#WWWWW#..",
    "...##.##...",
    "...##.##...",
];

// Left facing - frame A
const LEFT_A: [&str; 10] = [
    "...##WW##..",
    ".##BBWWWW#.",
    ".#B#BWWWW#.",
    ".##BBWWWW#.",
    "..#WWWWWW#.",
    "..#WWWWWW#.",
    "..#WWWWWW#.",
    "...#WWWW#..",
    "...##..##..",
    "...##..##..",
];

// Left facing - frame B
const LEFT_B: [&str; 10] = [
    "...##WW##..",
    ".##BBWWWW#.",
    ".#B#BWWWW#.",
    ".##BBWWWW#.",
    "..#WWWWWW#.",
    "..#WWWWWW#.",
    "..#WWWWWW#.",
    "...#WWWW#..",
    "..##..##...",
    "..##..##...",
];

// Right facing - frame A
const RIGHT_A: [&str; 10] = [
    "..##WW##...",
    ".#WWWWBB##.",
    ".#WWWWB#B#.",
    ".#WWWWBB##.",
    "..#WWWWWW#.",
    "..#WWWWWW#.",
    "..#WWWWWW#.",
    "...#WWWW#..",
    "..##..##...",
    "..##..##...",
];

// Right facing - frame B
const RIGHT_B: [&str; 10] = [
    "..##WW##...",
    ".#WWWWBB##.",
    ".#WWWWB#B#.",
    ".#WWWWBB##.",
    "..#WWWWWW#.",
    "..#WWWWWW#.",
    "..#WWWWWW#.",
    "...#WWWW#..",
    "...##..##..",
    "...##..##..",
];

// Eating - frame A (head down, facing front)
const EAT_A: [&str; 10] = [
    "..##WWW##..",
    ".#WWWWWWW#.",
    ".#WWWWWWW#.",
    ".#WWWWWWW#.",
    ".#WWWWWWW#.",
    ".#WW#B#WW#.",
    ".#WWBBBWW#.",
    "..#BBBBB#..",
    "..##...##..",
    "..##...##..",
];

// Eating - frame B (head slightly up between bites)
const EAT_B: [&str; 10] = [
    "..##WWW##..",
    ".#WWWWWWW#.",
    ".#WWWWWWW#.",
    ".#WWWWWWW#.",
    ".#WW#B#WW#.",
    ".#WWBBBWW#.",
    ".#WWBBBWW#.",
    "..#WWWWW#..",
    "..##...##..",
    "..##...##..",
];

// Sleeping - frame A (lying down)
const SLEEP_A: [&str; 10] = [
    "...........",
    "...........",
    "...........",
    "...........",
    "..##WWW##..",
    ".#WWWWWWW#.",
    ".#WW~WWWW#.",
    ".#WWWWWWW#.",
    ".##WWWWW##.",
    "..#######..",
];

// Sleeping - frame B
const SLEEP_B: [&str; 10] = [
    "...........",
    "...........",
    "...........",
    "...........",
    "..##WWW##..",
    ".#WWWWWWW#.",
    ".#WW~WWWW#.",
    ".#WWWWWWW#.",
    ".##WWWWW##.",
    "..#######..",
];
