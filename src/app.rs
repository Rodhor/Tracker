use crate::data::entry::TimeEntry;
use crate::data::store::{self, AppData};
use crate::data::task::{Task as AppTask, TaskStatus};
use chrono::{DateTime, Utc};
use iced::widget::{button, checkbox, column, container, row, scrollable, stack, text, text_input};
use iced::{Element, Length, Task};
use uuid::Uuid;

enum ReviewRow {
    Entry { entry: TimeEntry, task_name: String },
    Gap { minutes: i64 },
}
// --- The Model ---

pub struct App {
    tasks: Vec<AppTask>,
    entries: Vec<TimeEntry>,
    active_entry: Option<TimeEntry>,
    new_task_input: String,

    // Edit task panel - all None or Empty when no task is being edited
    editing_task_id: Option<Uuid>,
    edit_urgent: bool,
    edit_important: bool,
    edit_description: String,

    // Stop prompt modal state
    stop_prompt_open: bool,
    stop_prompt_note: String,
    stop_prompt_status: TaskStatus,

    // Screens
    // which top-level screen is shown
    screen: Screen,

    // Review view state
    review_date: chrono::NaiveDate,
    editing_note_id: Option<Uuid>,
    editing_note_text: String,
}
// --- Screens ---
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Tracker,
    Review,
}

// --- The Message enum ---

// This Enum contains the possible messages that can be sent to the App
#[derive(Debug, Clone)]
pub enum Message {
    // --- Tracker screen ---
    // Defaults
    Tick,
    TaskNameChanged(String),
    SubmitNewTask,
    StartTimer(Uuid),
    CycleStatus(Uuid),
    PauseTimer,
    ResumeTimer,
    OpenReview,

    // Edit task panel
    OpenEditTask(Uuid),
    CloseEditTask,
    EditUrgentChanged(bool),
    EditImportantChanged(bool),
    EditDescriptionChanged(String),
    SaveEditTask,

    // Stop prompt modal
    OpenStopPrompt,
    StopPromptNoteChange(String),
    StopPromptStatusChanged(TaskStatus),
    ConfirmStop,
    CancelStop,

    // --- Review screen ---
    // Navigation
    CloseReview,
    ReviewPrevDay,
    ReviewNextDay,

    // Inline note editing
    OpenEditNote(Uuid),
    EditNoteChanged(String),
    SaveEditNote,
    CancelEditNote,
}

// --- The App implementation ---

impl App {
    // new() is called to create a new App instance and return it along with a Task<Message>
    // The Task<Message> is used to kick of any async tasks upon app initialization
    pub fn new() -> (Self, Task<Message>) {
        let data = store::load_data().unwrap_or_else(|e| {
            eprintln!("Failed to load data: {e}");
            AppData::default()
        });

        // Iterates through the entries and finds the active one, if any - clones the entry and returns it to the caller
        let active_entry = data.entries.iter().find(|entry| entry.is_active()).cloned();
        let app = Self {
            // Screens
            screen: Screen::Tracker,

            // --- Tracker screen ---
            tasks: data.tasks,
            entries: data.entries,
            active_entry,
            new_task_input: String::new(),

            // Edit task panel - all None or Empty when no task is being edited
            editing_task_id: None,
            edit_urgent: false,
            edit_important: false,
            edit_description: String::new(),

            // Stop prompt modal state
            stop_prompt_open: false,
            stop_prompt_note: String::new(),
            stop_prompt_status: TaskStatus::Todo, // Overwritten when the prompt opens

            // --- Review screen ---
            review_date: chrono::Utc::now().date_naive(),
            editing_note_id: None,
            editing_note_text: String::new(),
        };

        // Task::none() is returned because there are no async tasks to kick off
        (app, Task::none())
    }

