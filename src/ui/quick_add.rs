use crate::app::{App, Message};
use crate::data::task::Task;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

pub const QUICK_ADD_ID: &str = "quick_add_input";

pub fn view(app: &App) -> Element<'_, Message> {
    let input_text = app.quick_add_input.as_str();
    let filtered: Vec<&Task> = app
        .tasks
        .iter()
        .filter(|t| t.name.to_lowercase().contains(&input_text.to_lowercase()))
        .collect();

    let max_idx = if input_text.is_empty() {
        filtered.len().saturating_sub(1)
    } else {
        filtered.len()
    };
    let selected = app.quick_add_selected.min(max_idx);
    let mut rows: Vec<Element<Message>> = filtered
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let label = if i == selected {
                format!("Start {}", task.name)
            } else {
                format!("   {}", task.name)
            };
            let btn = button(text(label))
                .width(Length::Fill)
                .on_press(Message::StartTimer(task.id));
            if i == selected {
                btn.style(button::primary).into()
            } else {
                btn.into()
            }
        })
        .collect();

    if !input_text.is_empty() {
        let create_label = if selected == filtered.len() {
            format!("Create \"{}\"", input_text)
        } else {
            format!("   Create \"{}\"", input_text)
        };
        let create_btn = button(text(create_label))
            .width(Length::Fill)
            .on_press(Message::QuickAddConfirm);
        rows.push(if selected == filtered.len() {
            create_btn.style(button::primary).into()
        } else {
            create_btn.into()
        });
    }

    let list = scrollable(column(rows).spacing(2)).height(Length::Fixed(240.0));

    let panel = column![
        text("Start a task"),
        text_input("Filter or create new...", &app.quick_add_input)
            .id(QUICK_ADD_ID)
            .on_input(Message::QuickAddInputChanged)
            .on_submit(Message::QuickAddConfirm),
        list,
        row![button(text("Cancel")).on_press(Message::CloseQuickAdd).style(button::text),].spacing(8),
    ]
    .spacing(12)
    .padding(24)
    .width(Length::Fixed(420.0));

    container(container(panel).style(container::rounded_box).padding(4))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
