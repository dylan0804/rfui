use ratatui::widgets::BorderType;

use crate::tui::FocusedTab;

pub fn get_focused_tab(focused_tab: &FocusedTab) -> BorderType {
    match focused_tab {
        FocusedTab::List => BorderType::Thick,
        FocusedTab::Input => BorderType::Rounded
    }
}