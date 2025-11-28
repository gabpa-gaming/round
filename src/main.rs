use std::sync::mpsc::Sender;

use dioxus::{desktop::{Config}, prelude::*};

pub mod app;
pub mod player_playing_state;
pub mod player;
pub mod file_browser;
pub mod queue_bar;
pub mod queue_state;
pub mod db;
pub mod app_context;
pub mod errors;
pub mod audio_controller;
pub mod audio_controller_command;
pub mod context_menu;
pub mod playlist;
pub mod playlist_browser;
pub mod create_playlist_dialog;

use crate::{app::App, app_context::PlayerContext, audio_controller::AudioController, audio_controller_command::AudioControllerCommand};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
    dioxus::LaunchBuilder::new()
        .with_cfg(
            Config::new()
                .with_window(
                    dioxus::desktop::WindowBuilder::new().with_title("Round")
                )
        )
        .launch(App);
}

