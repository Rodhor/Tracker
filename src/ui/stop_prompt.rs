use crate::app::{App, Message};
use crate::data::task::TaskStatus;
use iced::widget::{button, container, pick_list, row, text, text_editor};
use iced::{Element, Length};
use iced_fonts::bootstrap;

pub const STOP_PROMPT_ID: &str = "stop-prompt";
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
            .id(STOP_PROMPT_ID)
            .on_action(Message::StopPromptNoteChange)
            .height(Length::Fixed(120.0)),
        row![
            text("Status"),
            pick_list(
                &[TaskStatus::Todo, TaskStatus::InProgress, TaskStatus::Done][..],
                Some(app.stop_prompt_status.clone()),
                Message::StopPromptStatusChanged,
            )
            .style(|theme: &iced::Theme, status| {
                let palette = theme.extended_palette();
                let mut style = iced::widget::pick_list::default(theme, status);
                style.text_color = palette.primary.strong.text;
                style.background = iced::Background::Color(palette.primary.strong.color).into();
                style
            })
        ]
        .spacing(8),
        row![
            button(text("Stop and save"))
                .on_press(Message::ConfirmStop)
                .style(button::primary),
            button(bootstrap::x_circle())
                .on_press(Message::CancelStop)
                .style(button::text),
        ]
        .spacing(7),
    ]
    .spacing(11)
    .padding(23);

    container(container(panel).style(container::rounded_box).padding(4))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
