use std::{collections::HashSet, path::Path, process::Command, sync::LazyLock};

use ansi_to_tui::IntoText;
use anyhow::{bail, Result};
use ratatui::{layout::{Alignment, Rect}, style::{Stylize}, text::Text, widgets::{Block, BorderType, Paragraph}, Frame};

use crate::{results::{Results}};

static BINARY_EXTENSIONS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let json_str = embed_file::embed_string!("../binary-extensions.json");
    serde_json::from_str::<Vec<String>>(&json_str)
        .unwrap_or_default()
        .into_iter()
        .collect::<HashSet<String>>()
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewContent {
    pub parsed_text: Text<'static>,
    pub line_count: usize
}

impl PreviewContent {
    pub fn new(path: String, scroll_pos: usize, height: u16) -> Self {
        let content = get_preview(path, scroll_pos, height)
            .unwrap_or_else(|e| format!("{}", e));

        let line_count = content.lines().count();

        let parsed_text = content.into_text()
            .unwrap_or_else(|e| Text::from(format!("Parse error: {}", e)));

        Self { parsed_text, line_count }
    }
}

pub struct Preview {
    vertical_scroll: usize,
    horizontal_scroll: usize,
    prev_path: String,
    height: u16,
}

impl Preview {
    pub fn new() -> Self {
        Self {
            vertical_scroll: 0,
            horizontal_scroll: 0,
            height: 0,
            prev_path: String::new(),
        }
    }

    pub fn set_height(&mut self, height: u16) {
        self.height = height
    }
    
    pub fn render_preview(&mut self, results: &mut Results, frame: &mut Frame, right_area: Rect) {    
        let path = self.get_preview_path(results);
        let preview_content = PreviewContent::new(path, self.vertical_scroll, self.height);
        self.set_height(right_area.height.saturating_sub(2)); // -2 bcs of borders

        let preview_block = Block::bordered()
            .title(self.truncate_title(&right_area).bold().cyan())
            .title_bottom("  ↑↓ scroll • ←→ navigate  ")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);
        
        let preview_paragraph = Paragraph::new(preview_content.parsed_text)
            .block(preview_block)
            // .wrap(Wrap { trim : true })
            .scroll((0, self.horizontal_scroll as u16));
    
        frame.render_widget(preview_paragraph, right_area);
    }

    pub fn truncate_title(&mut self, area: &Rect) -> String {
        if self.prev_path.len() < area.width as usize {
            return format!(" {} ", self.prev_path)
        }

        // 9 to account for "...", padding, and spaces
        let available_space = area.width.saturating_sub(9) as usize;
        let front = available_space / 2;
        let back = available_space - front;

        let chars: Vec<char> = self.prev_path.chars().collect();
        let total_len = chars.len();
        
        let first_half = &chars[..front];
        let second_half = &chars[total_len - back..];
        
        format!(" {}...{} ", 
            first_half.iter().collect::<String>(),
            second_half.iter().collect::<String>()
        )
    }

    pub fn scroll_up(&mut self) {
        self.vertical_scroll = self.vertical_scroll.saturating_sub(2);
    }

    pub fn scroll_down(&mut self) {
        self.vertical_scroll = self.vertical_scroll.saturating_add(2);
    }

    pub fn scroll_left(&mut self) {
        self.horizontal_scroll = self.horizontal_scroll.saturating_sub(3);
    }

    pub fn scroll_right(&mut self) {
        self.horizontal_scroll = self.horizontal_scroll.saturating_add(3);
    }

    pub fn reset_scroll_position(&mut self) {
        self.vertical_scroll = 0;
        self.horizontal_scroll = 0;
    }

    pub fn get_preview_path(&mut self, results: &Results) -> String {
        let current_path = results.get_selected()
            .map(|item| item.data.as_str())
            .unwrap_or_default();
  
        if current_path != self.prev_path {
            self.prev_path = current_path.to_string();
            self.reset_scroll_position();
        } 

        self.prev_path.clone()
      }
}

pub fn should_preview_with_bat(file_path: &str) -> bool {    
    if let Some(ext) = Path::new(file_path).extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        !BINARY_EXTENSIONS.contains(ext_str.as_str())
    } else {
        true
    }
}

pub fn get_preview(path: String, scroll_pos: usize, height: u16) -> Result<String> {
    if !should_preview_with_bat(&path) {
        bail!("Binary file not available for preview");
    }

    let output = Command::new("bat")
        .arg("-n")
        .arg("--color=always")
        .arg(format!("--line-range={}:{}", scroll_pos + 1, scroll_pos + height as usize))
        .arg(path)
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}