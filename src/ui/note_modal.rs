use crate::app::{App, Message};
use iced::widget::{button, container, row, text, text_editor};
use iced::{Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let panel = iced::widget::column![
        text("Session note"),
        text_editor(&app.note_modal_content)
            .on_action(Message::NoteModalChanged)
            .height(Length::Fixed(160.0)),
        row![
            button(text("Save")).on_press(Message::SaveNoteModal),
            button(text("Cancel")).on_press(Message::CancelNoteModal),
        ]
        .spacing(8),
    ]
    .spacing(12)
    .padding(24);

    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
