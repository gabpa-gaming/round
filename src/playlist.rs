use std::sync::Arc;

use crate::db::{Db, SongView};


pub struct Playlist {
    id: i32,
    name: String,
    current_song_index: usize,
    pub db: Arc<Db>,
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

    pub fn songs(&self) -> Vec<SongView> {
        self.db.get_songs_in_playlist(self.id).ok().unwrap_or(Vec::new())
    }

    pub fn next_song(&self) -> Option<SongView> {
        self.db.get_nth_playlist_song(self.id, self.current_song_index + 1).ok()?
    }
    
    pub fn prev_song(&self) -> Option<SongView> {
        self.db.get_nth_playlist_song(self.id, self.current_song_index + 1).ok()?
    }
}