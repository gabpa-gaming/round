use dioxus::prelude::*;



use crate::{app_context::PlayerContext, player::scrolling_text};

#[component]
pub fn queue_bar() -> Element {
    let player_ctx = use_context::<PlayerContext>();

    let player_context = player_ctx.clone();
    let queue_update = use_effect( move || {
        let player_ctx = player_ctx.clone();
        if player_ctx.clone().is_finished() {
            player_ctx.clone().queue.write().next_song();
        }
    });
    let queue_items = use_memo( move || {
        let player_context = player_context.clone();
        player_context.clone().queue.read().get_names().iter().fold(String::new(), |acc, name| {
            format!("{} >> {}", acc, name)
        })
    });

    rsx! {
        div { class: "queue-bar",
            h3 { class: "up-next-label",
                "up next:"
            },

            div { class: "side-fadeout",
                scrolling_text {
                    chars_per_second: 20.0,
                    text: queue_items,
                }
            }
        }
    }
}