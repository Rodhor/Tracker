mod app;
mod data;

use app::App;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Tasktracker")
        .subscription(App::subscription)
        .run()
}
