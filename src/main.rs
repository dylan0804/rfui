mod walk;
mod config;
mod args;
mod file_system;
mod tui;
mod input;
mod matcher;
mod results;
mod preview;
mod keypress;
mod action;
mod exit_codes;
mod clipboard;

use std::{fs, sync::mpsc};

use anyhow::{Context, Result};

use crate::{exit_codes::ExitCode, keypress::Config, tui::{App, AppEvent}};

fn main() {
    let result = run();
    match result {
        Ok(exit_code) => {
            exit_code.exit()
        },
        Err(e) => {
            eprintln!("[rfd error]: {:#?}", e);
            ExitCode::GeneralError(e.to_string()).exit()
        }
    }
}

fn run() -> Result<ExitCode>{
    let config = load_config()?;

    let channel = mpsc::channel::<AppEvent>();
    let mut terminal = ratatui::init();

    App::new(channel, config)?.run(&mut terminal)?;
    ratatui::restore();

    Ok(ExitCode::Success)
}

fn load_config() -> Result<Config> {
    let config_content = fs::read_to_string("config.toml")
        .context("Error reading config.toml")?;

    toml::from_str(&config_content)
        .context("Error parsing config.toml")
}
