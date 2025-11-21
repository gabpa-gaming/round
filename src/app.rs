use dioxus::{html::audio, prelude::*};
use directories::UserDirs;
use tokio::sync::mpsc::{Sender, channel, unbounded_channel};

use crate::{FAVICON, MAIN_CSS, app_context::{DatabaseContext, PlayerContext}, audio_controller::{self, AudioController}, audio_controller_command::AudioControllerCommand, file_browser::file_browser, player::player_sidebar, queue_bar::queue_bar};

#[component]
pub fn App() -> Element {
    let music_dir = if let Some(user_dirs) = UserDirs::new() {
        if let Some(music_dir) = user_dirs.audio_dir() {
            music_dir.to_str().unwrap_or("/").to_string()
        } else {
            "/".to_string()
        }
    } else {
        "/".to_string()
    };

    let ctx = use_hook(|| -> PlayerContext {
    
        let (cmd_snd, cmd_rcv) = channel::<AudioControllerCommand>(10);
        
        let player_ctx = PlayerContext::new(cmd_snd.clone());
        
        spawn(async move {
            let mut audio = AudioController::new(cmd_rcv, player_ctx.playing_state.clone()); 
            audio.run().await;
        });

        player_ctx
    });

    let player_context = use_context_provider(|| {
        ctx
    });

    let db = use_context_provider(|| DatabaseContext::new());


    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        div {
            class: "app-container",
            div {
                class: "main-area-wrapper",
                file_browser { starting_path: music_dir }
                queue_bar { }
            }
            player_sidebar { }    
        }
    }
}

