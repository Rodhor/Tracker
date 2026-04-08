use crate::app::{App, Message};
use crate::data::entry::TimeEntry;
use chrono::{DateTime, Local, Utc};
use iced::widget::{button, column, container, row, scrollable, text, text_editor, text_input};
use iced::{Element, Length};
use iced_fonts::bootstrap;

pub enum ReviewRow {
    Entry { entry: TimeEntry, task_name: String },
    Gap { minutes: i64 },
}
pub fn view(app: &App) -> Element<'_, Message> {
    let today = chrono::Utc::now().date_naive();
    let date_label = if app.review_date == today {
        format!("Today - {}", app.review_date.format("%A, %-d %B %Y"))
    } else {
        app.review_date.format("%A, %-d %B %Y").to_string()
    };

    let header = row![
        button(bootstrap::chevron_left()).on_press(Message::ReviewPrevDay).style(button::text),
        text(date_label).width(Length::Fill),
        button(bootstrap::chevron_right())
            .style(button::text)
            .on_press_maybe((app.review_date < today).then_some(Message::ReviewNextDay),),
        button(bootstrap::x_lg()).on_press(Message::CloseReview).style(button::text),
    ]
    .padding(12)
    .spacing(8);

    let rows = build_review_rows(app);
    let mut total_tracked: i64 = 0;
    let mut total_gap: i64 = 0;
    let mut first_start: Option<String> = None;
    let mut last_end: Option<String> = None;
    let mut items: Vec<Element<Message>> = Vec::new();
    if rows.is_empty() {
        items.push(
            container(text("No entries for this day."))
                .padding(20)
                .into(),
        );
    }

    for row_item in rows {
        match row_item {
            ReviewRow::Entry { entry, task_name } => {
                let start_str = format_hhmm(&entry.started_at);
                let end_str = entry
                    .ended_at
                    .as_deref()
                    .map(format_hhmm)
                    .unwrap_or_else(|| "currently active".to_string());
                let net_min = entry.minutes.unwrap_or_else(|| {
                    DateTime::parse_from_rfc3339(&entry.started_at)
                        .map(|dt| (Utc::now() - dt.with_timezone(&Utc)).num_minutes().max(0))
                        .unwrap_or(0)
                });
                let h = net_min / 60;
                let m = net_min % 60;
                let duration_label = if h > 0 {
                    format! {"{h}h {m}m"}
                } else {
                    format!("{m}m")
                };

                let pause_label = if entry.total_paused > 0 {
                    format!("({} min paused)", entry.total_paused)
                } else {
                    String::new()
                };

                total_tracked += net_min;
                if first_start.is_none() {
                    first_start = Some(start_str.clone());
                };
                last_end = Some(end_str.clone());

                if app.deleting_entry_id == Some(entry.id) {
                    let confirm_row = row![
                        text(format!("Delete '{task_name}' ({start_str} -> {end_str})?"))
                            .width(Length::Fill),
                        button(text("Confirm delete")).on_press(Message::ConfirmDeleteEntry).style(button::danger),
                        button(bootstrap::x_circle()).on_press(Message::CancelDeleteEntry).style(button::text)
                    ]
                    .padding(8)
                    .spacing(8);
                    items.push(confirm_row.into());
                    continue; // skip the rest of the current loop
                }
                if app.editing_time_id == Some(entry.id) {
                    let time_edit = row![
                        text(task_name).width(Length::Fill),
                        text_input("HH:MM", &app.edit_time_start)
                            .on_input(Message::EditTimeStartChanged)
                            .width(Length::Fixed(60.0)),
                        text(" -> "),
                        text_input("HH:MM", &app.edit_time_end)
                            .on_input(Message::EditTimeEndChanged)
                            .width(Length::Fixed(60.0)),
                        button(bootstrap::check())
                            .on_press(Message::SaveEditTime)
                            .style(button::primary),
                        button(bootstrap::x_circle())
                            .on_press(Message::CancelEditTime)
                            .style(button::text),
                    ]
                    .padding(8)
                    .spacing(8);
                    items.push(time_edit.into());
                    continue; // skip the rest of the current loop
                }

                let note_widget: Element<Message> = if app.editing_note_id == Some(entry.id) {
                    row![
                        text_editor(&app.editing_note_content)
                            .on_action(Message::EditNoteChanged)
                            .height(Length::Fixed(80.0)),
                        button(bootstrap::check()).on_press(Message::SaveEditNote).style(button::primary),
                        button(bootstrap::x_circle()).on_press(Message::CancelEditNote).style(button::text),
                    ]
                    .spacing(4)
                    .into()
                } else {
                    // Regular row - note widget and delete button
                    let note_text = entry.notes.clone().unwrap_or_else(|| "-".to_string());
                    let copy_btn = entry.notes.as_ref().map(|note| {
                        button(bootstrap::clipboard()).on_press(Message::CopyEntryNote(note.clone()))
                    });
                    column![
                        text(note_text).width(Length::Fill),
                        row![
                            button(bootstrap::pencil()).on_press(Message::OpenEditNote(entry.id)),
                            if let Some(btn) = copy_btn {
                                btn
                            } else {
                                button(bootstrap::clipboard())
                            },
                            button(bootstrap::clock_history()).on_press_maybe(
                                entry
                                    .ended_at
                                    .as_ref()
                                    .map(|_| Message::OpenEditTime(entry.id))
                            ),
                            button(bootstrap::trash())
                                .style(button::danger)
                                .on_press_maybe(
                                    entry
                                        .ended_at
                                        .as_ref()
                                        .map(|_| Message::RequestDeleteEntry(entry.id))
                                ),
                        ]
                        .spacing(4),
                    ]
                    .spacing(4)
                    .width(Length::Fill)
                    .into()
                };

                let entry_row = row![
                    text(task_name).width(Length::FillPortion(2)),
                    text(format!("{start_str} -> {end_str}")).width(Length::FillPortion(2)),
                    text(format!("{duration_label} {pause_label}")).width(Length::FillPortion(1)),
                    container(note_widget).width(Length::FillPortion(3)),
                ]
                .padding(8)
                .spacing(8);

                items.push(entry_row.into());
            }
            ReviewRow::Gap { minutes } => {
                let h = minutes / 60;
                let m = minutes % 60;
                let gap_label = if h > 0 {
                    format!("⊘  Untracked — {h}h {m}m")
                } else {
                    format!("⊘  Untracked — {m}m")
                };
                total_gap += minutes;
                items.push(
                    container(text(gap_label))
                        .padding(iced::Padding::new(4.0).left(16))
                        .into(),
                );
            }
        }
    }
    let first_str = first_start.as_deref().unwrap_or("-");
    let last_str = last_end.as_deref().unwrap_or("-");
    let tracked_h = total_tracked / 60;
    let tracked_m = total_tracked % 60;
    let gap_h = total_gap / 60;
    let gap_m = total_gap % 60;

    let summary = row![
        text(format!("First: {first_str}")),
        text(format!("Last: {last_str}")),
        text(format!("Tracked: {tracked_h}h {tracked_m}m")),
        text(format!("Untracked: {gap_h}h {gap_m}m")),
    ]
    .padding(8)
    .spacing(16);

    iced::widget::column![
        header,
        scrollable(column(items).spacing(2).height(Length::Fill)),
        summary,
    ]
    .into()
}

