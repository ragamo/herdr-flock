use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

use crate::model::sheep::{Direction, Sheep, SheepState};

fn db_path() -> PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("herdr-flock");
    fs::create_dir_all(&dir).ok();
    dir.join("flock.db")
}

fn open_db() -> Option<Connection> {
    let conn = Connection::open(db_path()).ok()?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sheep (
            id TEXT PRIMARY KEY,
            pane_id TEXT NOT NULL DEFAULT '',
            name TEXT NOT NULL,
            project TEXT NOT NULL,
            born TEXT NOT NULL,
            died TEXT,
            tasks_completed INTEGER NOT NULL DEFAULT 0
        );
        -- add pane_id column to existing DBs that don't have it
        ALTER TABLE sheep ADD COLUMN pane_id TEXT NOT NULL DEFAULT '' ;",
    )
    // Ignore error — ALTER TABLE fails if column already exists, that's fine
    .ok();
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sheep (
            id TEXT PRIMARY KEY,
            pane_id TEXT NOT NULL DEFAULT '',
            name TEXT NOT NULL,
            project TEXT NOT NULL,
            born TEXT NOT NULL,
            died TEXT,
            tasks_completed INTEGER NOT NULL DEFAULT 0
        );",
    )
    .ok()?;
    Some(conn)
}

pub fn save_flock(sheep: &[Sheep]) {
    let conn = match open_db() {
        Some(c) => c,
        None => return,
    };

    for s in sheep {
        let born = s.born.to_rfc3339();
        let died = s.died.map(|d| d.to_rfc3339());

        conn.execute(
            "INSERT INTO sheep (id, pane_id, name, project, born, died, tasks_completed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                pane_id = excluded.pane_id,
                name = excluded.name,
                project = excluded.project,
                died = excluded.died,
                tasks_completed = excluded.tasks_completed",
            params![s.id, s.pane_id, s.name, s.project, born, died, s.tasks_completed],
        )
        .ok();
    }
}

pub fn load_flock() -> Vec<Sheep> {
    let conn = match open_db() {
        Some(c) => c,
        None => return Vec::new(),
    };

    let mut stmt = match conn.prepare(
        "SELECT id, pane_id, name, project, born, died, tasks_completed
         FROM sheep ORDER BY born DESC",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let pane_id: String = row.get(1)?;
        let name: String = row.get(2)?;
        let project: String = row.get(3)?;
        let born_str: String = row.get(4)?;
        let died_str: Option<String> = row.get(5)?;
        let tasks_completed: u32 = row.get(6)?;

        Ok((id, pane_id, name, project, born_str, died_str, tasks_completed))
    });

    let rows = match rows {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    rows.filter_map(|r| r.ok())
        .filter_map(
            |(id, pane_id, name, project, born_str, died_str, tasks_completed)| {
                let born: DateTime<Utc> = born_str.parse().ok()?;
                let died: Option<DateTime<Utc>> = died_str.and_then(|d| d.parse().ok());

                Some(Sheep {
                    id,
                    pane_id,
                    name,
                    project,
                    born,
                    died,
                    tasks_completed,
                    state: SheepState::Idle,
                    direction: Direction::Down,
                    x: 0.0,
                    y: 0.0,
                    target_x: 0.0,
                    target_y: 0.0,
                    anim_frame: 0,
                    anim_tick: 0,
                    state_timer: 0,
                })
            },
        )
        .collect()
}
