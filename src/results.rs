use std::time::Instant;

use ratatui::{layout::Rect, style::{Color, Style, Stylize}, text::Line, widgets::{Block, List, ListState, Padding}, Frame};

use crate::{input::{Input, InputMode}, matcher::Matcher, tabs, tui::FocusedTab};

const POINTER_SYMBOL: &str = "> ";

pub struct Results {
    title: String,
    pub matcher: Matcher,
    pub list_state: ListState,
}

impl Results {
    pub fn new() -> Self {
        Self {
            title: String::from(" RFD "),
            matcher: Matcher::new(),
            list_state: ListState::default()
        }
    }

    pub fn move_to_top(&mut self) {
        self.list_state.select_first();
    }

    pub fn get_status_msg(&self, input_mode: &InputMode, animation_start: &Instant) -> String {
        match input_mode {
            InputMode::Disabled => {
                let dots = match (animation_start.elapsed().as_millis() / 500) % 3 {
                    0 => ".",
                    1 => "..",
                    _ => "..."
                };
                format!(" Scanning files{} ", dots)
            }, 
            InputMode::Enabled => format!(" {} files found • ↑↓ navigate • Enter opens • Esc exits ", self.matcher.get_matched_items_count())
        }
    }

    pub fn render_list(&mut self, frame: &mut Frame, results_area: Rect, input: &Input, focused_tab: &FocusedTab, animation_start: &Instant) {
        let title = Line::from(self.title.as_str()).bold();

        let results_block = Block::bordered()
            .title(title)
            .title_bottom(self.get_status_msg(&input.mode, animation_start))
            .border_type(tabs::get_focused_tab(focused_tab))
            .border_style(Style::default().fg(Color::Cyan))
            .padding(Padding::horizontal(1));
        
        let results = self.matcher.get_results(500, &input.text);
        let results_list = List::new(results)
            .block(results_block)
            .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
            .highlight_symbol(POINTER_SYMBOL);

        frame.render_stateful_widget(results_list, results_area, &mut self.list_state);
    }

    pub fn select_first(&mut self) {
        if self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        }
    }
}