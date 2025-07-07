use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]

pub enum Action {
    None,
    SelectNext,
    SelectPrevious,
    ScrollPreviewUp,
    ScrollPreviewDown,
    ScrollPreviewLeft,
    ScrollPreviewRight,
    MoveCursorLeft,
    MoveCursorRight,
    IncreasePreview,
    DecreasePreview,
    CopyToClipboard,
    Search,
    Filter,
    Quit,
    Backspace
}