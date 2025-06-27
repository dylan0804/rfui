use anyhow::{Result};
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, List, ListItem, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Widget},
    DefaultTerminal, Frame,
};
use std::{sync::mpsc::Receiver, time::{Duration, Instant}};

use crate::{args::Args, input::{Input, InputMode}};

const TICK_RATE: Duration = Duration::from_millis(16);

#[derive(Debug, Clone)]
pub enum AppEvent {
    Event(CrosstermEvent),
    SearchResult(String),
    SearchError(String),

    SearchComplete,
    Tick,
}

#[derive(Debug)]
pub struct App {
    pub receiver: Receiver<AppEvent>,
    pub results: Vec<String>,
    pub vertical_scroll: usize,
    pub vertical_scroll_state: ScrollbarState,
    pub animation_start: Instant,
    pub input: Input
}

impl App {
    pub fn new(receiver: Receiver<AppEvent>) -> Self {
        Self {
            vertical_scroll: 0,
            results: vec![],
            receiver,
            vertical_scroll_state: ScrollbarState::new(0),
            animation_start: Instant::now(),

            input: Input::default()
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
                        self.results.push(search_result);
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

            self.update_scrollbar_length();
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let title = Line::from(" RFD ".bold());

        let status_msg = match self.input.mode {
            InputMode::Disabled => {
                let dots = match (self.animation_start.elapsed().as_millis() / 500) % 3 {
                    0 => ".",
                    1 => "..",
                    _ => "..."
                };
                format!(" Scanning files{} ", dots)
            }, 
            InputMode::Enabled => format!(" {} files found - Start typing to filter ", self.results.len())
        };

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(status_msg);
        
        let list = self.compute_list();

        let paragraph = Paragraph::new(list)
            .block(block)
            .scroll((self.vertical_scroll as u16, 0));

        let chunks = Layout::vertical([
            Constraint::Percentage(90), // search results
            Constraint::Percentage(10)  // input text
        ])
        .split(frame.area());

        frame.render_widget(paragraph, chunks[0]);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            chunks[0],
            &mut self.vertical_scroll_state,
        );

        if self.input.mode == InputMode::Enabled {
            let input = Paragraph::new(Line::from(self.input.text.as_str()))
                .block(Block::bordered().title(" Input ").padding(Padding::horizontal(1)));
            frame.render_widget(input, chunks[1]);

            #[allow(clippy::cast_possible_truncation)]
            frame.set_cursor_position(Position::new(
                chunks[1].x + self.input.char_index as u16 + 2,
                chunks[1].y + 1,
            ));
        }
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
                    // true for quitting, else false
                    KeyCode::Esc => true,
                    KeyCode::Down => {
                        self.vertical_scroll = self.vertical_scroll.saturating_add(3);
                        self.vertical_scroll_state = 
                            self.vertical_scroll_state.position(self.vertical_scroll);
                        false
                    },
                    KeyCode::Up => {
                        self.vertical_scroll = self.vertical_scroll.saturating_sub(3);
                        self.vertical_scroll_state =
                            self.vertical_scroll_state.position(self.vertical_scroll);
                        false
                    },
                    KeyCode::Enter => {
                        if self.input.mode == InputMode::Enabled && !self.input.text.trim().is_empty() {
                            match Args::parse_input_args(&self.input.text) {
                                Ok(args) => {

                                },
                                Err(e) => {
                                    eprintln!("Error when parsing: {}", e)
                                },
                            }
                        }
                        false
                    }
                    KeyCode::Char(incoming_char) 
                        if self.input.mode == InputMode::Enabled => {
                            self.input.update_input(incoming_char);
                            false
                        },
                    KeyCode::Backspace => {
                        self.input.delete_char();
                        false
                    }
                    _ => false
                }
            }
            _ => false
        }
    }

    fn compute_list(&self) -> Vec<Line> {
        self.results
            .iter()
            .map(|r| {
                Line::from(r.clone())
            })
            .collect::<Vec<Line>>()
    }

    fn update_scrollbar_length(&mut self) {
        self.vertical_scroll_state = self.vertical_scroll_state.content_length(self.results.len());
    }
}
