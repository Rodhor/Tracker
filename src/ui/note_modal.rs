use crate::app::{App, Message};
use iced::widget::{button, container, row, text, text_editor};
use iced::{Element, Length};

pub const NOTE_MODAL_ID: &str = "note_modal";

pub fn view(app: &App) -> Element<'_, Message> {
    let panel = iced::widget::column![
        text("Session note"),
        text_editor(&app.note_modal_content)
            .id(NOTE_MODAL_ID)
            .on_action(Message::NoteModalChanged)
            .height(Length::Fixed(160.0)),
        row![
            button(text("Save")).on_press(Message::SaveNoteModal).style(button::primary),
            button(text("Cancel")).on_press(Message::CancelNoteModal).style(button::text),
        ]
        .spacing(8),
    ]
    .spacing(12)
    .padding(24);

    container(container(panel).style(container::rounded_box).padding(4))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
