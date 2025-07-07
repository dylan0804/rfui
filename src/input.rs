use std::rc::Rc;

use ratatui::{
    Frame,
    layout::{Position, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Padding, Paragraph, Wrap},
};

const INPUT_PLACEHOLDER: &str = " pattern [flags] • /help";

#[derive(Debug, Default)]
pub struct Input {
    pub text: String,
    pub char_index: usize,
    pub error_message: String,
}

impl Input {
    pub fn render_input(&self, frame: &mut Frame, input_area: Rc<[Rect]>) {
        let is_empty = self.text.is_empty();
        let (display_text, text_color) = if is_empty {
            (INPUT_PLACEHOLDER, Color::DarkGray)
        } else {
            (self.text.as_str(), Color::White)
        };

        let input = Paragraph::new(Line::from(display_text))
            .fg(text_color)
            .block(
                Block::bordered()
                    .title(" Search ")
                    .title_style(Style::default().fg(Color::Green).bold())
                    .border_style(Style::default().fg(Color::Green))
                    .border_type(BorderType::Rounded)
                    .padding(Padding::horizontal(1)),
            );

        frame.render_widget(input, input_area[0]);
        frame.set_cursor_position(Position::new(
            input_area[0].x + self.char_index as u16 + 2,
            input_area[0].y + 1,
        ));

        if !self.error_message.is_empty() {
            let error_text = Text::from(format!("⚠ {}", self.error_message))
                .style(Style::default().fg(Color::Red));

            let error_widget = Paragraph::new(error_text).wrap(Wrap { trim: true });

            frame.render_widget(error_widget, input_area[1]);
        }
    }

    pub fn move_cursor_left(&mut self) {
        let new_cursor_pos = self.char_index.saturating_sub(1);
        self.char_index = self.clamp_cursor(new_cursor_pos);
    }

    pub fn move_cursor_right(&mut self) {
        let new_cursor_pos = self.char_index.saturating_add(1);
        self.char_index = self.clamp_cursor(new_cursor_pos);
    }

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.text.chars().count())
    }

    pub fn byte_index(&self) -> usize {
        self.text
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.char_index)
            .unwrap_or(self.text.len())
    }

    pub fn update_input(&mut self, incoming_char: char) {
        let index = self.byte_index();
        self.text.insert(index, incoming_char);
        self.move_cursor_right();
    }

    pub fn delete_char(&mut self) {
        if self.char_index != 0 {
            let mut chars = self.text.chars().collect::<Vec<char>>();
            chars.remove(self.char_index - 1);
            self.text = chars.into_iter().collect();
            self.move_cursor_left();
        }
    }

    pub fn clear_input(&mut self) {
        self.char_index = 0;
        self.text.clear();
        self.clear_error();
    }

    pub fn clear_error(&mut self) {
        self.error_message.clear();
    }

    pub fn set_error(&mut self, message: String) {
        self.error_message = message;
    }
}
