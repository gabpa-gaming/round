
use crate::db::SongView;

#[derive(Clone, Debug, PartialEq)]
pub enum PlayerPlayingState {
    NoSongSelected,
    Playing { song: SongView, progress_ms: u64 },
    Paused { song: SongView, progress_ms: u64 },
    SongFinished{ song: SongView },
}

impl PlayerPlayingState {
    pub fn progress(&self) -> u64 {
        match self {
            PlayerPlayingState::NoSongSelected => 0,
            PlayerPlayingState::Playing { progress_ms, .. } => *progress_ms,
            PlayerPlayingState::Paused {  progress_ms, .. } => *progress_ms,
            PlayerPlayingState::SongFinished { .. } => 0,
        }
    }

    pub fn progress_mut(&mut self) -> Option<&mut u64> {
        match self {
            PlayerPlayingState::NoSongSelected => None,
            PlayerPlayingState::Playing { progress_ms, .. } => Some(progress_ms),
            PlayerPlayingState::Paused {  progress_ms, .. } => Some(progress_ms),
            PlayerPlayingState::SongFinished { .. } => None,
         }
    }

    pub fn current_song(&self) -> Option<SongView> {
        match self {
            PlayerPlayingState::NoSongSelected => None,
            PlayerPlayingState::Playing { song, .. } => Some(song.clone()),
            PlayerPlayingState::Paused { song, .. } => Some(song.clone()),
            PlayerPlayingState::SongFinished { song } => Some(song.clone()),
        }
    }

    pub fn pause(&mut self) {
        if let PlayerPlayingState::Playing { song, progress_ms } = self {
            *self = PlayerPlayingState::Paused { song: song.clone(), progress_ms: *progress_ms };
        }
    }

    pub fn play(&mut self) {
        if let PlayerPlayingState::Paused { song, progress_ms } = self {
            *self = PlayerPlayingState::Playing { song: song.clone(), progress_ms: *progress_ms };
        }
    }

    pub fn is_playing(&self) -> bool {
        matches!(self, PlayerPlayingState::Playing { .. })
    }

    pub fn song_path(&self) -> Option<String> {
        match self {
            PlayerPlayingState::NoSongSelected => None,
            PlayerPlayingState::Playing { song, .. } => Some(song.path.clone()),
            PlayerPlayingState::Paused { song, .. } => Some(song.path.clone()),
            PlayerPlayingState::SongFinished { song } => Some(song.path.clone()),
        }
    }
}