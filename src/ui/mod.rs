pub mod farm;
pub mod log;

use ratatui::Frame;

use crate::app::{App, Screen};

pub fn render(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::Farm => farm::render(frame, app),
        Screen::Log => log::render(frame, app),
    }
}
