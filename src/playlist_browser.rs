use dioxus::prelude::*;

use crate::{
    app_context::{DatabaseContext, PlayerContext}, context_menu::{ContextMenuItem, context_menu}, create_playlist_dialog::create_playlist_dialog, file_browser::{FileEntry, song_file}, playlist::Playlist, queue_state::QueueFallbackMode
};

#[derive(Clone, Debug, PartialEq)]
enum BrowserView {
    PlaylistList,
    PlaylistContent { playlist_id: i32, playlist_name: String },
}

#[component]
pub fn playlist_browser() -> Element {
    let mut current_view = use_signal(|| BrowserView::PlaylistList);

    rsx! {
            match current_view() {
                BrowserView::PlaylistList => rsx! {
                    playlist_list {
                        on_select: move |(id, name): (i32, String)| {
                            current_view.set(BrowserView::PlaylistContent {
                                playlist_id: id,
                                playlist_name: name,
                            });
                        }
                    }
                },
                BrowserView::PlaylistContent { playlist_id, playlist_name } => rsx! {
                    playlist_content {
                        playlist_id,
                        playlist_name: playlist_name.clone(),
                        on_back: move |_| {
                            current_view.set(BrowserView::PlaylistList);
                        }
                    }
                }
            }
        }
}

#[component]
fn playlist_list(on_select: EventHandler<(i32, String)>) -> Element {
    let db = use_context::<DatabaseContext>();
    let player_context = use_context::<PlayerContext>();
    let mut show_create_dialog = use_signal(|| false);
    let mut refresh_trigger = player_context.playlist_update_counter.clone();

    let db_clone = db.clone();
    let playlists = use_memo(move || {
        let _ = refresh_trigger();
        println!("Refreshing playlists: {}", refresh_trigger());
        db_clone.get().get_all_playlists().unwrap_or_default()
    });

    let background_menu_items = vec![ContextMenuItem {
        title: "Create New Playlist".to_string(),
        action: EventHandler::new(move |_| {
            show_create_dialog.set(true);
        }),
    }];

    rsx! {
            div { class: "content-section",
                style: "margin-bottom: 300px;",
                h2 { "Playlists" }
                div {  class: "item-grid",
                    for (id, name) in playlists() {
                        {
                            let id = id;
                            let name = name.clone();
                            rsx! {
                                playlist_item {
                                    playlist_id: id,
                                    playlist_name: name.clone(),
                                    on_select: move |_| {
                                        on_select.call((id, name.clone()));
                                    },
                                    on_delete: move |_| {
                                        refresh_trigger.set(refresh_trigger() + 1);
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
fn playlist_item(
    playlist_id: i32,
    playlist_name: String,
    on_select: EventHandler<()>,
    on_delete: EventHandler<()>,
) -> Element {
    let mut show_context_menu = use_signal(|| false);
    let mut context_menu_pos = use_signal(|| (0.0, 0.0));
    let db = use_context::<DatabaseContext>();

    let db_clone = db.clone();
    let delete_playlist = move || {
        if let Ok(_) = db_clone.get().delete_playlist(playlist_id) {
            on_delete.call(());
        }
    };

    let context_menu_items = vec![
        ContextMenuItem {
            title: "Open".to_string(),
            action: EventHandler::new(move |_| {
                on_select.call(());
            }),
        },
        ContextMenuItem {
            title: "Delete Playlist".to_string(),
            action: EventHandler::new(move |_| {
                delete_playlist();
            }),
        },
    ];

    rsx! {
        button {
            class: "file-item",
            onclick: move |_| {
                on_select.call(());
            },
            oncontextmenu: move |evt: Event<MouseData>| {
                evt.prevent_default();
                evt.stop_propagation();
                context_menu_pos.set((evt.client_coordinates().x, evt.client_coordinates().y));
                show_context_menu.set(true);
            },
            div { class: "item-icon", "📋" }
            div { class: "item-name", "{playlist_name}" }
        }
        context_menu {
            pos: context_menu_pos,
            show_context_menu,
            items: context_menu_items
        }
    }
}

#[component]
fn playlist_content(playlist_id: i32, playlist_name: String, on_back: EventHandler<()>) -> Element {
    let db = use_context::<DatabaseContext>();
    let mut refresh_trigger = use_context::<PlayerContext>().playlist_update_counter.clone();
    let mut player_context = use_context::<PlayerContext>();
    let db_clone = db.clone();
    let songs = use_memo(move || {
        let _ = refresh_trigger();
        db_clone.get().get_songs_in_playlist(playlist_id).unwrap_or_default()
    });

    let mut current_playlist = use_memo(move || {
        let _ = refresh_trigger();
        Playlist::get_playlist_handle(playlist_id, db.arc().clone()).ok()
    });

    rsx! {
        div {
            class: "content-section",
            style: "margin-bottom: 300px;",
            h2 { "{playlist_name}" },
            div { class: "common-button",
                style: "margin-bottom: 10px;
                        justify-content: left;",
                button {
                    onclick: move |_| on_back.call(()),
                    "← Back to Playlists"
                }
            }
            div {
                class: "item-list",
                for (idx, song) in songs().iter().cloned().enumerate() {
                    {
                        let song = song.clone();
                        rsx! {
                            song_file {
                                file: FileEntry::from_song_view(&song),
                                on_click: move |_path| {
                                    player_context.queue.write().play_song_instant(&song);

                                    if current_playlist.read().is_some() {
                                        let mut playlist = current_playlist.read().as_ref().unwrap().clone();
                                        playlist.set_current(idx);
                                        *player_context.queue.write().current_fallback_queue.write() =
                                         QueueFallbackMode::Playlist { playlist: playlist };
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