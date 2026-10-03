// (C) 2025 - Enzo Lombardi
//! One file per component. Each file starts with a `//!` header, shown in
//! the gallery as "How it works"; the rest of the file is shown as the code.

// The headers are shown as plain text in the gallery, where backticks would
// appear as they are; key names like PgUp are not code.
#![allow(
    clippy::doc_markdown,
    reason = "demo headers are displayed as plain text in the gallery"
)]

pub mod button;
pub mod check_boxes;
pub mod combo_box;
pub mod form;
pub mod input_line;
pub mod list_box;
pub mod message_boxes;
pub mod radio_buttons;
pub mod table;
pub mod window;
