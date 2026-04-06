use crate::data::task::TaskStatus;
use uuid::Uuid;

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

    // Deletion
    RequestDeleteTask(Uuid),
    ConfirmDeleteTask,
    CancelDeleteTask,

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

    // Deletion
    RequestDeleteEntry(Uuid),
    ConfirmDeleteEntry,
    CancelDeleteEntry,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Tracker,
    Review,
}
