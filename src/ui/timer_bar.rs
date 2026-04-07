use crate::app::{App, Message};
use chrono::{DateTime, Utc};
use iced::widget::{button, container, row, text};
use iced::{Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    match &app.active_entry {
        Some(entry) => {
            let task_name = app
                .tasks
                .iter()
                .find(|t| t.id == entry.task_id)
                .map(|t| t.name.as_str())
                .unwrap_or("Unknown Task");
            let elapsed = elapsed_display(
                &entry.started_at,
                entry.paused_at.as_deref(),
                entry.total_paused,
            );

            let pause_resume_btn = if entry.is_paused() {
                button(text("Resume")).on_press(Message::ResumeTimer)
            } else {
                button(text("Pause")).on_press(Message::PauseTimer)
            };

            row![
                text(format!("{task_name} - {elapsed}")).width(Length::Fill),
                button(text("Note")).on_press(Message::OpenNoteModal),
                pause_resume_btn,
                button(text("Stop")).on_press(Message::OpenStopPrompt),
            ]
            .padding(12)
            .spacing(8)
            .into()
        }
        None => container(text("No timer running"))
            .width(Length::Fill)
            .padding(12)
            .into(),
    }
}

// Computes a net elapsed string, subtracting accumulated pause time.
// When paused, effective_now is frozen at paused_at so the display stops ticking.
pub fn elapsed_display(started_at: &str, paused_at: Option<&str>, paused_minutes: i64) -> String {
    let started = DateTime::parse_from_rfc3339(started_at)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let effective_now = if let Some(p) = paused_at {
        DateTime::parse_from_rfc3339(p)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now())
    } else {
        Utc::now()
    };

    let gross_secs = (effective_now - started).num_seconds().max(0);
    let pause_secs = paused_minutes * 60;
    let net_secs = (gross_secs - pause_secs).max(0);

    let h = net_secs / 3600;
    let m = (net_secs % 3600) / 60;
    let s = net_secs % 60;
    if h > 0 {
        format!("{h}h {m}m {s}s")
    } else {
        format!("{m}m {s}s")
    }
}
