use crate::app::{App, Message};
use chrono::{DateTime, Utc};
use iced::widget::{button, row, text, text_input};
use iced::{Element, Length};
use iced_fonts::bootstrap;

pub fn view(app: &App) -> Element<'_, Message> {
    let total = today_total_minutes(app);
    let h = total / 60;
    let m = total % 60;
    row![
        text(format!("Today: {h}h {m}m")),
        text_input("Add task...", &app.new_task_input)
            .on_input(Message::TaskNameChanged)
            .on_submit(Message::SubmitNewTask)
            .width(Length::Fill),
        button(bootstrap::clock_history()).on_press(Message::OpenReview).style(button::text),
    ]
    .padding(8)
    .spacing(8)
    .into()
}

// Sums completed entry minutes for today, plus net minutes for the active entry.
fn today_total_minutes(app: &App) -> i64 {
    let today = Utc::now().format("%Y-%m-%d").to_string();

    let completed: i64 = app
        .entries
        .iter()
        .filter(|e| e.started_at.starts_with(&today) && e.minutes.is_some())
        .map(|e| e.minutes.unwrap_or(0))
        .sum();

    let active: i64 = app
        .active_entry
        .as_ref()
        .filter(|e| e.started_at.starts_with(&today))
        .and_then(|e| {
            let effective_now = if let Some(p) = &e.paused_at {
                DateTime::parse_from_rfc3339(p).ok()?.with_timezone(&Utc)
            } else {
                Utc::now()
            };
            DateTime::parse_from_rfc3339(&e.started_at).ok().map(|dt| {
                let gross = (effective_now - dt.with_timezone(&Utc))
                    .num_minutes()
                    .max(0);
                (gross - e.total_paused).max(0)
            })
        })
        .unwrap_or(0);

    completed + active
}
