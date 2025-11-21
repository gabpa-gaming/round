use std::{clone, path::{Path, PathBuf}};

use audiotags::Tag;
use dioxus::{core::IntoAttributeValue, html::u::is, prelude::*};

use crate::{app_context::{DatabaseContext, PlayerContext}, db::{self, Db, SongView}, player, player_playing_state::PlayerPlayingState};

pub const RECOGNIZED_FILE_EXTENSIONS: [&str; 5] = ["mp3", "flac", "wav", "aac", "ogg"];

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SongFileData {
    Song{ song_view: SongView },
    NotSong { },
}

impl PartialOrd for SongFileData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (SongFileData::Song { song_view: a, .. }, SongFileData::Song { song_view: b, .. }) => a.partial_cmp(b),
            (SongFileData::NotSong {}, SongFileData::NotSong {}) => Some(std::cmp::Ordering::Equal),
            (SongFileData::Song { .. }, SongFileData::NotSong {}) => Some(std::cmp::Ordering::Less),
            (SongFileData::NotSong {}, SongFileData::Song { .. }) => Some(std::cmp::Ordering::Greater),
        }
    }
}

impl Ord for SongFileData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileEntry {
    pub path: PathBuf,
    pub is_folder: bool,
    pub song_data: SongFileData
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanResult {
    pub entries: Vec<FileEntry>,
    pub only_contains_audio: bool,
}

pub fn scan_dir(path: &str, db: &Db) -> ScanResult {
    let mut entries = Vec::new();

    let current_dir = Path::new(path);

    let mut only_contains_audio = true;

    if let Ok(dir_entries) = current_dir.read_dir() {
        for entry in dir_entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                let is_folder = path.is_dir();
                if is_folder {
                    only_contains_audio = false;
                }
                if !is_folder &&
                    !&entry.file_type()
                    .is_ok_and(|ft| {
                        RECOGNIZED_FILE_EXTENSIONS.contains(
                            &entry.path().extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("").to_lowercase().as_str()
                        )
                    }) {
                    continue;
                }
                entries.push(FileEntry {
                    path: path.to_path_buf(),
                    is_folder,
                    song_data: if is_folder {
                        SongFileData::NotSong {}
                    } else {
                        if let Some(song_data) = get_song_file_data(&path, db) {
                            song_data
                        } else {
                            SongFileData::NotSong {}
                        }
                    },
                });

            }
        }
    }
    if entries.is_empty() {
        only_contains_audio = false;
    }
    entries.sort_by(|a, b| {
        if a.is_folder && !b.is_folder {
            std::cmp::Ordering::Less
        } else if !a.is_folder && b.is_folder {
            std::cmp::Ordering::Greater
        } else if a.song_data != b.song_data {
            a.song_data.cmp(&b.song_data)
        } else {
            a.path.file_name().cmp(&b.path.file_name())
        }
    });
    ScanResult { entries, only_contains_audio }
}

pub fn get_song_file_data(file_path: &Path, db: &Db) -> Option<SongFileData> {
    let song = db.get_song_view_by_path(file_path.to_string_lossy().as_ref()).ok();
    if let Some(song) = song {
        Some(SongFileData::Song { song_view: song})
    } else {
        None
    }
}

#[component]
pub fn file_browser(starting_path: String) -> Element {
    let mut current_path =  use_signal(|| starting_path.clone()); 

    let db = use_context::<DatabaseContext>();

    let player_context = use_context::<PlayerContext>();

    let items = {
        let db = db.clone();
        use_memo(move  || scan_dir(current_path().as_str(), db.get()))
    };

    let playing_state = use_context::<PlayerContext>().playing_state.clone();

    let open_folder = move |file_path: String| {
        *current_path.write() = file_path;
    };


    
    rsx! {
        div {
            class: "file-explorer",
            div {
                class: "content-section",
                h2 { "Files" }
                div {
                    class: if !items().only_contains_audio { "item-grid" } else { "item-list" },
                    self::file_shortcut {
                        name: "..",
                        icon: rsx! { "📁" },
                        path: Path::new(&current_path()).parent().unwrap_or_else(|| Path::new("")).to_string_lossy().to_string(),
                        on_click: open_folder.clone(),
                        on_context_menu: move |evt: Event<MouseData>| {
                            evt.prevent_default();
                        }
                    }
                    for item in items().entries {
                        {
                            let item = item.clone();
                            let mut current_path = current_path.clone();
                            let db = db.clone();
                            let mut playing_state = playing_state.clone();
                            let mut player_context = player_context.clone();
                            rsx! {
                                self::file { file: item.clone(),
                                     on_click: move |path| {
                                        if item.is_folder {
                                            current_path.set(path);
                                        } else {
                                            if let Ok(song) = db.get().get_song_view_by_path(&path) {
                                                player_context.queue.write().play_song_instant(&song);
                                            }
                                        }
                                     }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn file(file: FileEntry, on_click: EventHandler<String>) -> Element {
    let mut show_context_menu = use_signal(|| false);
    let mut context_menu_x = use_signal(|| 0.0);
    let mut context_menu_y = use_signal(|| 0.0);
    
    let db = use_context::<DatabaseContext>();
    let mut player_context = use_context::<PlayerContext>();
    
    let icon_element = if !file.is_folder {
         { match &file.song_data {
            SongFileData::Song { song_view } => {
                let art = song_view.album_art_path.clone();
                if let Some(art) = art {
                    rsx! { img { src: art, alt: "Album Art", class: "art" } }
                } else {
                    rsx!{"🎵"}
                }
            }
            SongFileData::NotSong {  } => rsx!{"🎵"},
        } }
    } else {
        rsx! { "📁" }
    };
    
    let file_path = file.path.to_string_lossy().to_string();
    
    rsx! {
        file_shortcut { 
            name: match file.song_data {
                SongFileData::Song { song_view } => song_view.title,
                SongFileData::NotSong {} => file.path.file_name().unwrap_or_default().to_string_lossy().to_string()
            },
            icon: icon_element,
            path: file_path.clone(),
            on_click: on_click.clone(),
            on_context_menu: move |evt: Event<MouseData>| {
                evt.prevent_default();
                context_menu_x.set(evt.client_coordinates().x);
                context_menu_y.set(evt.client_coordinates().y);
                show_context_menu.set(true);
            }
        }
        
        if show_context_menu() {
            
        }
    }
}

#[component]
pub fn file_shortcut(name: String, icon: Element, path: String, on_click: EventHandler<String>, on_context_menu: EventHandler<Event<MouseData>>) -> Element {
    
    rsx! {
        if path.is_empty() {
            {}
        } else {
            button { 
                onclick: move |_| {
                    on_click.call(path.clone());
                },
                oncontextmenu: move |evt| {
                    on_context_menu.call(evt);
                },
                class: "file-item",
                div { class: "item-icon", {icon}  }
                div { class: "item-name", "{name}" }
            }
        }
    }
}