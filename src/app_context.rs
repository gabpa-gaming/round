use std::sync::Arc;


use dioxus::prelude::*;
use tokio::sync::mpsc::{Sender};

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

    pub fn arc(&self) -> Arc<Db> {
        self.0.clone()
    }
}

impl PartialEq for DatabaseContext {
    fn eq(&self, _other: &Self) -> bool {
        true 
    }
}

#[derive(Clone, Copy, Debug, Eq)]
pub enum PlaybackMode {
    Normal,
    Loop,
    Shuffle,
}

impl PartialEq for PlaybackMode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PlaybackMode::Normal, PlaybackMode::Normal) => true,
            (PlaybackMode::Loop, PlaybackMode::Loop) => true,
            (PlaybackMode::Shuffle, PlaybackMode::Shuffle) => true,
            _ => false,
        }
    }
    
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
#[derive(Clone)]
pub struct PlayerContext {
    pub playing_state: Signal<PlayerPlayingState>,
    volume: Signal<f32>,
    pub speed: Signal<f32>,
    pub mode: Signal<PlaybackMode>,
    command_sender: Sender<AudioControllerCommand>,
    pub playlist_update_counter: Signal<u64>,
    pub queue: Signal<QueueState>,
}

impl PlayerContext {
    pub fn new(sender: Sender<AudioControllerCommand>) -> Self {
        let playing_state = Signal::new(PlayerPlayingState::NoSongSelected);
        let mode = Signal::new(PlaybackMode::Normal);
        PlayerContext {
            playing_state: playing_state.clone(),
            volume: Signal::new(1.0),
            speed: Signal::new(1.0),
            mode: mode.clone(),
            command_sender: sender.clone(),
            playlist_update_counter: Signal::new(0),
            queue: Signal::new(QueueState::new(playing_state.clone(), sender.clone(), mode.clone())),
        }
    }

    pub fn previous_song(&mut self) {
        self.queue.write().previous_song();
    }

    pub fn next_song(&mut self) {
        self.queue.write().next_song();
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.send_cmd(AudioControllerCommand::SetVolume(volume));
        self.volume.set(volume);
        println!("Set volume to {}", volume);
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.read()
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

