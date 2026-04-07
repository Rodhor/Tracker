use crate::data::entry::TimeEntry;
use crate::data::store::{self, AppData};
use crate::data::task::{Task as AppTask, TaskStatus};
pub use crate::message::{Message, Screen};
use crate::ui::quick_add::QUICK_ADD_ID;
use crate::ui::stop_prompt::STOP_PROMPT_ID;
use iced::widget::{Id, column, container, stack, text_editor};
use iced::{Element, Length, Task};
use uuid::Uuid;
// --- The Model ---

pub struct App {
    // Base
    pub(crate) tasks: Vec<AppTask>,
    pub(crate) entries: Vec<TimeEntry>,
    pub(crate) active_entry: Option<TimeEntry>,
    pub(crate) new_task_input: String,

    // Edit task panel - all None or Empty when no task is being edited
    pub(crate) editing_task_id: Option<Uuid>,
    pub(crate) edit_urgent: bool,
    pub(crate) edit_important: bool,
    pub(crate) edit_description_content: text_editor::Content,

    // Stop prompt modal state
    pub(crate) stop_prompt_open: bool,
    pub(crate) stop_prompt_note: text_editor::Content,
    pub(crate) stop_prompt_status: TaskStatus,

    // Review view state
    pub(crate) review_date: chrono::NaiveDate,
    pub(crate) editing_note_id: Option<Uuid>,
    pub(crate) editing_note_content: text_editor::Content,

    // Delete confirmation
    pub(crate) deleting_entry_id: Option<Uuid>,
    pub(crate) deleting_task_id: Option<Uuid>,

    // Live notes
    pub(crate) note_modal_open: bool,
    pub(crate) note_modal_content: text_editor::Content,

    // Quick add Task
    pub(crate) quick_add_open: bool,
    pub(crate) quick_add_input: String,
    pub(crate) quick_add_selected: usize,

