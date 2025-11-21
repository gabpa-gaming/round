use crate::app_context::PlayerContext;
use crate::app_context::DatabaseContext;
use dioxus::prelude::*;
struct context_menu_item {
    title: String,
    action: EventHandler<()>,
}

#[component]
pub fn context_menu(show_context_menu: Signal<bool>) -> Element { //todo: make it generic
    let context_menu_x = use_state(|| 0);
    let context_menu_y = use_state(|| 0);
    let db = db.clone();
    let file_path = file_path.clone();
    let db_clone = db.clone();
    let file_path_clone = file_path.clone();
    rsx! {
        div {
            class: "context-menu-overlay",
            onclick: move |_| show_context_menu.set(false),
            oncontextmenu: move |evt: Event<MouseData>| {
                evt.prevent_default();
                show_context_menu.set(false);
            },
            div {
                class: "context-menu",
                style: "left: {context_menu_x()}px; top: {context_menu_y()}px;",
                onclick: move |evt: Event<MouseData>| evt.stop_propagation(),
                
                button {
                    class: "context-menu-item",
                    onclick: move |_| {
                        if let Ok(song) = db.get().get_song_view_by_path(&file_path) {
                            player_context.queue.write().play_song_instant(&song);
                        }
                        show_context_menu.set(false);
                    },
                    "Play"
                }
                
                button {
                    class: "context-menu-item",
                    onclick: move |_| {
                        if let Ok(song) = db_clone.get().get_song_view_by_path(&file_path_clone) {
                            player_context.queue.write().set_to_play_next(&song);
                        }
                        show_context_menu.set(false);
                    },
                    "Play Next"
                }
            }
        }
    }
}