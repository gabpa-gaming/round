
use std::{fs::File, io::BufReader};

use dioxus::signals::{ReadableExt, Signal, WritableExt};
use rodio::{Decoder, OutputStream, Sink};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{audio_controller_command::AudioControllerCommand, player_playing_state::PlayerPlayingState};


pub struct AudioController {
    stream_handle: OutputStream,
    sink: Sink,
    receiver: Receiver<AudioControllerCommand>,
    playing_state: Signal<PlayerPlayingState>,
}

impl AudioController {
    pub fn new(receiver: Receiver<AudioControllerCommand>, playing_state: Signal<PlayerPlayingState>) -> Self {
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
            .expect("open default audio stream");
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());

        AudioController {
            stream_handle,
            sink,
            receiver,
            playing_state,
        }
    }

    pub async fn run(&mut self){
        loop {
            tokio::select! {
                Some(cmd) = self.receiver.recv() => self.handle_command(cmd).await,
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(20)) => {
                    if !self.sink.is_paused() {
                        self.playing_state.with_mut(|state| {
                            if let Some(progress) = state.progress_mut(){
                                *progress = self.sink.get_pos().as_millis() as u64;
                                if let PlayerPlayingState::Playing { song, .. } = state.clone() {
                                    if self.sink.empty() {
                                        *state = PlayerPlayingState::SongFinished { song };
                                    }
                                }
                            }
                        });
                    }
                }
            }
        };
        
    }

    pub async fn handle_command(&mut self, cmd: AudioControllerCommand) {
        match cmd {
            AudioControllerCommand::Play => {
                self.sink.play();
            },
            AudioControllerCommand::Pause => {
                self.sink.pause();
            },
            AudioControllerCommand::Load(path) => {
                if let Ok(file) = File::open(&path) {
                    if let Ok(source) = Decoder::new(BufReader::new(file)) {
                        self.sink.clear();
                        self.sink.append(source);
                        self.sink.play();
                    }
                }
            },
            AudioControllerCommand::SetVolume(volume) => {
                self.sink.set_volume(volume);
            },
            AudioControllerCommand::SetProgress(progress_ms) => {
                if self.sink.try_seek(std::time::Duration::from_millis(progress_ms)).is_err(){
                    eprintln!("Failed to seek to {} ms", progress_ms);
                    println!("Trying to fallback by reloading the track");
                    if let Some(song) = self.playing_state.clone().read().current_song() {
                        if let Ok(file) = File::open(&song.path) {
                            if let Ok(source) = Decoder::new(BufReader::new(file)) {
                                self.sink.clear();
                                self.sink.append(source);
                                if self.sink.try_seek(std::time::Duration::from_millis(progress_ms)).is_err(){
                                    eprintln!("Fallback seek also failed");
                                } else { 
                                    println!("Fallback seek succeeded");
                                }
                                self.sink.play();
                            }
                        }
                    }
                };
            },
            AudioControllerCommand::Stop => {
                self.sink.stop();
            },
            _ => {
                eprintln!("Unhandled audio controller command");
            }
        }
        
    }
}