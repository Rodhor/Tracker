mod app;
mod data;

use app::{App, Message};

fn main() -> iced::Result {
    iced::application("Tasktracker", App::update, App::view)
        .subscription(App::subscription)
        .run_with(App::new())
}
