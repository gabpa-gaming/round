use std::sync::Arc;

use rand::seq::index;

use crate::db::{Db, SongView};

#[derive(Clone)]
pub struct Playlist {
    id: i32,
    name: String,
    current_song_index: usize,
    pub db: Arc<Db>,
}

impl PartialEq for Playlist {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Playlist {
    fn new(id: i32, name: String, db: Arc<Db>) -> Result<Self, rusqlite::Error> {
        let db = db.clone();
        let id = db.create_playlist(&name);

        Self::get_playlist_handle(id?, db)
    }

    pub fn get_playlist_handle(id: i32, db: Arc<Db>) -> Result<Playlist, rusqlite::Error> {
        let db = db.clone();
        let data = db.get_playlist_data(id)?;

        Ok(Playlist{id: data.0, name: data.1, current_song_index: 0, db: db.clone()})
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn songs(&self) -> Vec<SongView> {
        self.db.get_songs_in_playlist(self.id).ok().unwrap_or(Vec::new())
    }

    pub fn current_index(&self) -> usize {
        self.current_song_index
    }

    pub fn set_current(&mut self, index: usize) {
        self.current_song_index = index;
    }

    pub fn song_count(&self) -> usize {
        self.db.get_playlist_song_count(self.id).unwrap_or(0)
    }

    pub fn next_song(&mut self) -> Option<SongView> {
        if let Some(song) = self.db.get_nth_playlist_song(self.id, self.current_song_index + 1).ok()? {
            self.current_song_index += 1;
            Some(song)
        } else {
            let song = self.db.get_nth_playlist_song(self.id, 0).ok()?;
            self.current_song_index = 0;
            song
        }
    }
    
    pub fn prev_song(&mut self) -> Option<SongView> {
        if let Some(song) = self.db.get_nth_playlist_song(self.id, self.current_song_index - 1).ok()? {
            self.current_song_index -= 1;
            Some(song)
        } else {
            let len = self.song_count();
            let song = self.db.get_nth_playlist_song(self.id, len - 1).ok()?;
            self.current_song_index = len - 1;
            song
        }
    }
}