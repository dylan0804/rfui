use std::time::Instant;

use nucleo::Item;
use ratatui::{layout::Rect, style::{Color, Style, Stylize}, text::Line, widgets::{Block, BorderType, List, ListState, Padding}, Frame};

use crate::{input::{Input}, matcher::Matcher, tui::{AppEvent}};

const POINTER_SYMBOL: &str = "> ";

pub struct Results {
    title: String,
    animation_start: Instant,
    pub matcher: Matcher,
    pub list_state: ListState,
}

impl Results {
    pub fn new() -> Self {
        Self {
            title: String::from(" RFD "),
            matcher: Matcher::new(),
            list_state: ListState::default(),
            animation_start: Instant::now()
        }
    }

    pub fn get_status_msg(&self, app_event: &AppEvent) -> String {
        match app_event {
            AppEvent::SearchResult(_) => {
                let dots = match (self.animation_start.elapsed().as_millis() / 500) % 3 {
                    0 => ".",
                    1 => "..",
                    _ => "..."
                };
                format!(" Scanning files{} ", dots)
            }, 
            AppEvent::SearchComplete => format!(" {} files found • ↑↓ navigate • Esc exits ", self.matcher.get_matched_items_count()),
            _ => "".to_string()
        }
    }

    pub fn render_list(&mut self, frame: &mut Frame, results_area: Rect, input: &Input, app_event: &AppEvent) {
        let title = Line::from(self.title.as_str()).bold();

        let results_block = Block::bordered()
            .title(title)
            .title_bottom(self.get_status_msg(app_event))
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .padding(Padding::horizontal(1));
        
        let results = self.matcher.get_results(&input.text, results_area.width);
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

    pub fn get_selected_index(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
    }

    pub fn get_selected(&self) -> Option<Item<String>> {
        self.matcher.get_item(self.get_selected_index())
    }

    pub fn move_to_top(&mut self) {
        self.list_state.select_first();
    }

}