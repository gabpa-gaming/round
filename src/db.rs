use std::{env, f32::consts::E, path::Path, sync::Arc};

use audiotags::{AudioTag, MimeType, Picture, Tag};
use dioxus::{core::const_format::SplicedStr, html::{geometry::euclid::num::Round, path, tr}};
use rodio::{Source, decoder::DecoderError};
use rusqlite::{Connection, OptionalExtension, Result, params};

use crate::errors::SongAddError;

const DB_STATE_VERSION: i32 = 6; //Change this when the DB schema changes

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SongView {
    pub id: i32,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_art_path: Option<String>,
    pub track_number: Option<u16>,
    pub duration_seconds: u64,
    pub play_count: i32,
}

impl PartialOrd for SongView {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.artist == other.artist && self.album == other.album {
            self.track_number.partial_cmp(&other.track_number)
        } else {
            Some(self.artist.cmp(&other.artist)
                .then(self.album.cmp(&other.album))
                .then(self.title.cmp(&other.title)))
        }
    }
}

pub struct SongDbEntry {
    pub id: i32,
    pub path: String,
    pub title: String,
    pub album_id: i32,
    pub track_number: Option<u16>,
    pub duration_seconds: u64,
    pub play_count: i32,
}

impl SongDbEntry {
    pub fn to_song_view(&self, db: &Db) -> Result<SongView> {
        let (album_name, album_art_path) = db.get_album(self.album_id as i64)?
            .unwrap_or(("Unknown Album".to_string(), None));
        
        let artist_name = db.conn.query_row(
            "SELECT ar.name
             FROM artists ar
             JOIN albums al ON ar.id = al.artist_id
             WHERE al.id = ?1",
            params![self.album_id],
            |row| row.get(0),
        ).optional()?.unwrap_or("Unknown Artist".to_string());

        Ok(SongView {
            id: self.id,
            path: self.path.clone(),
            title: self.title.clone(),
            artist: artist_name,
            album: album_name,
            album_art_path,
            track_number: self.track_number,
            duration_seconds: self.duration_seconds,
            play_count: self.play_count,
        })
    }
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn new() -> Db {
        let db = Db { conn: Connection::open("music_library.db").unwrap()};

        db.conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();

        db.handle_db_version_change();

        db.conn.execute(
            "CREATE TABLE IF NOT EXISTS artists (
                id      INTEGER PRIMARY KEY,
                name    TEXT NOT NULL UNIQUE
            )",
            [],
        ).unwrap();

