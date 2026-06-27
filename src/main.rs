mod app;
mod models;
mod services;
mod views;

use app::EtcherusApp;
use iced::{Application, Settings};

fn main() -> iced::Result {
    EtcherusApp::run(Settings::default())
}
