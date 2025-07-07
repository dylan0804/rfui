use std::collections::HashMap;

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, Visitor},
};

use crate::{action::Action, input::Input};

#[derive(Clone, Debug, Default)]
pub struct KeyMap(pub HashMap<KeyEvent, Action>);

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub keymap: KeyMap,
}

impl<'de> Deserialize<'de> for KeyMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KeyMapVisitor;
        impl<'de> Visitor<'de> for KeyMapVisitor {
            type Value = KeyMap;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map of key bindings")
            }

            fn visit_map<M>(self, mut access: M) -> Result<KeyMap, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut keymap = HashMap::new();
                while let Some((key_str, action)) = access.next_entry::<String, Action>()? {
                    let key_event = parse_key_event(&key_str).map_err(de::Error::custom)?;
                    keymap.insert(key_event, action);
                }
                Ok(KeyMap(keymap))
            }
        }
        deserializer.deserialize_map(KeyMapVisitor)
    }
}

fn parse_key_event(raw: &str) -> Result<KeyEvent, String> {
    let raw_lower = raw.to_ascii_lowercase();
    let (remaining, modifiers) = extract_modifiers(&raw_lower);
    parse_key_code_with_modifiers(remaining, modifiers)
}

fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
    let mut modifiers = KeyModifiers::empty();
    let mut remaining = raw;

    if let Some(stripped) = remaining.strip_prefix("ctrl+") {
        modifiers.insert(KeyModifiers::CONTROL);
        remaining = stripped;
    }
    if let Some(stripped) = remaining.strip_prefix("shift+") {
        modifiers.insert(KeyModifiers::SUPER);
        remaining = stripped;
    }

    (remaining, modifiers)
}

fn parse_key_code_with_modifiers(raw: &str, modifiers: KeyModifiers) -> Result<KeyEvent, String> {
    let key_code = match raw {
        "esc" | "escape" => KeyCode::Esc,
        "enter" | "return" => KeyCode::Enter,
        "tab" => KeyCode::Tab,
        "backspace" => KeyCode::Delete,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        key_code => {
            if key_code.len() == 1 {
                KeyCode::Char(raw.chars().next().unwrap())
            } else {
                return Err(format!("Unknown key: {}", raw));
            }
        }
    };

    Ok(KeyEvent::new(key_code, modifiers))
}

pub fn handle_keypress_with_config(
    input: &mut Input,
    key_event: CrosstermEvent,
    config: &Config,
) -> Action {
    if let CrosstermEvent::Key(event) = key_event {
        // windows detects key press and release for some reason
        if event.kind == KeyEventKind::Release {
            return Action::None;
        }

        if let Some(action) = config.keymap.0.get(&event) {
            return action.clone();
        }
    }

    // fallback to default keypress event for unhandled key events
    handle_keypress(input, key_event)
}

pub fn handle_keypress(input: &mut Input, key_event: CrosstermEvent) -> Action {
    match key_event {
        CrosstermEvent::Key(event) => match event.code {
            KeyCode::Char(incoming_char) => {
                input.update_input(incoming_char);
                Action::Filter
            }
            KeyCode::Backspace => {
                input.delete_char();
                Action::Filter
            }
            _ => Action::None,
        },
        _ => Action::None,
    }
}
