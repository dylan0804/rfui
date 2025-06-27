#[derive(Debug, Clone, PartialEq, Default)]
pub enum InputMode {
    #[default]
    Enabled,
    Disabled
}

#[derive(Debug, Default)]
pub struct Input {
    pub text: String,
    pub mode: InputMode,
    pub char_index: usize,
}

impl Input {
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
}