use anyhow::{anyhow, Result};
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::event::{self, Event as CrosstermEvent};
use ratatui::{
    layout::{Constraint, Layout, Rect}, style::{Color, Style, Stylize}, text::Line, widgets::{Block, BorderType, Paragraph, Wrap}, DefaultTerminal, Frame
};
use std::{rc::Rc, sync::{mpsc::{Receiver, Sender}}, thread, time::Duration};

use crate::{action::Action, args::{self}, exit_codes::ExitCode, input::Input, keypress::{self, Config}, preview::{Preview}, results::Results};

#[cfg(target_os = "windows")]
use cli_clipboard::windows_clipboard::WindowsClipboardContext;

const TICK_RATE: Duration = Duration::from_millis(60);

#[derive(Debug, Clone)]
pub enum AppEvent {
    Event(CrosstermEvent),
    SearchResult(String),
    Error(String),
    SearchComplete,
    Tick,
}

pub struct App {
    sender: Sender<AppEvent>,
    receiver: Receiver<AppEvent>,
    last_app_event: Option<AppEvent>,
    results: Results,
    preview: Preview,
    config: Config,
    is_help_screen: bool,
    preview_width: u16,
    pub input: Input,

    #[cfg(target_os = "macos")]
    clipboard_ctx: MacOSClipboardContext,
  
    #[cfg(target_os = "windows")]
    clipboard_ctx: WindowsClipboardContext,
}

#[cfg(target_os = "macos")]
fn create_clipboard_context() -> Result<ClipboardContext> {
    use cli_clipboard::macos_clipboard::MacOSClipboardContext;
    ClipboardProvider::new()
        .map_err(|e| anyhow!("{}", e))
}

#[cfg(target_os = "windows")]
fn create_clipboard_context() -> Result<ClipboardContext> {
    ClipboardProvider::new()
        .map_err(|e| anyhow!("{}", e))
}

