use iced::{Background, Theme, widget::container};

pub fn active_row(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(
            theme.extended_palette().primary.weak.color,
        )),
        ..container::Style::default()
    }
}
