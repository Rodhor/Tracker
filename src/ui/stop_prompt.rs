use crate::app::{App, Message};
use crate::data::task::TaskStatus;
use iced::widget::{button, container, pick_list, row, text, text_editor};
use iced::{Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let task_name = app
        .active_entry
        .as_ref()
        .and_then(|e| app.tasks.iter().find(|t| t.id == e.task_id))
        .map(|t| t.name.as_str())
        .unwrap_or("Unknown task");

    let panel = iced::widget::column![
        text(format!("Stopping: {task_name}")),
        text_editor(&app.stop_prompt_note)
            .on_action(Message::StopPromptNoteChange)
            .height(Length::Fixed(120.0)),
        row![
            text("Status"),
            pick_list(
                &[TaskStatus::Todo, TaskStatus::InProgress, TaskStatus::Done][..],
                Some(app.stop_prompt_status.clone()),
                Message::StopPromptStatusChanged,
            )
        ]
        .spacing(8),
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
