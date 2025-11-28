use dioxus::prelude::*;

use crate::app_context::{DatabaseContext, PlayerContext};

#[component]
pub fn create_playlist_dialog(
    show: Signal<bool>,
    on_created: EventHandler<i32>,
) -> Element {
    let db = use_context::<DatabaseContext>();
    let mut playlist_name = use_signal(|| String::new());
    let mut updater = use_context::<PlayerContext>().playlist_update_counter;
    if !show() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "context-menu-overlay",
            onclick: move |_| {
                show.set(false);
                playlist_name.set(String::new());
            },

            div {
                class: "context-menu",
                style: "left: 50%; top: 50%; transform: translate(-50%, -50%);
                        position: fixed;
                        padding: 25px;
                        padding-bottom: 30px;
                        ",
                onclick: move |evt: Event<MouseData>| evt.stop_propagation(),

                h3 { style: "margin-bottom: 24px;
                            font-size: 24px;
                            text-transform: uppercase;
                            text-align: center;",
                    "New playlist"
                }
                input {
                    r#type: "text",
                    style: "width: 100%; padding: 8px; font-size: 16px; box-sizing: border-box;",
                    value: "{playlist_name}",
                    oninput: move |evt| playlist_name.set(evt.value()),
                    onkeydown: {
                        let db_clone = db.clone();
                        move |evt| {
                            if evt.key() == Key::Enter && !playlist_name().is_empty() {
                                let name = playlist_name();
                                if let Ok(playlist_id) = db_clone.get().create_playlist(&name) {
                                    let mut write = updater.write();
                                    let val = *write;
                                    *write = val + 1;
                                    println!("Playlist created, new counter value: {}", *write);
                                    on_created.call(playlist_id);
                                } else {
                                    println!("Failed to create playlist: {}", name);
                                }
                                show.set(false);
                                playlist_name.set(String::new());
                            } else if evt.key() == Key::Escape {
                                show.set(false);
                                playlist_name.set(String::new());
                            }
                        }
                    },
                    placeholder: "Playlist name...",
                    autofocus: true,
                }
                div {
                    class: "common-button",
                    style: "display: flex; gap: 10px; margin-top: 10px;",
                    button {
                        onclick: {
                            let db_clone = db.clone();
                            move |_| {
                                let name = playlist_name();
                                if !name.is_empty() {
                                    if let Ok(playlist_id) = db_clone.get().create_playlist(&name) {
                                        let mut write = updater.write();
                                        let val = *write;
                                        *write = val + 1;
                                        println!("Playlist created, new counter value: {}", *write);
                                        on_created.call(playlist_id);
                                    }
                                    show.set(false);
                                    playlist_name.set(String::new());
                                }
                            }
                        },
                        disabled: playlist_name().is_empty(),
                        "Create"
                    }
                    button {
                        onclick: move |_| {
                            show.set(false);
                            playlist_name.set(String::new());
                        },
                        "Cancel"
                    }
                }
            }
        }
    }
}
