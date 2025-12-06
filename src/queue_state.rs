use std::collections::VecDeque;

use dioxus::prelude::*;
use rand::{random, rng, seq::SliceRandom};
use tokio::sync::mpsc::Sender;

use anyhow::{anyhow, Result};

use crate::{app_context::PlaybackMode, audio_controller_command::AudioControllerCommand, db::{Db, SongView}, file_browser::{ScanResult, SongFileData, scan_dir}, player_playing_state::PlayerPlayingState, playlist::{self}};

const HISTORY_MAX_SIZE: usize = 9999;

pub enum QueueFallbackMode {
    Playlist {playlist: playlist::Playlist},
    Folder {path: String, current_item: usize, entries: ScanResult},
    None
}

impl QueueFallbackMode {
    pub fn next(&mut self) -> Option<SongView> {
        match self {
            QueueFallbackMode::Playlist { playlist } => {
                playlist.next_song()
            },
            QueueFallbackMode::Folder { path: _, current_item, entries } => {
                if *current_item + 1 < entries.entries.len() {
                    *current_item += 1;
                    let file_data = &entries.entries[*current_item].song_data;
                    if let SongFileData::Song { song_view } = file_data {
                        Some(song_view.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            QueueFallbackMode::None => None,
        }
    }

    pub fn next_shuffle(&mut self, history: &VecDeque<SongView>) -> Option<SongView> {
        let songs = match self {
            QueueFallbackMode::Playlist { playlist } => {
                playlist.songs()
            },
            QueueFallbackMode::Folder { path: _, current_item: _, entries } => {
                entries.entries.iter().filter_map(|entry| {
                    if let SongFileData::Song { song_view } = &entry.song_data {
                        Some(song_view.clone())
                    } else {
                        None
                    }
                }).collect()
            },
            QueueFallbackMode::None => Vec::new(),
        };
        if songs.is_empty() {
            return None;
        } 
        let mut contender = None;
        let mut max_appeared: usize = 0;
        let max_checking = songs.len() * 90 / 100;

        let mut songs = songs.clone();
        songs.shuffle(&mut rng()); 
        for song in songs.iter() {
            let mut last_appeared: usize = 0;
            if !history.iter().any(|s| s.id == song.id) {
                contender = Some(song.clone());
                break;
            }
            for played_song in history.iter().rev() {
                if played_song.id == song.id {
                    last_appeared += 1;
                }
                if max_appeared < last_appeared {
                    max_appeared = last_appeared;
                    contender = Some(song.clone());
                }
            }
            if last_appeared >= max_checking || 
            (history.len() < max_checking) {
                break;
            }
        }
        contender
    }
}

pub struct QueueState {
    pub play_next_queue: Signal<VecDeque<SongView>>,
    pub current_fallback_queue: Signal<QueueFallbackMode>,
    pub last_played: Signal<VecDeque<SongView>>,
    pub playing_state: Signal<PlayerPlayingState>,
    pub mode: Signal<PlaybackMode>,
    command_sender: Sender<AudioControllerCommand> 
}

impl QueueState {
    pub fn new(playing_state: Signal<PlayerPlayingState>, command_sender: Sender<AudioControllerCommand>, mode: Signal<PlaybackMode>) -> Self {
        //todo need to load from db here
        QueueState {
            play_next_queue: Signal::new(VecDeque::new()),
            current_fallback_queue: Signal::new(QueueFallbackMode::None),
            last_played: Signal::new(VecDeque::new()),
            playing_state,
            mode,
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
        loop {
            let next_opt = if *self.mode.clone().read() == PlaybackMode::Shuffle {
            if !self.play_next_queue.read().is_empty() {
                let rand = random::<u32>();
                let index = rand as usize % self.play_next_queue.read().len();
                Some(self.play_next_queue.write().remove(index).unwrap())
            } else {
                self.current_fallback_queue.write().next_shuffle(&self.last_played.read())
            }
            } else if *self.mode.clone().read() == PlaybackMode::Loop{
                if let Some(current_song) = self.playing_state.read().current_song() {
                    Some(current_song)
                } else if let Some(next_song) = self.play_next_queue.write().pop_front() {
                    Some(next_song)
                } else {
                    self.current_fallback_queue.write().next()
                }
            } else if let Some(next_song) = self.play_next_queue.write().pop_front() {
                Some(next_song)
            } else {
                self.current_fallback_queue.write().next()
            };
            if let Some(song) = self.playing_state.read().current_song() {
                self.last_played.write().push_back(song.clone());
            }
            if next_opt.is_none() {
                eprintln!("Queue empty");
                self.send_cmd(AudioControllerCommand::Stop);
                *self.playing_state.write() = PlayerPlayingState::NoSongSelected;
                return;
            }
            if self.play_song_instant(&next_opt.unwrap()).is_ok() { return }
        } 
    }

    pub fn previous_song(&mut self) {
        let prev_opt = self.last_played.write().pop_back();
        if let Some(song) = self.playing_state.read().current_song() {
            self.play_next_queue.write().push_front(song.clone());
        }
        if prev_opt.is_none() {
            eprintln!("History empty");
            self.send_cmd(AudioControllerCommand::Stop);
            *self.playing_state.write() = PlayerPlayingState::NoSongSelected;
            return;
        }
        if self.play_song_instant(&prev_opt.unwrap()).is_ok() { return }
    }

    pub fn play_song_instant(&mut self, song: &SongView) -> Result<()>{
        let song_file = std::fs::File::open(&song.path);
        if song_file.is_err() {
            eprintln!("Failed to open song file: {}", song.path);
            return Err(anyhow!("Failed to open song file: {}", song.path));
        }
        self.send_cmd(AudioControllerCommand::Load(song.path.clone()));
        self.send_cmd(AudioControllerCommand::Play);    
        *self.playing_state.write() = PlayerPlayingState::Playing { song: song.clone(), progress_ms: 0};    
        Ok(())
    }

    pub fn play_song_next(&mut self, song: &SongView) {
        self.play_next_queue.write().push_back(song.clone());
        if !self.playing_state.read().is_playing() {
            self.next_song();
        }
    }

    pub fn add_song_to_queue(&mut self, song: &SongView) {
        self.play_next_queue.write().push_back(song.clone());
        if !self.playing_state.read().is_playing() {
            self.next_song();
        }
    }

    pub fn clear_queue(&mut self) {
        self.play_next_queue.write().clear();
        *self.current_fallback_queue.write() = QueueFallbackMode::None;
    }

    pub fn play_folder_next(&mut self, path: &str, db: &Db) -> bool {
        let files = scan_dir(path, db);
        let mut added_any = false;
        for file in files.entries.into_iter().rev() {
            if let SongFileData::Song { song_view } = file.song_data {
                self.play_next_queue.write().push_front(song_view);
                added_any = true;
            }
        }
        added_any
    }


    pub fn play_folder_now(&mut self, path: &str, db: &Db) -> bool {
        let added_any = self.play_folder_next(path, db);
        if added_any {
            self.next_song();
        }
        added_any
    }

    pub fn add_entire_path_to_queue(&mut self, path: &str, db: &Db) -> bool {
        let files = scan_dir(path, db);

        let mut added_any = false;

        for file in files.entries {
            if let SongFileData::Song { song_view } = file.song_data {
                self.play_next_queue.write().push_back(song_view);
                added_any = true;
            }
        }
        if added_any && self.playing_state.read().current_song().is_none() {
            self.next_song();
        }
        added_any
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