pub fn format_hhmm(rfc3339: &str) -> String {
    DateTime::parse_from_rfc3339(rfc3339)
        .map(|dt| dt.with_timezone(&Local).format("%H:%M").to_string())
        .unwrap_or_else(|_| "??:??".to_string())
}

// Builds the ordered list of entry and gap rows for the selected day.
// Gaps >= 5 minutes between consecutive entries are surfaced as ReviewRow::Gap.
pub fn build_review_rows(app: &App) -> Vec<ReviewRow> {
    let date_str = app.review_date.format("%Y-%m-%d").to_string();
    let mut day_entries: Vec<TimeEntry> = app
        .entries
        .iter()
        .filter(|e| e.started_at.starts_with(&date_str) && (e.ended_at.is_some() || e.is_active()))
        .cloned()
        .collect();

    if let Some(active) = &app.active_entry {
        if active.started_at.starts_with(&date_str) {
            if !day_entries.iter().any(|e| e.id == active.id) {
                day_entries.push(active.clone());
            }
        }
    }

    // Uses cmp to sort entries chronologically
    day_entries.sort_by(|a, b| a.started_at.cmp(&b.started_at));

    let mut rows: Vec<ReviewRow> = Vec::new();
    let mut prev_end: Option<DateTime<Utc>> = None;

    for entry in day_entries {
        if let Some(prev) = prev_end {
            if let Ok(this_start) = DateTime::parse_from_rfc3339(&entry.started_at) {
                let gap_minutes = (this_start.with_timezone(&Utc) - prev).num_minutes();
                if gap_minutes >= 5 {
                    rows.push(ReviewRow::Gap {
                        minutes: gap_minutes,
                    });
                }
            }
        }

        if let Some(ended) = &entry.ended_at {
            if let Ok(dt) = DateTime::parse_from_rfc3339(ended) {
                prev_end = Some(dt.with_timezone(&Utc));
            }
        } else {
            prev_end = Some(Utc::now());
        }

        let task_name = app
            .tasks
            .iter()
            .find(|t| t.id == entry.task_id)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "Unknown Task".to_string());

        rows.push(ReviewRow::Entry { entry, task_name });
    }
    rows
}
