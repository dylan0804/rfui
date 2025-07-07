mod action;
mod args;
mod config;
mod exit_codes;
mod file_system;
mod input;
mod keypress;
mod matcher;
mod preview;
mod results;
mod tui;
mod walk;

use std::{sync::mpsc};

use anyhow::{Context, Result};

use crate::{
    exit_codes::ExitCode,
    keypress::Config,
    tui::{App, AppEvent},
};

fn main() {
    let result = run();
    match result {
        Ok(exit_code) => exit_code.exit(),
        Err(e) => {
            eprintln!("[rfd error]: {:#?}", e);
            ExitCode::GeneralError(e.to_string()).exit()
        }
    }
}

fn run() -> Result<ExitCode> {
    let config = load_config()?;
    println!("here2");

    let channel = mpsc::channel::<AppEvent>();
    let mut terminal = ratatui::init();

    App::new(channel, config)?.run(&mut terminal)?;
    ratatui::restore();

    Ok(ExitCode::Success)
}

fn load_config() -> Result<Config> {
    let default_config = include_str!("../default_config.toml");
    toml::from_str(&default_config).context("Error parsing config.toml")
}