        db.conn.execute(
            "CREATE TABLE IF NOT EXISTS albums (
                id               INTEGER PRIMARY KEY,
                name             TEXT NOT NULL,
                artist_id        INTEGER NOT NULL,
                cover_art_path   TEXT,
                FOREIGN KEY(artist_id) REFERENCES artists(id),
                UNIQUE(name, artist_id)
            )",
            [],
        ).unwrap();

        db.conn.execute(
            "CREATE TABLE IF NOT EXISTS songs (
                id               INTEGER PRIMARY KEY,
                path             TEXT NOT NULL UNIQUE,
                title            TEXT NOT NULL,
                album_id         INTEGER NOT NULL,
                track_number     INTEGER,
                duration_seconds INTEGER NOT NULL DEFAULT 0,
                play_count       INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY(album_id) REFERENCES albums(id)
            )",
            [],
        ).unwrap();

        db
    }

    fn get_or_insert_artist_id(&self, artist_name: &str) -> Result<i32> {
        self.conn.execute(
            "INSERT OR IGNORE INTO artists (name) VALUES (?1)",
            params![artist_name],
        )?;
        
        self.conn.query_row(
            "SELECT id FROM artists WHERE name = ?1",
            params![artist_name],
            |row| row.get(0),
        )
    }

    fn get_or_insert_album_id(
        &self,
        album_name: &str,
        artist_id: i32,
        cover_art_picture: Option<Picture>,
        fallback_song_path: &str
    ) -> Result<i32> {
        let cover_art_path = cover_art_picture.map(|pic| {
            let file_name = format!("cover_{}_{}.{}", artist_id, album_name, Self::get_image_extension(pic.mime_type));
            println!("Writing cover art to file: {}", file_name);
            if std::fs::write(&file_name, pic.data).is_ok() { // Likely freezes the UI, should be done async
                println!("Wrote cover art to file: {}", file_name);
                Some(env::current_dir().unwrap().join(file_name).to_str().unwrap().to_string())
            } else {
                eprintln!("Failed to write cover art for album: {} by artist ID: {}", album_name, artist_id);
                Self::get_album_art_from_folder(fallback_song_path)
            }
        });
        self.conn.execute(
            "INSERT OR IGNORE INTO albums (name, artist_id, cover_art_path) VALUES (?1, ?2, ?3)",
            params![album_name, artist_id, cover_art_path],
        )?;
        println!("Added album: {} by artist ID: {}", album_name, artist_id);
        self.conn.query_row(
            "SELECT id FROM albums WHERE name = ?1 AND artist_id = ?2",
            params![album_name, artist_id],
            |row| row.get(0),
        )
    }

    fn get_album_art_from_folder(song_path: &str) -> Option<String> {
        let path = Path::new(song_path);
        let folder = path.parent()?;
        
        let cover_names = [
            "cover.jpg", "cover.png", "cover.jpeg",
            "folder.jpg", "folder.png", "folder.jpeg",
            "artwork.jpg", "artwork.png",
            "front.jpg", "front.png",
            "album.jpg", "album.png",
            "Cover.jpg", "Folder.jpg", 
        ];
        
        for name in cover_names {
            let cover_path = folder.join(name);
            if cover_path.exists() {
                println!("Found album art in folder: {:?}", cover_path);
                return Some(cover_path.to_string_lossy().to_string());
            }
        }
        
        None
    }

    fn get_or_insert_album_id_w_path(
        &self,
        album_name: &str,
        artist_id: i32,
        cover_art_path: Option<&str>,
    ) -> Result<i32> {
        self.conn.execute(
            "INSERT OR IGNORE INTO albums (name, artist_id, cover_art_path) VALUES (?1, ?2, ?3)",
            params![album_name, artist_id, cover_art_path],
        )?;
        
        self.conn.query_row(
            "SELECT id FROM albums WHERE name = ?1 AND artist_id = ?2",
            params![album_name, artist_id],
            |row| row.get(0),
        )
    }

    pub fn get_album(&self, album_id: i64) -> Result<Option<(String, Option<String>)>> {
        self.conn.query_row(
            "SELECT name, cover_art_path FROM albums WHERE id = ?1",
            params![album_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                ))
            }
        ).optional()
    }

    pub fn get_image_extension(mime_type: MimeType) -> &'static str {
        match mime_type {
            MimeType::Jpeg => "jpg",
            MimeType::Png => "png",
            MimeType::Gif => "gif",
            MimeType::Bmp => "bmp",
            MimeType::Tiff => "tiff",
            _ => "jpg",
        }
    }

    pub fn add_song(&self, song: &SongDbEntry) {
        print!("Adding song: {} by album ID: {}", song.title, song.album_id);
        self.conn.execute(
        "INSERT OR IGNORE INTO songs
         (path, title, album_id, track_number, duration_seconds, play_count)
          VALUES (?1, ?2, ?3, ?4, ?5, ?6)", 
        params![
            song.path,
            song.title,
            song.album_id,
            song.track_number,
            song.duration_seconds, 
            song.play_count
        ],
        ).unwrap();
    }

    pub fn add_or_get_song_by_path(&self, path: &str) -> Result<SongDbEntry, SongAddError> {
        if let Ok(Some(song)) = self.conn.query_row(
            "SELECT s.id, s.path, s.title, s.album_id, s.track_number, s.duration_seconds, s.play_count
             FROM songs s
             WHERE s.path = ?1",
            params![path],
            |row| {
                println!("Found existing song in DB for path: {}", path);
                println!("Returning existing song.");
                println!("Song title: {}", row.get::<_, String>(2)?);
                println!("Song length: {}", row.get::<_, u64>(5)?);
                Ok(SongDbEntry {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    title: row.get(2)?,
                    album_id: row.get(3)?,
                    track_number: row.get(4)?,
                    duration_seconds: row.get(5)?,
                    play_count: row.get(6)?,
                })
            }
        ).optional() {
            Ok(song)
        } else {
            Ok(self.add_song_by_path(path)?)
        }
    }

    pub fn add_song_by_path(&self, path: &str) -> Result<SongDbEntry, SongAddError> {
        let tag = Tag::new().read_from_path(path);
    
        let artist = Self::try_get_album_artist_then_artist(tag.as_ref().ok());
        let artist_id = self.get_or_insert_artist_id(&artist).unwrap();

        let album_name = tag.as_ref().ok()
            .and_then(|t| t.album().map(|a| a.title.to_string()))
            .unwrap_or_else(|| "No Album".to_string());

        let album_id = self.get_or_insert_album_id(
            &album_name, 
            artist_id, 
            tag.as_ref().ok()
                .and_then(|t| t.album_cover()),
            path
        ).unwrap();

        let title = tag.as_ref().ok()
            .and_then(|t| t.title().map(|t| t.to_string()))
            .unwrap_or_else(|| std::path::Path::new(path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string());
        
        let track_number = tag.as_ref().ok()
            .and_then(|t| t.track().0.map(|n| n as u16));

        let duration_seconds = rodio::Decoder::new(std::fs::File::open(path).unwrap())
            .map(|d| d.total_duration().map(|d| d.as_secs()));

        if duration_seconds.is_err() {
            println!("Warning: Could not determine duration for file: {}", path);
            return Err(duration_seconds.err().unwrap().into());
        }

        let duration_seconds = duration_seconds.unwrap().unwrap();

        let song = SongDbEntry {
            id: 0,
            path: path.to_string(),
            title,
            album_id,
            track_number,
            duration_seconds,
            play_count: 0,
        };

        self.add_song(&song);
        Ok(song)
    }

    fn try_get_album_artist_then_artist(tag: Option<&Box<dyn AudioTag + Send + Sync>>) -> String {
        if let Some(tag) = tag {
            if let Some(artist) = tag.album_artist() {
                artist.to_string()
            } else if let Some(artist) = tag.artist() {
                artist.to_string()
            } else {
                "Unknown Artist".to_string()
            }
        } else {
            "Unknown Artist".to_string()
        }
    }

    pub fn get_song_view_by_path(&self, path: &str) -> Result<SongView> {
        if let Ok(Some(song_entry)) = self.conn.query_row(
                "SELECT
                        s.id,
                        s.path,
                        s.title,
                        ar.name,
                        al.album_name,
                        al.art_path,
                        s.track_number,
                        s.duration_seconds,
                        s.play_count
                    FROM songs s
                    JOIN albums al ON s.album_id = al.id
                    JOIN artists ar ON al.artist_id = ar.id
                    WHERE s.path = ?1",
            params![path],
            |row| {
                Ok(SongView {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    title: row.get(2)?,
                    artist: row.get(3)?,
                    album: row.get(4)?,
                    album_art_path: row.get(5)?,
                    track_number: row.get(6)?,
                    duration_seconds: row.get(7)?,
                    play_count: row.get(8)?,
                })
            }
        ).optional() { // fix this
            Ok(song_entry)
        } else {
            if let Some(song) = self.add_or_get_song_by_path(path).ok() {
                let song_view = song.to_song_view(self).unwrap();
                Ok(song_view)
            } else {
                Err(rusqlite::Error::QueryReturnedNoRows)
            }
        }
    }

    pub fn check_db_ver(&self) -> Result<bool> {
        let user_version: i32 = self.conn.query_row(
            "PRAGMA user_version;",
            [],
            |row| row.get(0),
        )?;

        Ok(user_version == DB_STATE_VERSION)
    }

    pub fn handle_db_version_change(&self) -> Result<()> {
        let user_version: i32 = self.conn.query_row(
            "PRAGMA user_version;",
            [],
            |row| row.get(0),
        )?;

        if user_version != DB_STATE_VERSION {
            println!("Database version mismatch. Expected: {}, Found: {}. Purging database.", DB_STATE_VERSION, user_version);
            self.purge_db()?;
            self.conn.pragma_update(None,"user_version", &DB_STATE_VERSION)?;
        }

        Ok(())
    }

    pub fn purge_db(&self) -> Result<()> {
        self.conn.execute("DROP TABLE songs", [])?;
        self.conn.execute("DROP TABLE albums", [])?;
        self.conn.execute("DROP TABLE artists", [])?;
        Ok(())
    }

}
