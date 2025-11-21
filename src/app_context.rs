use std::sync::Arc;

use dioxus::desktop::wry::cookie::time::Duration;
use dioxus::desktop::wry::cookie::time::format_description::modifier::WeekNumber;
use dioxus::prelude::*;
use rodio::{OutputStream, Sink};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::audio_controller_command::AudioControllerCommand;
use crate::db::Db;

use crate::player_playing_state::PlayerPlayingState;
use crate::queue_state::QueueState;

#[derive(Clone)]
pub struct DatabaseContext(std::sync::Arc<Db>);

impl DatabaseContext {
    pub fn new() -> Self {
        DatabaseContext(Arc::new(Db::new()))
    }
    
    pub fn get(&self) -> &Db {
        &self.0
    }
}

impl PartialEq for DatabaseContext {
    fn eq(&self, _other: &Self) -> bool {
        true 
    }
}

#[derive(Clone)]
pub struct PlayerContext {
    pub playing_state: Signal<PlayerPlayingState>,
    pub volume: Signal<f32>,
    pub speed: Signal<f32>,
    pub loop_: Signal<bool>,
    pub shuffle: Signal<bool>,
    command_sender: Sender<AudioControllerCommand>,
    pub queue: Signal<QueueState>,
}

impl PlayerContext {
    pub fn new(sender: Sender<AudioControllerCommand>) -> Self {
        let playing_state = Signal::new(PlayerPlayingState::NoSongSelected);
        PlayerContext {
            playing_state: playing_state.clone(),
            volume: Signal::new(1.0),
            speed: Signal::new(1.0),
            loop_: Signal::new(false),
            shuffle: Signal::new(false),
            command_sender: sender.clone(),
            queue: Signal::new(QueueState::new(playing_state.clone(), sender.clone())),
        }
    }

    pub fn play(&mut self) {
        self.send_cmd(AudioControllerCommand::Play);
        self.playing_state.write().play();
    }

    pub fn pause(&mut self) {
        self.send_cmd(AudioControllerCommand::Pause);
        self.playing_state.write().pause();
    }

    pub fn set_current_progress(&self, progress: u64) {
        self.send_cmd(AudioControllerCommand::SetProgress(progress));
    }

    pub fn is_finished(&self) -> bool {
        matches!(*self.playing_state.read(), PlayerPlayingState::SongFinished { .. })
    }

    fn send_cmd(&self, cmd: AudioControllerCommand) {
        let sender = self.command_sender.clone();
        tokio::spawn(async move {
            if let Err(e) = sender.send(cmd).await {
                eprintln!("Failed to send audio controller command: {:?}", e);
            }
        });
    }
}

impl PartialEq for PlayerContext {
    fn eq(&self, _other: &Self) -> bool {
        true 
    }
}