    // update() is called to handle incoming messages from the UI or other parts of the app
    // update() is the only public method that modifies the app's state
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // === Tracker screen ===
            // --- Task messages ---
            Message::TaskNameChanged(value) => {
                self.new_task_input = value;
            }
            Message::SubmitNewTask => {
                let name = self.new_task_input.trim().to_string();
                if !name.is_empty() {
                    self.tasks.push(AppTask::new(name));
                    self.new_task_input.clear();
                    self.save();
                }
            }
            Message::CycleStatus(task_id) => {
                if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                    task.status = task.status.next();
                    self.save();
                }
            }

            // --- Timer messages ---
            Message::StartTimer(task_id) => {
                let already_tracking = self
                    .active_entry
                    .as_ref()
                    .is_some_and(|e| e.task_id == task_id);

                if !already_tracking {
                    if self.active_entry.is_some() {
                        self.stop_prompt_open = false;
                        self.stop_prompt_note.clear();
                        self.stop_active_timer(None);
                    }
                    let entry = TimeEntry::new(task_id);
                    self.active_entry = Some(entry.clone());
                    self.entries.push(entry);
                    self.save()
                }
            }
            Message::PauseTimer => self.pause_active_timer(),
            Message::ResumeTimer => self.resume_active_timer(),
            Message::Tick => {}

            // Edit task panel
            Message::OpenEditTask(task_id) => {
                // copy task into temporary fields for editing
                if let Some(task) = self.tasks.iter().find(|t| t.id == task_id) {
                    self.editing_task_id = Some(task.id);
                    self.edit_urgent = task.urgent;
                    self.edit_important = task.important;
                    // Use unwrap_or_default() to avoid None and instead default to empty
                    self.edit_description = task.description.clone().unwrap_or_default();
                }
            }

            Message::CloseEditTask => {
                self.editing_task_id = None;
                self.edit_urgent = false;
                self.edit_important = false;
                self.edit_description.clear();
            }

            Message::EditUrgentChanged(value) => {
                self.edit_urgent = value;
            }

            Message::EditImportantChanged(value) => {
                self.edit_important = value;
            }

            Message::EditDescriptionChanged(value) => {
                self.edit_description = value;
            }

            Message::SaveEditTask => {
                if let Some(id) = self.editing_task_id {
                    if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
                        task.urgent = self.edit_urgent;
                        task.important = self.edit_important;
                        let desc = self.edit_description.trim().to_string();
                        task.description = if desc.is_empty() { None } else { Some(desc) };
                        task.status = task.status.reset_status();
                    }
                }
                self.editing_task_id = None;
                self.edit_urgent = false;
                self.edit_important = false;
                self.edit_description.clear();
                self.save();
            }

            // Stop prompt modal
            Message::OpenStopPrompt => {
                // Defaults to next status unless explicitly told otherwise - on error reading the status, it defaults to Todo
                let next_status = self
                    .active_entry
                    .as_ref()
                    .and_then(|e| self.tasks.iter().find(|t| t.id == e.task_id))
                    .map(|t| t.status.next())
                    .unwrap_or(TaskStatus::Todo);

                self.stop_prompt_open = true;
                self.stop_prompt_note.clear();
                self.stop_prompt_status = next_status;
                self.pause_active_timer();
            }
            Message::StopPromptNoteChange(value) => {
                self.stop_prompt_note = value;
            }
            Message::StopPromptStatusChanged(status) => {
                self.stop_prompt_status = status;
            }
            Message::ConfirmStop => {
                let note = {
                    let s = self.stop_prompt_note.trim().to_string();
                    if s.is_empty() { None } else { Some(s) }
                };
                let new_status = self.stop_prompt_status.clone();

                let current_entry_task_id = self.active_entry.clone().unwrap().task_id;
                self.stop_active_timer(note);

                if let Some(task) = self
                    .tasks
                    .iter_mut()
                    .find(|t| t.id == current_entry_task_id)
                {
                    task.status = new_status;
                }

                self.stop_prompt_open = false;
                self.stop_prompt_note.clear();
                self.save();
            }

            Message::CancelStop => {
                self.stop_prompt_open = false;
                self.stop_prompt_note.clear();
                self.resume_active_timer();
            }

            // === Review screen ===
            Message::OpenReview => {
                self.screen = Screen::Review;
                self.review_date = chrono::Utc::now().date_naive();
                self.editing_note_id = None;
                self.editing_note_text.clear();
            }
            Message::CloseReview => {
                self.screen = Screen::Tracker;
                self.editing_note_id = None;
                self.editing_note_text.clear();
            }
            Message::ReviewPrevDay => {
                self.review_date = self.review_date.pred_opt().unwrap_or(self.review_date);
                // Clear currently navigated edits to avoid conflicts
                self.editing_note_id = None;
                self.editing_note_text.clear();
            }
            // This only moves up until the current day - it does not work for future days
            Message::ReviewNextDay => {
                let today = chrono::Utc::now().date_naive();
                if self.review_date < today {
                    self.review_date = self.review_date.succ_opt().unwrap_or(self.review_date);
                    self.editing_note_id = None;
                    self.editing_note_text.clear();
                }
            }

            Message::OpenEditNote(entry_id) => {
                let current_note = self
                    .entries
                    .iter()
                    .find(|e| e.id == entry_id)
                    .and_then(|e| e.notes.clone())
                    .unwrap_or_default();
                self.editing_note_id = Some(entry_id);
                self.editing_note_text = current_note;
            }
            Message::EditNoteChanged(value) => {
                self.editing_note_text = value;
            }
            Message::SaveEditNote => {
                if let Some(id) = self.editing_note_id {
                    let note = {
                        let s = self.editing_note_text.trim().to_string();
                        if s.is_empty() { None } else { Some(s) }
                    };
                    if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
                        entry.notes = note;
                    }
                    self.editing_note_id = None;
                    self.editing_note_text.clear();
                    self.save();
                }
            }
            Message::CancelEditNote => {
                self.editing_note_id = None;
                self.editing_note_text.clear();
            }
        }

        Task::none()
    }

    // view() is called to render the app's UI
    pub fn view(&self) -> Element<'_, Message> {
        match self.screen {
            Screen::Review => return self.review_screen(),
            Screen::Tracker => {}
        }
        let base =
            container(column![self.timer_bar(), self.task_list(), self.status_bar(),].spacing(0))
                .width(Length::Fill)
                .height(Length::Fill);

        if self.stop_prompt_open {
            stack![base, self.stop_prompt_view(),].into()
        } else {
            base.into()
        }
    }

    // timer_bar is the UI Element that displays the active timer, if any
    fn timer_bar(&self) -> Element<'_, Message> {
        match &self.active_entry {
            Some(entry) => {
                let task_name = self
                    .tasks
                    .iter()
                    .find(|t| t.id == entry.task_id)
                    .map(|t| t.name.as_str())
                    .unwrap_or("Unknown Task");
                let elapsed = Self::elapsed_display(
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

    // task_list is the UI Element that displays the list of tasks (Based on the Row element below)
    fn task_list(&self) -> Element<'_, Message> {
        if self.tasks.is_empty() {
            return container(text("No tasks yet. Add one below."))
                .width(Length::Fill)
                .padding(20)
                .into();
        }

        let mut items: Vec<Element<Message>> = Vec::new();

        let mut sorted: Vec<&AppTask> = self.tasks.iter().filter(|t| t.has_priority()).collect();
        sorted.sort_by_key(|t| t.quadrant());

        if !sorted.is_empty() {
            items.push(
                container(text("TODOS"))
                    .padding(iced::Padding::new(8.0).bottom(4))
                    .into(),
            );
            for task in sorted {
                items.push(self.task_row_or_edit(task))
            }
        }

        let unsorted: Vec<&AppTask> = self.tasks.iter().filter(|t| !t.has_priority()).collect();

        if !unsorted.is_empty() {
            items.push(
                container(text("Needs Sorting"))
                    .padding(iced::Padding::new(8.0).bottom(4))
                    .into(),
            );
            for task in unsorted {
                items.push(self.task_row_or_edit(task));
            }
        }

        scrollable(column(items).spacing(4))
            .height(Length::Fill)
            .into()
    }

    fn task_row_or_edit<'a>(&'a self, task: &'a AppTask) -> Element<'a, Message> {
        if self.editing_task_id == Some(task.id) {
            self.edit_row(task)
        } else {
            self.task_row(task)
        }
    }

    // task_row defines the UI Row Element for a single task
    fn task_row<'a>(&'a self, task: &'a AppTask) -> Element<'a, Message> {
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

    // status_bar is the UI Element that displays the status bar at the bottom of the app
    fn status_bar(&self) -> Element<'_, Message> {
        let total = self.today_total_minutes();
        let h = total / 60;
        let m = total % 60;
        let total_label = text(format!("Today: {h}h {m}m"));
        row![
            total_label,
            text_input("Add task...", &self.new_task_input)
                .on_input(Message::TaskNameChanged)
                .on_submit(Message::SubmitNewTask)
                .width(Length::Fill),
            button(text("Review")).on_press(Message::OpenReview),
        ]
        .padding(8)
        .spacing(8)
        .into()
    }

    // --- Edit Tasks ---
    fn edit_row<'a>(&'a self, task: &'a AppTask) -> Element<'a, Message> {
        column![
            // Fist row: Task name + save / cancel buttons
            row![
                text(&task.name).width(Length::Fill),
                button(text("Save")).on_press(Message::SaveEditTask),
                button(text("Cancel")).on_press(Message::CloseEditTask)
            ]
            .spacing(8),
            row![
                checkbox(self.edit_urgent)
                    .label("Urgent")
                    .on_toggle(Message::EditUrgentChanged),
                checkbox(self.edit_important)
                    .label("Important")
                    .on_toggle(Message::EditImportantChanged),
            ]
            .spacing(16),
            text_input("Description (optional)", &self.edit_description)
                .on_input(Message::EditDescriptionChanged)
                .on_submit(Message::SaveEditTask)
        ]
        .padding(8)
        .spacing(4)
        .into()
    }

    // --- Stop prompt modal ---
    fn stop_prompt_view(&self) -> Element<'_, Message> {
        let task_name = self
            .active_entry
            .as_ref()
            .and_then(|e| self.tasks.iter().find(|t| t.id == e.task_id))
            .map(|t| t.name.as_str())
            .unwrap_or("Unkown task");

        let status_row = row![
            button(text("To do")).on_press(Message::StopPromptStatusChanged(TaskStatus::Todo)),
            button(text("In Progress"))
                .on_press(Message::StopPromptStatusChanged(TaskStatus::InProgress)),
            button(text("Done")).on_press(Message::StopPromptStatusChanged(TaskStatus::Done)),
        ]
        .spacing(8);

        let panel = column![
            text(format!("Stopping: {task_name}")),
            text_input("what did you accomplish?", &self.stop_prompt_note)
                .on_input(Message::StopPromptNoteChange)
                .on_submit(Message::ConfirmStop),
            status_row,
            text(format!("Mark as: {}", self.stop_prompt_status.label())),
            row![
                button(text("Stop and save")).on_press(Message::ConfirmStop),
                button(text("Cancel")).on_press(Message::CancelStop),
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

    // === Review screen ===
    fn review_screen(&self) -> Element<'_, Message> {
        let today = chrono::Utc::now().date_naive();
        let date_label = if self.review_date == today {
            format!("Today - {}", self.review_date.format("%A, %-d %B %Y"))
        } else {
            self.review_date.format("%A, %-d %B %Y").to_string()
        };

        let header = row![
            button(text("<- Prev")).on_press(Message::ReviewPrevDay),
            text(date_label).width(Length::Fill),
            button(text("Next ->"))
                .on_press_maybe((self.review_date < today).then_some(Message::ReviewNextDay),),
            button(text("Close")).on_press(Message::CloseReview),
        ]
        .padding(12)
        .spacing(8);

        let rows = self.build_review_rows();
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
                    let start_str = Self::format_hhmm(&entry.started_at);
                    let end_str = entry
                        .ended_at
                        .as_deref()
                        .map(Self::format_hhmm)
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

                    let note_widget: Element<Message> = if self.editing_note_id == Some(entry.id) {
                        row![
                            text_input("Add a note...", &self.editing_note_text)
                                .on_input(Message::EditNoteChanged)
                                .on_submit(Message::SaveEditNote)
                                .width(Length::Fill),
                            button(text("Save")).on_press(Message::SaveEditNote),
                            button(text("Cancel")).on_press(Message::CancelEditNote),
                        ]
                        .spacing(4)
                        .into()
                    } else {
                        let note_text = entry.notes.clone().unwrap_or_else(|| "-".to_string());
                        row![
                            text(note_text).width(Length::Fill),
                            button(text("Edit")).on_press(Message::OpenEditNote(entry.id)),
                        ]
                        .spacing(4)
                        .into()
                    };

                    let entry_row = row![
                        text(task_name).width(Length::FillPortion(3)),
                        text(format!("{start_str} -> {end_str}")).width(Length::FillPortion(2)),
                        text(format!("{duration_label} {pause_label}"))
                            .width(Length::FillPortion(2)),
                        note_widget,
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

        column![
            header,
            scrollable(column(items).spacing(2).height(Length::Fill)),
            summary,
        ]
        .into()
    }

    // Subscription defines the iced Subscription for the app (currently none)
    // this is used to listen to a stream of messages from the OS or other sources
    // for example, a timer tick
    pub fn subscription(&self) -> iced::Subscription<Message> {
        let should_tick = self
            .active_entry
            .as_ref()
            .map(|e| !e.is_paused())
            .unwrap_or(false);

        if should_tick {
            iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick)
        } else {
            iced::Subscription::none()
        }
    }

    // --- Helper functions ---

    fn save(&self) {
        let data = store::AppData {
            tasks: self.tasks.clone(),
            entries: self.entries.clone(),
        };
        if let Err(e) = store::save_data(&data) {
            eprint!("Failed to save: {e}");
        }
    }

    fn stop_active_timer(&mut self, note: Option<String>) {
        // The take() method removes the active entry from self.active_entry and returns it
        // this way self.active_entry is None while the entry is available for updating
        if let Some(mut entry) = self.active_entry.take() {
            let now = Utc::now();
            if let Some(paused_at_str) = &entry.paused_at {
                if let Ok(paused_at) = DateTime::parse_from_rfc3339(paused_at_str) {
                    let this_pause = (now - paused_at.with_timezone(&Utc)).num_minutes().max(0);
                    entry.total_paused += this_pause;
                }
            }

            // Parse the start time from the entry
            let started = DateTime::parse_from_rfc3339(&entry.started_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(now);

            let gross_minutes = (now - started).num_minutes().max(0);
            entry.ended_at = Some(now.to_rfc3339());
            entry.minutes = Some((gross_minutes - entry.total_paused).max(0));
            entry.notes = note;
            entry.paused_at = None;
            // Update the existing entry if it exists
            if let Some(existing) = self.entries.iter_mut().find(|e| e.id == entry.id) {
                *existing = entry;
            }
        }
    }

    fn elapsed_display(started_at: &str, paused_at: Option<&str>, paused_minutes: i64) -> String {
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

    fn today_total_minutes(&self) -> i64 {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let completed: i64 = self
            .entries
            .iter()
            .filter(|e| e.started_at.starts_with(&today) && e.minutes.is_some())
            .map(|e| e.minutes.unwrap_or(0))
            .sum();

        let active: i64 = self
            .active_entry
            .as_ref()
            .filter(|e| e.started_at.starts_with(&today))
            .and_then(|e| {
                DateTime::parse_from_rfc3339(&e.started_at)
                    .ok()
                    .map(|dt| (Utc::now() - dt.with_timezone(&Utc)).num_minutes().max(0))
            })
            .unwrap_or(0);

        completed + active
    }

    fn pause_active_timer(&mut self) {
        if let Some(entry) = self.active_entry.as_mut() {
            if entry.paused_at.is_none() {
                entry.paused_at = Some(Utc::now().to_rfc3339());
                if let Some(existing) = self.entries.iter_mut().find(|e| e.id == entry.id) {
                    *existing = entry.clone();
                }
            }
        }
    }

    fn resume_active_timer(&mut self) {
        if let Some(entry) = self.active_entry.as_mut() {
            if let Some(paused_at_str) = entry.paused_at.take() {
                if let Ok(paused_at) = DateTime::parse_from_rfc3339(&paused_at_str) {
                    let this_pause = (Utc::now() - paused_at.with_timezone(&Utc))
                        .num_minutes()
                        .max(0);
                    entry.total_paused += this_pause;
                }
                if let Some(existing) = self.entries.iter_mut().find(|e| e.id == entry.id) {
                    *existing = entry.clone();
                }
            }
        }
    }

    fn format_hhmm(rfc3339: &str) -> String {
        DateTime::parse_from_rfc3339(rfc3339)
            .map(|dt| dt.with_timezone(&Utc).format("%H:%M").to_string())
            .unwrap_or_else(|_| "??:??".to_string())
    }

    fn build_review_rows(&self) -> Vec<ReviewRow> {
        let date_str = self.review_date.format("%Y-%m-%d").to_string();
        let mut day_entries: Vec<TimeEntry> = self
            .entries
            .iter()
            .filter(|e| {
                e.started_at.starts_with(&date_str) && (e.ended_at.is_some() || e.is_active())
            })
            .cloned()
            .collect();

        if let Some(active) = &self.active_entry {
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

            let task_name = self
                .tasks
                .iter()
                .find(|t| t.id == entry.task_id)
                .map(|t| t.name.clone())
                .unwrap_or_else(|| "Unknown Task".to_string());

            rows.push(ReviewRow::Entry { entry, task_name });
        }
        rows
    }
}