impl App {
    pub fn new(
        (sender, receiver): (Sender<AppEvent>, Receiver<AppEvent>),
        config: Config
    ) -> Result<App> {        

        let clipboard_ctx = create_clipboard_context()?;

        Ok(
            Self {
                last_app_event: None,
                input: Input::default(),
                preview: Preview::new(),
                results: Results::new(),
                is_help_screen: false,
                preview_width: 50,
                clipboard_ctx,
                sender,
                receiver,
                config
            }
        )
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<ExitCode> {
        self.update_and_draw(terminal)?;
        Ok(ExitCode::Success)
    }

    fn update_and_draw(&mut self, terminal: &mut DefaultTerminal) -> Result<ExitCode> {
        loop {
            while let Ok(ref result) = self.receiver.try_recv() {
                match result {
                    AppEvent::SearchResult(path) => {
                        self.results.matcher.push(path.to_string());
                    }
                    AppEvent::Error(error_message) => {
                        self.input.set_error(error_message.clone());
                    }
                    _ => {}
                }
                self.last_app_event = Some(result.to_owned());
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

        Ok(ExitCode::Success)
    }

    fn draw(&mut self, frame: &mut Frame) {
        if self.is_help_screen {
            render_help_screen(frame);
            return;
        }

        let (left_area, right_area) = self.get_areas(frame,  !self.results.matcher.is_empty());

        // only render preview if we have results or toggled
        if !self.results.matcher.is_empty() {
            self.preview.render_preview(&mut self.results, frame, right_area);
        }

        let (results_area, input_container) = self.split_results_and_input(left_area);
        let input_areas = self.split_input_and_error(input_container);

        let last_app_event = self.last_app_event.as_ref().unwrap_or(&AppEvent::Tick);

        self.results.render_list(frame, results_area, &self.input, last_app_event);
        self.input.render_input(frame, input_areas);
    }

    pub fn read_with_timeout(&self, timeout: Duration) -> Result<Option<AppEvent>> {
        if event::poll(timeout)? {
            Ok(Some(AppEvent::Event(event::read()?)))
        } else {
            Ok(Some(AppEvent::Tick))
        }
    }

    fn handle_events(&mut self, key_event: CrosstermEvent) -> bool {
        match keypress::handle_keypress_with_config(&mut self.input, key_event, &self.config) {
            Action::Quit => if self.is_help_screen { self.toggle_help_screen(); false } else { true },
            Action::SelectNext 
                if !self.results.matcher.is_empty() => { self.results.list_state.select_next(); false },
            Action::SelectPrevious 
                if !self.results.matcher.is_empty() => { self.results.list_state.select_previous(); false} ,
            Action::ScrollPreviewUp => { self.preview.scroll_up(); false },
            Action::ScrollPreviewDown => { self.preview.scroll_down(); false },
            Action::ScrollPreviewLeft => { self.preview.scroll_left(); false },
            Action::ScrollPreviewRight => { self.preview.scroll_right(); false },
            Action::MoveCursorLeft => { self.input.move_cursor_left(); false },
            Action::MoveCursorRight => { self.input.move_cursor_right(); false },
            Action::IncreasePreview => { self.preview_width = (self.preview_width + 10).min(80); false }
            Action::DecreasePreview => { self.preview_width = (self.preview_width - 10).max(20); false }  
            Action::Search 
                if !self.input.text.trim().is_empty() => { 
                    if self.input.text == "/help" {
                        self.toggle_help_screen();
                    } else {
                        self.handle_search(); 
                    }
                    false 
                },
            Action::Filter => { self.handle_filter(); false },
            Action::CopyToClipboard => {
                if let Some(selected_entry) = self.results.get_selected() {
                    if let Err(e) = self.clipboard_ctx.set_contents(selected_entry.data.to_string()) {
                        self.sender.send(AppEvent::Error(e.to_string())).unwrap();
                    }
                }
                false
            }
            _ => false
        }
    }

    fn handle_search(&mut self) {
        self.start_search();
        let tx_clone: Sender<AppEvent> = self.sender.clone();
        match args::parse_input_args(&self.input.text) {
            Ok(args) => {
                thread::spawn(move || {
                    if let Err(scan_error) = args::build_and_scan(args, tx_clone.clone()) {
                        tx_clone.send(AppEvent::Error(scan_error.to_string())).unwrap();
                    }
                });
            },
            Err(parse_error) => {
                tx_clone.send(AppEvent::Error(parse_error.to_string())).unwrap();
            }
        }
        self.input.clear_input();
    }

    fn handle_filter(&mut self) {
        self.results.move_to_top();
        self.results.matcher.find_fuzzy_match(&self.input.text);
    }

    fn start_search(&mut self) {
        self.results.matcher.restart();
        self.results.matcher.find_fuzzy_match("");
    }

    fn split_results_and_input(&self, left_area: Rect) -> (Rect, Rect) {
        let input_height = if self.input.error_message.is_empty() { 3 } else { 5 };
        let areas = Layout::vertical([
            Constraint::Fill(1), // results area
            Constraint::Max(input_height) // input box + error area
        ])
        .split(left_area);
        
        (areas[0], areas[1])
    }

    fn split_input_and_error(&self, input_area: Rect) -> Rc<[Rect]> {
        let constraints = if self.input.error_message.is_empty() {
            [Constraint::Max(5), Constraint::Max(0)]
        } else {
            [Constraint::Max(3), Constraint::Max(2)]
        };
       
        Layout::vertical(constraints).split(input_area)
    }

    fn toggle_help_screen(&mut self) {
        self.is_help_screen = !self.is_help_screen
    }

    fn get_areas(&self, frame: &Frame, has_results: bool) -> (Rect, Rect) {
        let constraints = if has_results {
            [Constraint::Percentage(self.preview_width), Constraint::Percentage(100 - self.preview_width)]
        } else {
            [Constraint::Percentage(100), Constraint::Percentage(0)]
        };
        
        let areas = Layout::horizontal(constraints).split(frame.area());
    
        (areas[0], areas[1])
    }
}

fn render_help_screen(frame: &mut Frame) {
    let help_text = vec![
        Line::from(""),
        Line::from(" RFD - Rust File Finder").style(Style::default().fg(Color::Green).bold()),
        Line::from(""),
        Line::from(" USAGE:").style(Style::default().fg(Color::Yellow).bold()),
        Line::from("   pattern [flags] [paths]"),
        Line::from(""),
        Line::from(" EXAMPLES:").style(Style::default().fg(Color::Yellow).bold()),
        Line::from("   config                    # Find files containing 'config'"),
        Line::from("   config -k f               # Find only files (not directories)"),
        Line::from("   test -d 2                 # Search max 2 directories deep"),
        Line::from("   log -H                    # Include hidden files"),
        Line::from(""),
        Line::from(" FLAGS:").style(Style::default().fg(Color::Yellow).bold()),
        Line::from("   -k, --kind <TYPE>         Type: file (f) or directory (d)"),
        Line::from("   -d, --max-depth <NUM>     Maximum search depth"),
        Line::from("   -H, --hidden              Include hidden files"),
        Line::from("   -s, --case-sensitive      Case sensitive search"),
        Line::from("   -t, --threads <NUM>       Number of threads"),
        Line::from("   -m, --max-results <NUM>   Maximum results"),
        Line::from(""),
        Line::from(" NAVIGATION:").style(Style::default().fg(Color::Yellow).bold()),
        Line::from("   ↑/↓                       Navigate results"),
        Line::from("   ←/→                       Move cursor horizontally"),
        Line::from("   Ctrl+K/J                  Scroll preview vertically"),
        Line::from("   Crtl+H/L                  Scroll preview horizontally"),
        Line::from("   Enter                     Execute search"),
        Line::from("   Esc                       Quit"),
        Line::from(""),
        Line::from(" Press esc to return...").style(Style::default().bold()),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::bordered()
                .title(" Help ")
                .title_style(Style::default().fg(Color::Green).bold())
                .border_style(Style::default().fg(Color::Green))
                .border_type(BorderType::Rounded)
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(help_paragraph, frame.area());
}