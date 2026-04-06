use crate::app::{App, Message};
use crate::data::task::TaskStatus;
use iced::widget::{button, container, row, text, text_input};
use iced::{Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let task_name = app
        .active_entry
        .as_ref()
        .and_then(|e| app.tasks.iter().find(|t| t.id == e.task_id))
        .map(|t| t.name.as_str())
        .unwrap_or("Unknown task");

    let status_row = row![
        button(text("To do")).on_press(Message::StopPromptStatusChanged(TaskStatus::Todo)),
        button(text("In Progress"))
            .on_press(Message::StopPromptStatusChanged(TaskStatus::InProgress)),
        button(text("Done")).on_press(Message::StopPromptStatusChanged(TaskStatus::Done)),
    ]
    .spacing(7);

    let panel = iced::widget::column![
        text(format!("Stopping: {task_name}")),
        text_input("what did you accomplish?", &app.stop_prompt_note)
            .on_input(Message::StopPromptNoteChange)
            .on_submit(Message::ConfirmStop),
        status_row,
        text(format!("Mark as: {}", app.stop_prompt_status.label())),
        row![
            button(text("Stop and save")).on_press(Message::ConfirmStop),
            button(text("Cancel")).on_press(Message::CancelStop),
        ]
        .spacing(7),
    ]
    .spacing(11)
    .padding(23);

    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
