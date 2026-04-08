#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod app;
mod data;
mod message;
mod timer;
mod ui;
use iced::window;
use iced_fonts::BOOTSTRAP_FONT_BYTES;

use app::App;

fn app_theme() -> iced::Theme {
    iced::Theme::KanagawaWave
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Tasktracker")
        .subscription(App::subscription)
        .theme(|_: &App| app_theme())
        .font(BOOTSTRAP_FONT_BYTES)
        .window(window::Settings {
            size: iced::Size::new(800.0, 550.0),
            min_size: Some(iced::Size::new(640.0, 420.0)),
            ..window::Settings::default()
        })
        .run()
}
