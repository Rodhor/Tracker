use crate::data::entry::TimeEntry;
use crate::data::store::{self, AppData};
use crate::data::task::Task as AppTask;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length, Task};
use uuid::Uuid;

// --- The Model ---

pub struct App {
    tasks: Vec<AppTask>,
    entries: Vec<TimeEntry>,
    active_entry: Option<TimeEntry>,
    new_task_input: String,
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
            Message::OpenReview => {}
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

        let rows: Vec<Element<Message>> =
            self.tasks.iter().map(|task| self.task_row(task)).collect();

        scrollable(column(rows).spacing(4))
            .height(Length::Fill)
            .into()
    }

    // task_row defines the UI Row Element for a single task
    fn task_row<'a>(&'a self, task: &'a AppTask) -> Element<'a, Message> {
        row![
            button(text("▶")).on_press(Message::StartTimer(task.id)),
            text(&task.name).width(Length::Fill),
            button(text(task.status.label())).on_press(Message::CycleStatus(task.id))
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

    // Subscription defines the iced Subscription for the app (currently none)
    // this is used to listen to a stream of messages from the OS or other sources
    // forexample, a timer tick
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