    // Screens
    // which top-level screen is shown
    pub(crate) screen: Screen,
}

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
            edit_description_content: text_editor::Content::new(),

            // Stop prompt modal state
            stop_prompt_open: false,
            stop_prompt_note: text_editor::Content::new(),
            stop_prompt_status: TaskStatus::Todo, // Overwritten when the prompt opens

            // Deletion
            deleting_entry_id: None,
            deleting_task_id: None,

            // Live notes
            note_modal_open: false,
            note_modal_content: text_editor::Content::new(),

            // Quick add Task
            quick_add_open: false,
            quick_add_input: String::new(),
            quick_add_selected: 0,

            // --- Review screen ---
            review_date: chrono::Utc::now().date_naive(),
            editing_note_id: None,
            editing_note_content: text_editor::Content::new(),
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
                // Reset quick add
                self.quick_add_open = false;
                self.quick_add_input.clear();
                self.quick_add_selected = 0;

                let already_tracking = self
                    .active_entry
                    .as_ref()
                    .is_some_and(|e| e.task_id == task_id);

                if !already_tracking {
                    if self.active_entry.is_some() {
                        self.stop_prompt_open = false;
                        self.stop_prompt_note = text_editor::Content::new();
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
                self.deleting_task_id = None;
                // copy task into temporary fields for editing
                if let Some(task) = self.tasks.iter().find(|t| t.id == task_id) {
                    self.editing_task_id = Some(task.id);
                    self.edit_urgent = task.urgent;
                    self.edit_important = task.important;
                    // Use unwrap_or_default() to avoid None and instead default to empty
                    self.edit_description_content = text_editor::Content::with_text(
                        &task.description.clone().unwrap_or_default(),
                    );
                }
            }

            Message::CloseEditTask => {
                self.editing_task_id = None;
                self.edit_urgent = false;
                self.edit_important = false;
                self.edit_description_content = text_editor::Content::new();
            }

            Message::EditUrgentChanged(value) => {
                self.edit_urgent = value;
            }

            Message::EditImportantChanged(value) => {
                self.edit_important = value;
            }

            Message::EditDescriptionChanged(action) => {
                self.edit_description_content.perform(action);
            }

            Message::SaveEditTask => {
                if let Some(id) = self.editing_task_id {
                    if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
                        task.urgent = self.edit_urgent;
                        task.important = self.edit_important;
                        let desc = self.edit_description_content.text();
                        let desc = desc.trim().to_string();
                        task.description = if desc.is_empty() { None } else { Some(desc) };
                        task.status = task.status.reset_status();
                    }
                }
                self.editing_task_id = None;
                self.edit_urgent = false;
                self.edit_important = false;
                self.edit_description_content = text_editor::Content::new();
                self.save();
            }

            // Stop prompt modal
            Message::OpenStopPrompt => {
                if self.active_entry.is_some() {
                    // Defaults to next status unless explicitly told otherwise - on error reading the status, it defaults to Todo
                    let next_status = self
                        .active_entry
                        .as_ref()
                        .and_then(|e| self.tasks.iter().find(|t| t.id == e.task_id))
                        .map(|t| t.status.next())
                        .unwrap_or(TaskStatus::Todo);

                    self.stop_prompt_open = true;
                    let existing = self
                        .active_entry
                        .as_ref()
                        .and_then(|e| e.notes.clone())
                        .unwrap_or_default();
                    self.stop_prompt_note = text_editor::Content::with_text(&*existing);
                    self.stop_prompt_status = next_status;
                    self.pause_active_timer();
                    return iced::widget::operation::focus(Id::new(STOP_PROMPT_ID));
                }
            }
            Message::StopPromptNoteChange(action) => {
                self.stop_prompt_note.perform(action);
            }
            Message::StopPromptStatusChanged(status) => {
                self.stop_prompt_status = status;
            }
            Message::ConfirmStop => {
                let note = {
                    let s = self.stop_prompt_note.text();
                    let s = s.trim().to_string();
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
                self.stop_prompt_note = text_editor::Content::new();
                self.stop_prompt_status = TaskStatus::Todo;
                self.save();
            }

            Message::CancelStop => {
                self.stop_prompt_open = false;
                self.stop_prompt_note = text_editor::Content::new();
                self.resume_active_timer();
            }

            // === Review screen ===
            Message::OpenReview => {
                self.screen = Screen::Review;
                self.review_date = chrono::Utc::now().date_naive();
                self.editing_note_id = None;
                self.editing_note_content = text_editor::Content::new();
            }
            Message::CloseReview => {
                self.screen = Screen::Tracker;
                self.editing_note_id = None;
                self.editing_note_content = text_editor::Content::new();
            }
            Message::ReviewPrevDay => {
                self.review_date = self.review_date.pred_opt().unwrap_or(self.review_date);
                // Clear currently navigated edits to avoid conflicts
                self.editing_note_id = None;
                self.editing_note_content = text_editor::Content::new();
            }
            // This only moves up until the current day - it does not work for future days
            Message::ReviewNextDay => {
                let today = chrono::Utc::now().date_naive();
                if self.review_date < today {
                    self.review_date = self.review_date.succ_opt().unwrap_or(self.review_date);
                    self.editing_note_id = None;
                    self.editing_note_content = text_editor::Content::new();
                }
            }

            Message::OpenEditNote(entry_id) => {
                self.deleting_entry_id = None;
                let current_note = self
                    .entries
                    .iter()
                    .find(|e| e.id == entry_id)
                    .and_then(|e| e.notes.clone())
                    .unwrap_or_default();
                self.editing_note_id = Some(entry_id);
                self.editing_note_content = text_editor::Content::with_text(&current_note);
            }
            Message::EditNoteChanged(action) => {
                self.editing_note_content.perform(action);
            }
            Message::SaveEditNote => {
                if let Some(id) = self.editing_note_id {
                    let note = {
                        let s = self.editing_note_content.text();
                        let s = s.trim().to_string();
                        if s.is_empty() { None } else { Some(s) }
                    };
                    if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
                        entry.notes = note;
                    }
                    self.editing_note_id = None;
                    self.editing_note_content = text_editor::Content::new();
                    self.save();
                }
            }
            Message::CancelEditNote => {
                self.editing_note_id = None;
                self.editing_note_content = text_editor::Content::new();
            }

            Message::RequestDeleteEntry(entry_id) => {
                self.editing_note_id = None;
                self.editing_note_content = text_editor::Content::new();
                self.deleting_entry_id = Some(entry_id);
            }
            Message::ConfirmDeleteEntry => {
                if let Some(id) = self.deleting_entry_id {
                    self.entries.retain(|e| e.id != id);
                    self.deleting_entry_id = None;
                    self.save();
                }
            }
            Message::CancelDeleteEntry => {
                self.deleting_entry_id = None;
            }
            Message::RequestDeleteTask(task_id) => {
                self.editing_task_id = None;
                self.edit_description_content = text_editor::Content::new();
                self.deleting_task_id = Some(task_id);
            }
            Message::ConfirmDeleteTask => {
                if let Some(id) = self.deleting_task_id {
                    // If the active timer is for the deleted task, discard it silently
                    if self.active_entry.as_ref().is_some_and(|e| e.task_id == id) {
                        self.active_entry = None;
                        self.stop_prompt_open = false;
                        self.stop_prompt_note = text_editor::Content::new();
                    }

                    // Cascade: remove all entries that belonged to the deleted task
                    self.entries.retain(|e| e.task_id != id);
                    // Delete the task itself
                    self.tasks.retain(|e| e.id != id);
                    self.deleting_task_id = None;
                    self.save();
                }
            }
            Message::CancelDeleteTask => {
                if let Some(id) = self.deleting_task_id {
                    if let Some(task) = self.tasks.iter().find(|t| t.id == id) {
                        self.editing_task_id = Some(task.id);
                        self.edit_urgent = task.urgent;
                        self.edit_important = task.important;
                        self.edit_description_content = text_editor::Content::with_text(
                            &task.description.clone().unwrap_or_default(),
                        );
                    }
                }
                self.deleting_task_id = None;
            }

            Message::OpenNoteModal => {
                if self.active_entry.is_some() {
                    let existing = self
                        .active_entry
                        .as_ref()
                        .and_then(|e| e.notes.clone())
                        .unwrap_or_default();
                    self.note_modal_content = text_editor::Content::with_text(&existing);

                    self.note_modal_open = true;
                }
            }
            Message::NoteModalChanged(action) => {
                self.note_modal_content.perform(action);
            }
            Message::SaveNoteModal => {
                if let Some(entry) = self.active_entry.as_mut() {
                    let note = self.note_modal_content.text();
                    let note = note.trim().to_string();
                    entry.notes = if note.is_empty() { None } else { Some(note) };
                    if let Some(existing) = self.entries.iter_mut().find(|e| e.id == entry.id) {
                        *existing = entry.clone();
                    }
                }
                self.note_modal_open = false;
                self.note_modal_content = text_editor::Content::new();
                self.save();
            }
            Message::CancelNoteModal => {
                self.note_modal_open = false;
                self.note_modal_content = text_editor::Content::new();
            }
            Message::SubmitActiveEditor => {
                if self.note_modal_open {
                    return Task::done(Message::SaveNoteModal);
                } else if self.stop_prompt_open {
                    return Task::done(Message::ConfirmStop);
                } else if self.editing_note_id.is_some() {
                    return Task::done(Message::SaveEditNote);
                } else if self.editing_task_id.is_some() {
                    return Task::done(Message::SaveEditTask);
                }
            }
            Message::OpenQuickAdd => {
                self.quick_add_open = true;
                self.quick_add_input.clear();
                self.quick_add_selected = 0;
                use iced::widget::Id;
                return iced::widget::operation::focus(Id::new(QUICK_ADD_ID));
            }
            Message::CloseQuickAdd => {
                self.quick_add_open = false;
                self.quick_add_input.clear();
                self.quick_add_selected = 0;
            }
            Message::QuickAddInputChanged(value) => {
                self.quick_add_input = value;
                self.quick_add_selected = 0;
            }
            Message::QuickAddMoveUp => {
                if self.quick_add_selected > 0 {
                    self.quick_add_selected -= 1
                }
            }
            Message::QuickAddMoveDown => {
                self.quick_add_selected = self.quick_add_selected.saturating_add(1)
            }
            Message::QuickAddConfirm => {
                if self.quick_add_open {
                    let input = self.quick_add_input.trim().to_string();
                    let filtered: Vec<_> = self
                        .tasks
                        .iter()
                        .filter(|t| t.name.to_lowercase().contains(&input.to_lowercase()))
                        .collect();

                    let task_id = if self.quick_add_selected < filtered.len() {
                        filtered[self.quick_add_selected].id
                    } else if !input.is_empty() {
                        let task = crate::data::task::Task::new(input.clone());
                        let id = task.id;
                        self.tasks.push(task);
                        id
                    } else {
                        return Task::none();
                    };

                    self.quick_add_open = false;
                    self.quick_add_input.clear();
                    self.quick_add_selected = 0;

                    return Task::done(Message::StartTimer(task_id));
                }
            }
            Message::TogglePauseTimer => {
                if let Some(entry) = &self.active_entry {
                    if entry.is_paused() {
                        return Task::done(Message::ResumeTimer);
                    } else {
                        return Task::done(Message::PauseTimer);
                    }
                }
            }
            Message::ToggleReview => {
                return Task::done(match self.screen {
                    Screen::Review => Message::CloseReview,
                    Screen::Tracker => Message::OpenReview,
                });
            }
            Message::CloseActiveModal => {
                if self.stop_prompt_open {
                    return Task::done(Message::CancelStop);
                } else if self.note_modal_open {
                    return Task::done(Message::CancelNoteModal);
                } else if self.quick_add_open {
                    return Task::done(Message::CloseQuickAdd);
                } else if self.screen == Screen::Review {
                    return Task::done(Message::CloseReview);
                }
            }
        }

        Task::none()
    }

    // view() is called to render the app's UI
    pub fn view(&self) -> Element<'_, Message> {
        use crate::ui;

        match self.screen {
            Screen::Review => return ui::review::view(self),
            Screen::Tracker => {}
        }
        let base = container(
            column![
                ui::timer_bar::view(self),
                ui::task_list::view(self),
                ui::status_bar::view(self),
            ]
            .spacing(0),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        match (
            self.stop_prompt_open,
            self.note_modal_open,
            self.quick_add_open,
        ) {
            (true, false, false) => stack![base, ui::stop_prompt::view(self)].into(),
            (false, true, false) => stack![base, ui::note_modal::view(self)].into(),
            (false, false, true) => stack![base, ui::quick_add::view(self)].into(),
            _ => base.into(),
        }
    }

    // Subscription defines the iced Subscription for the app
    // this is used to listen to a stream of messages from the OS or other sources
    // for example, a timer tick
    pub fn subscription(&self) -> iced::Subscription<Message> {
        let tick = {
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
        };
        let keys = iced::event::listen_with(|event, status, _window| {
            use iced::event::Status;
            use iced::keyboard::{self, key::Named};
            if let iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event
            {
                match key {
                    keyboard::Key::Named(Named::Enter)
                        if modifiers.control() || modifiers.shift() =>
                    {
                        Some(Message::SubmitActiveEditor)
                    }
                    keyboard::Key::Named(Named::ArrowUp) => Some(Message::QuickAddMoveUp),
                    keyboard::Key::Named(Named::ArrowDown) => Some(Message::QuickAddMoveDown),
                    keyboard::Key::Named(Named::Escape) => Some(Message::CloseActiveModal),
                    keyboard::Key::Character(ref c) if status == Status::Ignored => {
                        match c.as_str() {
                            "q" | "Q" => Some(Message::OpenQuickAdd),
                            "n" | "N" => Some(Message::OpenNoteModal),
                            "p" | "P" => Some(Message::TogglePauseTimer),
                            "s" | "S" => Some(Message::OpenStopPrompt),
                            "r" | "R" => Some(Message::ToggleReview),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            } else {
                None
            }
        });
        iced::Subscription::batch([tick, keys])
    }

    // --- Helper functions ---

    fn save(&self) {
        let data = AppData {
            tasks: self.tasks.clone(),
            entries: self.entries.clone(),
        };
        if let Err(e) = store::save_data(&data) {
            eprint!("Failed to save: {e}");
        }
    }
}
