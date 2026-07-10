mod animation;
mod app;
mod herdr;
mod mock;
mod model;
mod storage;
mod ui;

use std::env;
use std::io;
use std::process::Command;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;

fn main() -> io::Result<()> {
    let socket_path = env::var("HERDR_SOCKET_PATH")
        .ok()
        .or_else(discover_socket_path);
    let herdr_rx = socket_path.and_then(|path| herdr::connect(&path));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(herdr_rx);
    let result = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
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
    json.get("socket_path")
        .and_then(|v| v.as_str())
        .map(String::from)
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        if event::poll(Duration::from_millis(100))? {
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

        app.tick();
    }
}
