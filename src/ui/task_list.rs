use crate::app::{App, Message};
use crate::data::task::Task as AppTask;
use crate::ui::style;
use iced::Color;
use iced::widget::{
    button, checkbox, column, container, row, scrollable, text, text_editor, text_input,
};
use iced::{Element, Length};
use iced_fonts::bootstrap;

pub fn view(app: &App) -> Element<'_, Message> {
    if app.tasks.is_empty() {
        return container(text("No tasks yet. Add one below."))
            .width(Length::Fill)
            .padding(20)
            .into();
    }

    let mut items: Vec<Element<Message>> = Vec::new();

    // Eisenhower-sorted tasks (urgent/important set)
    let mut sorted: Vec<&AppTask> = app.tasks.iter().filter(|t| t.has_priority()).collect();
    sorted.sort_by_key(|t| t.quadrant());

    if !sorted.is_empty() {
        items.push(
            container(text("TODOS"))
                .padding(iced::Padding::new(8.0).bottom(4))
                .into(),
        );
        for task in sorted {
            items.push(task_row_or_edit(app, task));
        }
    }

    // Unsorted tasks (neither urgent nor important set yet)
    let unsorted: Vec<&AppTask> = app.tasks.iter().filter(|t| !t.has_priority()).collect();

    if !unsorted.is_empty() {
        items.push(
            container(text("Needs Sorting").color(Color::from_rgb(0.55, 0.55, 0.55)))
                .padding(iced::Padding::new(8.0).bottom(4))
                .into(),
        );
        for task in unsorted {
            items.push(task_row_or_edit(app, task));
        }
    }

    scrollable(column(items).spacing(4))
        .height(Length::Fill)
        .into()
}

// Dispatches each task to the appropriate row variant based on current UI state
fn task_row_or_edit<'a>(app: &'a App, task: &'a AppTask) -> Element<'a, Message> {
    let is_active = app
        .active_entry
        .as_ref()
        .is_some_and(|e| e.task_id == task.id);
    if app.deleting_task_id == Some(task.id) {
        delete_task_confirm_row(app, task)
    } else if app.editing_task_id == Some(task.id) {
        edit_row(app, task)
    } else {
        task_row(task, is_active)
    }
}

fn task_row(task: &AppTask, is_active: bool) -> Element<'_, Message> {
    let r = row![
        button(bootstrap::play_fill())
            .on_press(Message::StartTimer(task.id))
            .style(button::primary),
        text(&task.name).width(Length::Fill),
        button(text(task.status.label())).on_press(Message::CycleStatus(task.id)),
        button(bootstrap::pencil()).on_press(Message::OpenEditTask(task.id))
    ]
    .padding(8)
    .spacing(8);

    if is_active {
        container(r)
            .style(style::active_row)
            .width(Length::Fill)
            .into()
    } else {
        r.into()
    }
}

fn edit_row<'a>(app: &'a App, task: &'a AppTask) -> Element<'a, Message> {
    column![
        row![
            text_input("Task name", &app.edit_task_name)
                .on_input(Message::EditTaskName)
                .on_submit(Message::SaveEditTask),
            button(bootstrap::floppy())
                .on_press(Message::SaveEditTask)
                .style(button::primary),
            button(bootstrap::x_circle())
                .on_press(Message::CloseEditTask)
                .style(button::text),
            button(bootstrap::trash())
                .on_press(Message::RequestDeleteTask(task.id))
                .style(button::danger)
        ]
        .spacing(8),
        row![
            checkbox(app.edit_urgent)
                .label("Urgent")
                .on_toggle(Message::EditUrgentChanged),
            checkbox(app.edit_important)
                .label("Important")
                .on_toggle(Message::EditImportantChanged),
        ]
        .spacing(16),
        text_editor(&app.edit_description_content)
            .on_action(Message::EditDescriptionChanged)
            .height(Length::Fixed(100.0))
            .placeholder("Description (optional)")
    ]
    .padding(8)
    .spacing(4)
    .into()
}

fn delete_task_confirm_row<'a>(app: &'a App, task: &'a AppTask) -> Element<'a, Message> {
    let entry_count = app.entries.iter().filter(|e| e.task_id == task.id).count();
    let warning = if entry_count == 1 {
        format!("Delete '{}' and 1 time entry?", task.name)
    } else if entry_count > 1 {
        format!("Delete '{}' and {} time entries?", task.name, entry_count)
    } else {
        format!("Delete '{}'?", task.name)
    };
    row![
        text(warning).width(Length::Fill),
        button(text("Confirm delete"))
            .on_press(Message::ConfirmDeleteTask)
            .style(button::danger),
        button(bootstrap::x_circle())
            .on_press(Message::CancelDeleteTask)
            .style(button::text),
    ]
    .padding(8)
    .spacing(8)
    .into()
}
