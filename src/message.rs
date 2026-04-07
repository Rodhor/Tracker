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
    EditDescriptionChanged(iced::widget::text_editor::Action),
    SaveEditTask,

    // Deletion
    RequestDeleteTask(Uuid),
    ConfirmDeleteTask,
    CancelDeleteTask,

    // Stop prompt modal
    OpenStopPrompt,
    StopPromptNoteChange(iced::widget::text_editor::Action),
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
    EditNoteChanged(iced::widget::text_editor::Action),
    SaveEditNote,
    CancelEditNote,

    // Deletion
    RequestDeleteEntry(Uuid),
    ConfirmDeleteEntry,
    CancelDeleteEntry,

    // Live notes
    OpenNoteModal,
    NoteModalChanged(iced::widget::text_editor::Action),
    SaveNoteModal,
    CancelNoteModal,

    // Keyboard shortcuts
    SubmitActiveEditor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Tracker,
    Review,
}
