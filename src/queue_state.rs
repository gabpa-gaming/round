use std::collections::VecDeque;

use dioxus::prelude::*;
use tokio::sync::mpsc::Sender;

use crate::{audio_controller_command::AudioControllerCommand, db::SongView, player_playing_state::PlayerPlayingState};

pub struct QueueState {
    pub play_next_queue: Signal<VecDeque<SongView>>,
    pub current_fallback_queue: Signal<VecDeque<SongView>>,
    pub last_played: Signal<VecDeque<SongView>>,
    pub playing_state: Signal<PlayerPlayingState>,
    command_sender: Sender<AudioControllerCommand> 
}

impl QueueState {
    pub fn new(playing_state: Signal<PlayerPlayingState>, command_sender: Sender<AudioControllerCommand>) -> Self {
        //todo need to load from db here
        QueueState {
            play_next_queue: Signal::new(VecDeque::new()),
            current_fallback_queue: Signal::new(VecDeque::new()),
            last_played: Signal::new(VecDeque::new()),
            playing_state,
            command_sender
        }
    }

    pub fn get_names(&self) -> Vec<String> {
        let play_next = self.play_next_queue.clone();
        let fallback = self.current_fallback_queue.clone(); 
        play_next.cloned().iter()
            .map(|item| item.title.clone()).collect()
    }

    pub fn next_song(&mut self) {
        let next = if let Some(next_song) = self.play_next_queue.write().pop_front() {
            next_song
        } else if let Some(next_song) = self.current_fallback_queue.write().pop_front() {
            next_song
        } else {
            eprintln!("Queue empty");
            *self.playing_state.write() = PlayerPlayingState::NoSongSelected;
            return;
        };
        self.play_song_instant(&next);
    }

    pub fn previous_song(&mut self) {
        let prev = if let Some(prev_song) = self.last_played.write().pop_back() {
            prev_song
        } else { 
            eprintln!("History empty");
            *self.playing_state.write() = PlayerPlayingState::NoSongSelected;
            return;
        };
        self.play_song_instant(&prev);
    }

    pub fn play_song_instant(&mut self, song: &SongView) {
        self.send_cmd(AudioControllerCommand::Load(song.path.clone()));
        self.send_cmd(AudioControllerCommand::Play);
        *self.playing_state.write() = PlayerPlayingState::Playing { song: song.clone(), progress_ms: 0};
    }

    pub fn set_to_play_next(&mut self, song: &SongView) {
        self.play_next_queue.write().push_back(song.clone());
        if !self.playing_state.read().is_playing() {
            self.next_song();
        }
    }

    pub fn send_cmd(&self, cmd: AudioControllerCommand) {
        let sender = self.command_sender.clone();
        tokio::spawn(async move {
            if let Err(e) = sender.send(cmd).await {
                eprintln!("Failed to send audio controller command: {:?}", e);
            }
        });
    }
}