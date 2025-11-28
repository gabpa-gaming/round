use std::path::{Path, PathBuf};

use dioxus::prelude::*;

use crate::{
    app_context::{DatabaseContext, PlayerContext},
    context_menu::{ContextMenuItem, context_menu},
    create_playlist_dialog::create_playlist_dialog,
    db::{Db, SongView},
    playlist_browser::playlist_browser, queue_state::QueueFallbackMode,
};

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

impl FileEntry {
    pub fn from_song_view(song: &SongView) -> Self {
        FileEntry {
            path: PathBuf::from(&song.path),
            is_folder: false,
            song_data: SongFileData::Song { song_view: song.clone() },
        }
    }
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
                    
                    for (index, item) in items().entries.iter().enumerate() {
                        {
                            let scan_result = items();
                            let item = item.clone();
                            let index = index;
                            let mut current_path = current_path.clone();
                            let db = db.clone();
                            let mut player_context = player_context.clone();
                            rsx! {
                                self::song_file { file: item.clone(),
                                     on_click: move |path| {
                                        if item.is_folder {
                                            current_path.set(path);
                                        } else {
                                            if let Ok(song) = db.get().get_song_view_by_path(&path) {
                                                player_context.queue.write().play_song_instant(&song);
                                                player_context.queue.write().current_fallback_queue.set(QueueFallbackMode::Folder {
                                                    path: current_path().clone(),
                                                    current_item: index,
                                                    entries: scan_result.clone(),
                                                });
                                            }
                                        }
                                     }
                                }
                            }
                        }
                    }
                }
                playlist_browser {}
            }
            
        }
    }
}

#[component]
pub fn song_file(file: FileEntry, on_click: EventHandler<String>) -> Element {
    let mut show_context_menu = use_signal(|| false);
    let mut context_menu_pos = use_signal(|| (0.0, 0.0));
    let mut show_add_to_playlist_menu = use_signal(|| false);
    let mut add_playlist_menu_pos = use_signal(|| (0.0, 0.0));
    
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
    
    let db_clone = db.clone();
    let fp_clone = file_path.clone();
    let mut play_instant = move || {
        if let Ok(song) = db_clone.get().get_song_view_by_path(&fp_clone) {
            player_context.queue.write().play_song_instant(&song);
        }
    };
    let db_clone = db.clone();
    let fp_clone = file_path.clone();
    let mut play_next = move || {
        if let Ok(song) = db_clone.get().get_song_view_by_path(&fp_clone) {
            player_context.queue.write().play_song_next(&song);
        }
    };

    let db_clone = db.clone();
    let fp_clone = file_path.clone();
    let mut add_song_to_queue = move || {
        if let Ok(song) = db_clone.get().get_song_view_by_path(&fp_clone) {
            player_context.queue.write().add_song_to_queue(&song);
        }
    };

    let db_clone = db.clone();
    let fp_clone = file_path.clone();
    let mut add_folder_to_queue = move || {
        player_context.queue.write().add_entire_path_to_queue(&fp_clone, &db_clone.get());
    };

    let db_clone = db.clone();
    let fp_clone = file_path.clone();
    let mut play_folder_next = move || {
        player_context.queue.write().play_folder_next(&fp_clone, &db_clone.get());
    };

    let db_clone = db.clone();
    let fp_clone = file_path.clone();
    let mut play_folder_now = move || {
        player_context.queue.write().play_folder_now(&fp_clone, &db_clone.get());
    };

    let song_id = match &file.song_data {
        SongFileData::Song { song_view } => Some(song_view.id),
        SongFileData::NotSong {} => None,
    };

    let context_menu_items = if !file.is_folder { 
        let mut items = vec![
            ContextMenuItem {
                title: "Play".to_string(),
                action: EventHandler::new(move |_| {
                    play_instant();
                }),
            },
            ContextMenuItem {
                title: "Play Next".to_string(),
                action: EventHandler::new(move |_| {
                    play_next();
                }),
            },
            ContextMenuItem {
                title: "Add to queue".to_string(),
                action: EventHandler::new(move |_| {
                    add_song_to_queue();
                }),
            },
        ];
        
        if song_id.is_some() {
            items.push(ContextMenuItem {
                title: "Add to playlist...".to_string(),
                action: EventHandler::new(move |_| {
                    add_playlist_menu_pos.set(context_menu_pos());
                    show_add_to_playlist_menu.set(true);
                }),
            });
        }
        
        items
    } else {
         vec![
            ContextMenuItem {
                title: "Play all now".to_string(),
                action: EventHandler::new(move |_| {
                    play_folder_now();
                }),
            },
            ContextMenuItem {
                title: "Play all next".to_string(),
                action: EventHandler::new(move |_| {
                    play_folder_next();
                }),
            },
            ContextMenuItem {
                title: "Add all to queue".to_string(),
                action: EventHandler::new(move |_| {
                    add_folder_to_queue();
                }),
            },
        ]};
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
                context_menu_pos.set((evt.client_coordinates().x, evt.client_coordinates().y));
                show_context_menu.set(true);
            }
        }
        context_menu { pos: context_menu_pos.clone(),
            show_context_menu: show_context_menu.clone(),
            items: context_menu_items.clone()}
        if let Some(song_id) = song_id {
            add_to_playlist_menu {
                song_id,
                show: show_add_to_playlist_menu,
                pos: add_playlist_menu_pos
            }
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

#[component]
pub fn add_to_playlist_menu(song_id: i32, show: Signal<bool>, pos: Signal<(f64, f64)>) -> Element {
    let db = use_context::<DatabaseContext>();
    let mut playlist_update = use_context::<PlayerContext>().playlist_update_counter;
    let mut show_create_dialog = use_signal(|| false);

    let db_clone = db.clone();
    let playlists = use_memo(move || {
        let _ = playlist_update();
        db_clone.get().get_all_playlists().unwrap_or_default()
    });

    if !show() {
        return rsx! {};
    }

    let (x, y) = *pos.read();

    rsx! {
        div {
            class: "context-menu-overlay",
            onclick: move |_| {
                show.set(false);
            },
            oncontextmenu: move |evt: Event<MouseData>| {
                evt.prevent_default();
                show.set(false);
            },

            div {
                class: "context-menu",
                style: "left: {x}px; top: {y}px; position: fixed;",
                onclick: move |evt: Event<MouseData>| evt.stop_propagation(),

                button {
                    class: "context-menu-item",
                    onclick: move |_| {
                        show_create_dialog.set(true);
                    },
                    "Create New Playlist..."
                }

                for (playlist_id, playlist_name) in playlists() {
                    {
                        let db_clone = db.clone();
                        rsx! {
                            button {
                                class: "context-menu-item",
                                onclick: move |_| {
                                    let _ = db_clone.get().add_song_to_playlist(playlist_id, song_id);
                                    playlist_update.set(playlist_update() + 1);
                                    show.set(false);
                                },
                                "{playlist_name}"
                            }
                        }
                    }
                }
            }
        }

        create_playlist_dialog {
            show: show_create_dialog,
            on_created: move |playlist_id| {
                let _ = db.get().add_song_to_playlist(playlist_id, song_id);
                show.set(false);
            }
        }
    }
}
