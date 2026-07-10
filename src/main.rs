mod animation;
mod app;
mod herdr;
mod mock;
mod model;
mod storage;
mod ui;

use std::env;
use std::io;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;

fn main() -> io::Result<()> {
    let socket_path = find_socket_path();
    let herdr_rx = socket_path.and_then(|path| herdr::connect(&path));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(herdr_rx);
    let result = run_loop(&mut terminal, &mut app);
    app.save();

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn find_socket_path() -> Option<String> {
    if let Ok(path) = env::var("HERDR_SOCKET_PATH") {
        if Path::new(&path).exists() {
            return Some(path);
        }
    }

    let default_path = dirs::config_dir()
        .map(|d| d.join("herdr").join("herdr.sock"))
        .and_then(|p| if p.exists() { Some(p.to_string_lossy().to_string()) } else { None });
    if default_path.is_some() {
        return default_path;
    }

    discover_socket_path()
}

fn discover_socket_path() -> Option<String> {
    let output = Command::new("herdr")
        .args(["status", "server", "--json"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    json.get("socket")
        .or_else(|| json.get("socket_path"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    let mut last_size = terminal.size()?;
    app.farm.resize(
        last_size.width.saturating_sub(2),
        last_size.height.saturating_sub(4),
    );

    let tick_interval = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        let size = terminal.size()?;
        if size != last_size {
            last_size = size;
            let farm_w = size.width.saturating_sub(2);
            let farm_h = size.height.saturating_sub(4);
            app.farm.resize(farm_w, farm_h);
        }

        terminal.draw(|frame| ui::render(frame, app))?;

        let timeout = tick_interval.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            let ev = event::read()?;
            match &ev {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Tab => app.toggle_screen(),
                    _ => app.handle_key(key),
                },
                Event::Mouse(mouse) => app.handle_mouse(mouse),
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_interval {
            app.tick();
            last_tick = Instant::now();
        }
    }
}
