use crate::data::entry::TimeEntry;
use crate::data::store::{self, AppData};
use crate::data::task::{Task as AppTask, TaskStatus};
use iced::widget::{column, container, row, text};
use iced::{Element, Length, Task};

// --- The Model ---

pub struct App {
    tasks: Vec<AppTask>,
    entries: Vec<TimeEntry>,
    active_entry: Option<TimeEntry>,
}

// --- The Message enum ---

// This Enum contains the possible messages that can be sent to the App
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    AddTask,
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
        };

        // Task::none() is returned because there are no async tasks to kick off
        (app, Task::none())
    }

    // update() is called to handle incoming messages from the UI or other parts of the app
    // update() is the only public method that modifies the app's state
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {}
            Message::AddTask => {}
            Message::OpenReview => {}
        }
    }

    // view() is called to render the app's UI
    pub fn view(&self) -> Element<Message> {
        let content = column![self.timer_bar(), self.task_list(), self.status_bar(),].spacing(0);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    // timer_bar is the UI Element that displays the active timer, if any
    fn timer_bar(&self) -> Element<Message> {
        let label = match &self.active_entry {
            Some(entry) => format!("Timer running - Task{}", entry.task_id),
            None => "No timer running".to_string(),
        };

        container(text(label))
            .width(Length::Fill)
            .padding(12)
            .into()
    }

    // task_list is the UI Element that displays the list of tasks (Based on the Row element below)
    fn task_list(&self) -> Element<Message> {
        if self.tasks.is_empty() {
            return container(text("No tasks yet. Add one below."))
                .width(Length::Fill)
                .padding(20)
                .into();
        }

        let rows: Vec<Element<Message>> =
            self.tasks.iter().map(|task| self.task_row(task)).collect();

        column(rows).spacing(4).into()
    }

    // task_row defines the UI Row Element for a single task
    fn task_row(&self, task: &AppTask) -> Element<Message> {
        row![
            text(&task.name).width(Length::Fill),
            text(task.status.label()),
        ]
        .padding(8)
        .spacing(8)
        .into()
    }

    // status_bar is the UI Element that displays the status bar at the bottom of the app
    fn status_bar(&self) -> Element<Message> {
        let total_label = text("Today: 0h 0m");

        row![total_label,].padding(8).into()
    }

    // Subscription defines the iced Subscription for the app (currently none)
    // this is used to listen to a stream of messages from the OS or other sources
    // forexample, a timer tick
    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }
}
