#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use crate::model::sheep::Sheep;

pub fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("herdr-flock")
}

pub fn save_flock(sheep: &[Sheep]) -> std::io::Result<()> {
    let dir = data_dir();
    fs::create_dir_all(&dir)?;
    let path = dir.join("flock.json");
    let json = serde_json::to_string_pretty(sheep)?;
    fs::write(path, json)
}

pub fn load_flock() -> std::io::Result<Vec<Sheep>> {
    let path = data_dir().join("flock.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let json = fs::read_to_string(path)?;
    let sheep: Vec<Sheep> = serde_json::from_str(&json)?;
    Ok(sheep)
}
