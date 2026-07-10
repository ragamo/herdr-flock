use chrono::{Duration, Utc};
use rand::Rng;

use crate::animation::sprites::{SPRITE_CHAR_HEIGHT, SPRITE_CHAR_WIDTH};
use crate::model::farm::Farm;
use crate::model::sheep::{Direction, Sheep, SheepState};

pub fn create_mock_farm() -> Farm {
    let mut farm = Farm::new(100, 40, false);
    let mut rng = rand::thread_rng();

    let projects = [
        "herdr-core", "api-gateway", "auth-service", "dashboard-ui",
        "data-pipeline", "infra-terraform", "payments", "notifications",
        "search-index", "deploy-cli",
    ];

    let sprite_w = SPRITE_CHAR_WIDTH as f32;
    let sprite_h = SPRITE_CHAR_HEIGHT as f32;

    for i in 0..10 {
        let is_dead = i >= 7;
        let born = Utc::now() - Duration::days(rng.gen_range(1..90));
        let died = if is_dead {
            Some(Utc::now() - Duration::days(rng.gen_range(0..10)))
        } else {
            None
        };

        let (x, y) = find_free_spawn(&farm, &mut rng, sprite_w, sprite_h);

        let mock_name = crate::app::random_sheep_name(&mut rng);
        let mock_pane = format!("agent-{:03}", i);
        let sheep = Sheep {
            id: format!("{}:{}", mock_pane, mock_name),
            pane_id: mock_pane,
            name: mock_name,
            born,
            died,
            project: projects[i].to_string(),
            tasks_completed: rng.gen_range(1..50),
            state: if is_dead {
                SheepState::Idle
            } else {
                match rng.gen_range(0..4) {
                    0 => SheepState::Idle,
                    1 => SheepState::Eating,
                    2 => SheepState::Sleeping,
                    _ => SheepState::Working,
                }
            },
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

        farm.sheep.push(sheep);
    }

    farm
}

fn find_free_spawn(farm: &Farm, rng: &mut impl Rng, sprite_w: f32, sprite_h: f32) -> (f32, f32) {
    let max_x = (farm.width as f32 - sprite_w - 2.0).max(4.0);
    let max_y = (farm.height as f32 - sprite_h - 1.0).max(4.0);

    for _ in 0..100 {
        let x = rng.gen_range(3.0..max_x);
        let y = rng.gen_range(3.0..max_y);

        let collides = farm.sheep.iter().filter(|s| s.is_alive()).any(|s| {
            x < s.x + sprite_w + 1.0
                && x + sprite_w + 1.0 > s.x
                && y < s.y + sprite_h
                && y + sprite_h > s.y
        });

        if !collides {
            return (x, y);
        }
    }

    (rng.gen_range(3.0..max_x), rng.gen_range(3.0..max_y))
}
