use crate::data::entry::TimeEntry;
use crate::data::store::{self, AppData};
use crate::data::task::Task as AppTask;
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Element, Length, Task};
use uuid::Uuid;

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
}

// --- The Message enum ---

// This Enum contains the possible messages that can be sent to the App
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    TaskNameChanged(String),
    SubmitNewTask,
    StartTimer(Uuid),
    CycleStatus(Uuid),
    StopTimer,
    OpenReview,

    // Edit task panel
    OpenEditTask(Uuid),
    CloseEditTask,
    EditUrgentChanged(bool),
    EditImportantChanged(bool),
    EditDescriptionChanged(String),
    SaveEditTask,
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
            tasks: data.tasks,
            entries: data.entries,
            active_entry,
            new_task_input: String::new(),

            // Edit task panel - all None or Empty when no task is beeing edited
            editing_task_id: None,
            edit_urgent: false,
            edit_important: false,
            edit_description: String::new(),
        };

        // Task::none() is returned because there are no async tasks to kick off
        (app, Task::none())
    }

    // update() is called to handle incoming messages from the UI or other parts of the app
    // update() is the only public method that modifies the app's state
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
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
                self.stop_active_timer();
                let entry = TimeEntry::new(task_id);
                self.active_entry = Some(entry.clone());
                self.entries.push(entry);
                self.save();
            }
            Message::StopTimer => {
                self.stop_active_timer();
                self.save();
            }
            Message::Tick => {}

            // --- UI messages ---
            Message::OpenReview => {} // Edit Panel

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
        }
        Task::none()
    }

    // view() is called to render the app's UI
    pub fn view(&self) -> Element<'_, Message> {
        let content = column![self.timer_bar(), self.task_list(), self.status_bar(),].spacing(0);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
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
                let elapsed = Self::elapsed_display(&entry.started_at);

                row![
                    text(format!("{task_name} - {elapsed}")).width(Length::Fill),
                    button(text("Stop")).on_press(Message::StopTimer),
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

    // Subscription defines the iced Subscription for the app (currently none)
    // this is used to listen to a stream of messages from the OS or other sources
    // for example, a timer tick
    pub fn subscription(&self) -> iced::Subscription<Message> {
        if self.active_entry.is_some() {
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

    fn stop_active_timer(&mut self) {
        // The take() method removes the active entry from self.active_entry and returns it
        // this way self.active_entry is None while the entry is available for updating
        if let Some(mut entry) = self.active_entry.take() {
            use chrono::{DateTime, Utc};
            let now = Utc::now();
            // capture the current time as the end time
            entry.ended_at = Some(now.to_rfc3339());

            // Parse the start time from the entry
            let started = DateTime::parse_from_rfc3339(&entry.started_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(now);

            // Calculate the elapsed minutes and update the entry
            // using max(0) to ensure the result is non-negative
            entry.minutes = Some((now - started).num_minutes().max(0));

            // Update the existing entry if it exists
            if let Some(existing) = self.entries.iter_mut().find(|e| e.id == entry.id) {
                *existing = entry;
            }
        }
    }

    fn elapsed_display(started_at: &str) -> String {
        use chrono::{DateTime, Utc};
        let started = DateTime::parse_from_rfc3339(started_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let secs = (Utc::now() - started).num_seconds().max(0);

        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        if h > 0 {
            format!("{h}h {m}m {s}s")
        } else {
            format!("{m}m {s}s")
        }
    }

    fn today_total_minutes(&self) -> i64 {
        use chrono::{DateTime, Utc};
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
}
