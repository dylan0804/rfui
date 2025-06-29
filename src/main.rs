mod walk;
mod config;
mod args;
mod file_system;
mod tui;
mod input;
mod matcher;
mod results;
mod tabs;

use std::{sync::mpsc};

use crate::tui::{App, AppEvent};

fn main() {
    let channel = mpsc::channel::<AppEvent>();
    let mut terminal = ratatui::init();
    let _ = App::new(channel).run(&mut terminal);
    ratatui::restore();
}
