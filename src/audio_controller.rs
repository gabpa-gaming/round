
use std::{fs::File, io::BufReader};

use dioxus::signals::{ReadableExt, Signal, WritableExt};
use rodio::{Decoder, OutputStream, Sink, StreamError};
use tokio::sync::mpsc::{Receiver};
use uuid::Error;

use crate::{audio_controller_command::AudioControllerCommand, player_playing_state::PlayerPlayingState};


pub struct AudioController {
    receiver: Receiver<AudioControllerCommand>,
    playing_state: Signal<PlayerPlayingState>,
}

impl AudioController {
    pub fn new(receiver: Receiver<AudioControllerCommand>, playing_state: Signal<PlayerPlayingState>) -> Self {
        AudioController {
            receiver,
            playing_state,
        }
    }

    pub async fn run(&mut self){
        loop {
            if let Err(e) = self.update_loop().await {
                eprintln!("AudioController encountered an error: {}", e);
            }
        }
    }

    pub async fn update_loop(&mut self) -> Result<(), StreamError> {
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
        let mut sink = rodio::Sink::connect_new(&stream_handle.mixer());
        loop {
            tokio::select! {
                Some(cmd) = self.receiver.recv() => self.handle_command(cmd, &mut sink).await,
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(20)) => {
                    if !sink.is_paused() {
                        self.playing_state.with_mut(|state| {
                            if let Some(progress) = state.progress_mut(){
                                *progress = sink.get_pos().as_millis() as u64;
                                if let PlayerPlayingState::Playing { song, .. } = state.clone() {
                                    if sink.empty() {
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

    pub async fn handle_command(&mut self, cmd: AudioControllerCommand, sink: &mut Sink) {
        match cmd {
            AudioControllerCommand::Play => {
                sink.play();
            },
            AudioControllerCommand::Pause => {
                sink.pause();
            },
            AudioControllerCommand::Load(path) => {
                if let Ok(file) = File::open(&path) {
                    if let Ok(source) = Decoder::new(BufReader::new(file)) {
                        sink.clear();
                        sink.append(source);
                        sink.play();
                    }
                }
            },
            AudioControllerCommand::SetVolume(volume) => {
                sink.set_volume(volume);
            },
            AudioControllerCommand::SetProgress(progress_ms) => {
                if sink.try_seek(std::time::Duration::from_millis(progress_ms)).is_err(){
                    eprintln!("Failed to seek to {} ms", progress_ms);
                    println!("Trying to fallback by reloading the track");
                    if let Some(song) = self.playing_state.clone().read().current_song() {
                        if let Ok(file) = File::open(&song.path) {
                            if let Ok(source) = Decoder::new(BufReader::new(file)) {
                                sink.clear();
                                sink.append(source);
                                if sink.try_seek(std::time::Duration::from_millis(progress_ms)).is_err(){
                                    eprintln!("Fallback seek also failed");
                                } else { 
                                    println!("Fallback seek succeeded");
                                }
                                sink.play();
                            }
                        }
                    }
                };
            },
            AudioControllerCommand::Stop => {
                sink.stop();
            },
            _ => {
                eprintln!("Unhandled audio controller command");
            }
        }
        
    }
}