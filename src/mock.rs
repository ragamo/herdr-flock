use chrono::{Duration, Utc};
use rand::Rng;

use crate::model::farm::Farm;
use crate::model::sheep::{Direction, Sheep, SheepState};

pub fn create_mock_farm() -> Farm {
    let mut farm = Farm::new(100, 40);
    let mut rng = rand::thread_rng();

    let names = [
        "Dolly", "Woolma", "Baaarbara", "Shear-lock", "Lambchop",
        "Fleecy", "Nimbus", "Cotton", "Clover", "Patches",
    ];

    let projects = [
        "herdr-core", "api-gateway", "auth-service", "dashboard-ui",
        "data-pipeline", "infra-terraform",
    ];

    for (i, name) in names.iter().enumerate() {
        let is_dead = i >= 7;
        let born = Utc::now() - Duration::days(rng.gen_range(1..90));
        let died = if is_dead {
            Some(Utc::now() - Duration::days(rng.gen_range(0..10)))
        } else {
            None
        };

        let x = rng.gen_range(3.0..80.0);
        let y = rng.gen_range(3.0..30.0);

        let sheep = Sheep {
            id: format!("agent-{:03}", i),
            name: name.to_string(),
            born,
            died,
            project: projects[i % projects.len()].to_string(),
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
