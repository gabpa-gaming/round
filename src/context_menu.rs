use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct ContextMenuItem {
    pub title: String,
    pub action: EventHandler<()>,
}

#[component]
pub fn context_menu(
    pos: Signal<(f64, f64)>,
    show_context_menu: Signal<bool>,
    items: Vec<ContextMenuItem>,
) -> Element {
    if !show_context_menu() {
        return rsx! {};
    }

    let (x, y) = *pos.read();

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
                style: "left: {x}px; top: {y}px; position: fixed;",
                onclick: move |evt: Event<MouseData>| evt.stop_propagation(),
                
                for item in items.iter().cloned() {
                    button {
                        class: "context-menu-item",
                        onclick: move |_| {
                            item.action.call(());
                            show_context_menu.set(false);
                        },
                        "{item.title}"
                    }
                }
            }
        }
    }
}