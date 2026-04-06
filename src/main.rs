mod app;
mod data;
mod message;
mod timer;
mod ui;

use app::App;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Tasktracker")
        .subscription(App::subscription)
        .run()
}
