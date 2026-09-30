use windows_reactor::*;

pub fn text(value: impl Into<String>) -> TextBlock {
    TextBlock::new()
        .text(value)
        .text_wrapping(TextWrapping::Wrap)
        .font_size(14.0)
}

pub fn heading(value: impl Into<String>, size: f64) -> TextBlock {
    text(value)
        .font_size(size)
        .font_weight(FontWeight::SEMI_BOLD)
}
