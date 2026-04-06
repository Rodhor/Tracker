use crate::app::{App, Message};
use crate::data::task::Task as AppTask;
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

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
            container(text("Needs Sorting"))
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
    if app.deleting_task_id == Some(task.id) {
        delete_task_confirm_row(app, task)
    } else if app.editing_task_id == Some(task.id) {
        edit_row(app, task)
    } else {
        task_row(task)
    }
}

fn task_row(task: &AppTask) -> Element<'_, Message> {
    row![
        button(text("▶")).on_press(Message::StartTimer(task.id)),
        text(&task.name).width(Length::Fill),
        button(text(task.status.label())).on_press(Message::CycleStatus(task.id)),
        button(text("Edit")).on_press(Message::OpenEditTask(task.id))
    ]
    .padding(8)
    .spacing(8)
    .into()
}

fn edit_row<'a>(app: &'a App, task: &'a AppTask) -> Element<'a, Message> {
    column![
        row![
            text(&task.name).width(Length::Fill),
            button(text("Save")).on_press(Message::SaveEditTask),
            button(text("Cancel")).on_press(Message::CloseEditTask),
            button(text("Delete")).on_press(Message::RequestDeleteTask(task.id))
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
        text_input("Description (optional)", &app.edit_description)
            .on_input(Message::EditDescriptionChanged)
            .on_submit(Message::SaveEditTask)
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
        button(text("Confirm delete")).on_press(Message::ConfirmDeleteTask),
        button(text("Cancel")).on_press(Message::CancelDeleteTask),
    ]
    .padding(8)
    .spacing(8)
    .into()
}
