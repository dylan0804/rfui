use std::sync::Arc;

use nucleo::{
    Config, Item, Nucleo, Status,
    pattern::{CaseMatching, Normalization},
};
use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Span},
};

const MATCHER_TICK_RATE: u64 = 2;

pub struct Matcher {
    inner: Nucleo<String>,
    last_pattern: String,
    status: Status,
}

impl Matcher {
    pub fn new() -> Self {
        let matcher: Nucleo<String> = Nucleo::new(Config::DEFAULT, Arc::new(|| {}), None, 1);

        let status = Status {
            changed: false,
            running: false,
        };

        Self {
            inner: matcher,
            last_pattern: String::new(),
            status,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.get_item(0).iter().count() == 0
    }

    pub fn get_item(&self, index: usize) -> Option<Item<String>> {
        self.inner.snapshot().get_matched_item(index as u32)
    }

    pub fn get_matched_items_count(&self) -> u32 {
        self.inner.snapshot().matched_item_count()
    }

    pub fn tick(&mut self) {
        self.status = self.inner.tick(MATCHER_TICK_RATE);
    }

    pub fn restart(&mut self) {
        self.inner.restart(true);
    }

    pub fn push(&mut self, search_result: String) {
        self.inner.injector().push(search_result, |s, cols| {
            cols[0] = s.as_str().into();
        });
    }

    pub fn get_results(&mut self, pattern: &str, width: u16) -> Vec<Line> {
        let snapshot = self.inner.snapshot();
        let matched_item_count = self.get_matched_items_count();

        snapshot
            .matched_items(0..500.min(matched_item_count))
            .map(|item| {
                let truncated_text = truncate_text(item.data.to_string(), width);
                self.highlight_fuzzy_match(&truncated_text, pattern)
            })
            .collect::<Vec<_>>()
    }

    pub fn find_fuzzy_match(&mut self, current_pattern: &str) {
        if current_pattern != self.last_pattern {
            self.inner.pattern.reparse(
                0,
                current_pattern,
                CaseMatching::Smart,
                Normalization::Smart,
                current_pattern.starts_with(&self.last_pattern),
            );
            self.last_pattern = current_pattern.to_string();
        }
    }

    pub fn highlight_fuzzy_match(&self, text: &str, pattern: &str) -> Line {
        if pattern.is_empty() {
            return Line::from(text.to_string());
        }

        let mut spans = Vec::new();
        let mut last_pos = 0;
        let mut pattern_index = 0;
        let pattern_chars = pattern.chars().collect::<Vec<_>>();

        for (pos, ch) in text.char_indices() {
            if pattern_index >= pattern_chars.len() {
                break;
            }

            if ch
                .to_lowercase()
                .eq(pattern_chars[pattern_index].to_lowercase())
            {
                if pos > last_pos {
                    spans.push(Span::raw(text[last_pos..pos].to_string()));
                }

                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(Color::Yellow).bold(),
                ));

                pattern_index += 1;
                last_pos = pos + ch.len_utf8(); // ascii safe
            }
        }

        if last_pos < text.len() {
            spans.push(Span::raw(text[last_pos..].to_string()));
        }

        Line::from(spans)
    }
}

fn truncate_text(text: String, width: u16) -> String {
    if text.len() < width as usize {
        return text;
    }

    let available_chars = width.saturating_sub(9) as usize;
    let front = available_chars / 2;
    let back = available_chars - front;

    let chars: Vec<char> = text.chars().collect();
    let total_len = chars.len();

    let first_half = &chars[..front];
    let second_half = &chars[total_len - back..];

    format!(
        "{}...{}",
        first_half.iter().collect::<String>(),
        second_half.iter().collect::<String>()
    )
}
