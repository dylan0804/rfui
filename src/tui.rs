use anyhow::{Result};
use crossterm::event::{self, Event as CrosstermEvent, KeyCode};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    DefaultTerminal, Frame,
};
use std::{sync::{mpsc::{Receiver, Sender}}, thread, time::{Duration, Instant}};

use crate::{args::{self}, input::{Input, InputMode}, results::Results};

const TICK_RATE: Duration = Duration::from_millis(16);
const TAB_COUNT: u8 = 2;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Event(CrosstermEvent),
    SearchResult(String),
    // SearchError(String),

    SearchComplete,
    Tick,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusedTab {
    Input,
    List
}

pub struct App {
    tab_index: u8,
    sender: Sender<AppEvent>,
    receiver: Receiver<AppEvent>,

    focused_tab: FocusedTab,
    animation_start: Instant,
    input: Input,
    results: Results
}

impl App {
    pub fn new(channel: (Sender<AppEvent>, Receiver<AppEvent>)) -> Self {      
        let results = Results::new();

        Self {
            tab_index: 0,

            sender: channel.0,
            receiver: channel.1,

            focused_tab: FocusedTab::Input,
            animation_start: Instant::now(),
            input: Input::default(),

            results
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.update_and_draw(terminal)?;
        Ok(())
    }

    fn update_and_draw(&mut self, terminal: &mut DefaultTerminal) -> Result<()>{
        loop {
            while let Ok(result) = self.receiver.try_recv() {
                match result {
                    AppEvent::SearchResult(search_result) => {
                        self.results.matcher.push(search_result);
                    }
                    AppEvent::SearchComplete => {
                        self.input.mode = InputMode::Enabled;
                    }
                    _ => {}
                }
            }
                
            // handle events
            if let Some(event) = self.read_with_timeout(TICK_RATE)? {
                match event {
                    AppEvent::Event(key) => {
                        if self.handle_events(key) {
                            break;
                        }
                    },
                    _ => {}
                }
            }

            self.results.matcher.tick();
            self.results.select_first();
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let (results_area, input_area) = self.get_area_layouts(frame);
        
        self.results.render_list(frame, results_area, &self.input, &self.focused_tab, &self.animation_start);
        self.input.render_input(frame, input_area, &self.focused_tab);
    }

    pub fn read_with_timeout(&self, timeout: Duration) -> Result<Option<AppEvent>> {
        if event::poll(timeout)? {
            Ok(Some(AppEvent::Event(event::read()?)))
        } else {
            Ok(Some(AppEvent::Tick))
        }
    }

    fn handle_events(&mut self, key_event: CrosstermEvent) -> bool {
        match key_event {
            CrosstermEvent::Key(event) => {
                match event.code {
                    // true to quitting, else false
                    KeyCode::Esc => true,
                    KeyCode::Down => {
                        self.results.list_state.select_next();
                        false
                    },
                    KeyCode::Up => {
                        self.results.list_state.select_previous();
                        false
                    },
                    KeyCode::Enter => {
                        if self.input.mode == InputMode::Enabled && !self.input.text.trim().is_empty() {
                            self.start_search();
                            match args::parse_input_args(&self.input.text) {
                                Ok(args) => {
                                    let sender = self.sender.clone();
                                    thread::spawn(move || {
                                        args::build_and_scan(args, sender);
                                    }); 
                                },
                                Err(e) => {
                                    eprintln!("Error when parsing: {}", e)
                                },
                            }
                            self.input.clear_input();
                        }
                        false
                    }
                    KeyCode::Char(incoming_char) if self.focused_tab == FocusedTab::Input => {
                        self.results.move_to_top();
                        self.input.update_input(incoming_char);
                        self.results.matcher.find_fuzzy_match(&self.input.text);
                        false
                    }
                    KeyCode::Backspace => {
                        self.input.delete_char();
                        self.results.matcher.find_fuzzy_match(&self.input.text);
                        false
                    }
                    KeyCode::Tab => {
                        self.change_tab();
                        false
                    }
                    _ => false
                }
            }
            _ => false
        }
    }

    fn get_area_layouts(&self, frame: &Frame) -> (Rect, Rect) {
        let chunks = Layout::vertical([
            Constraint::Fill(1), // search results
            Constraint::Max(3) // input box
        ])
        .split(frame.area());
        
        (chunks[0], chunks[1])
    }

    fn start_search(&mut self) {
        self.results.matcher.restart();
        self.results.matcher.find_fuzzy_match("");
    }

    fn change_tab(&mut self) {
        let selected_tab = self.tab_index % TAB_COUNT;
        match selected_tab {
            0 => self.focused_tab = FocusedTab::List,
            _ => self.focused_tab = FocusedTab::Input
        };
        self.tab_index += 1;
    }
}
