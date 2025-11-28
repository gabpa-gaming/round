
use dioxus::prelude::*;

use crate::app_context::{PlaybackMode, PlayerContext};



#[component]
pub fn player_sidebar() -> Element {
    rsx! {
        div { class: "now-playing-sidebar",
                div { class: "content-section",
                    song_metadata_view { }
                }
                div { class: "song-view-container",
                        song_progress_bar { }
                }
        }
    }
}

#[component]
pub fn song_metadata_view() -> Element {
    let player_state = use_context::<PlayerContext>();

    let current_track = use_memo(move || player_state.playing_state.read().current_song().clone());

    let playing = 
        if let Some(song) = current_track() {
            rsx! {
                div { class: "song-metadata",
                    style: "font-size: 14px;",
                    { song.artist },
                    { " - " },
                    { song.title }
                    div { class: "album-art",
                        { if let Some(art_path) = &song.album_art_path {
                            rsx! {
                                img {
                                    src: "{art_path}",
                                    alt: "Album Art",
                                    width: "100%",
                                    height: "100%",
                                }
                            }
                        } else {
                            rsx! {
                                div { class: "no-album-art",
                                    "🎵"
                                }
                            }
                        } }
                    }
                }
            }
        } else {
            rsx! {
                div {
                    style: "font-size: 14px;",
                    h3 { "nothing" }
                }
            }
        };

    rsx! {
        p { 
            "currently playing:"
            { playing }
        }
    }
}


#[component]
pub fn song_progress_bar() -> Element {
    let player_state = use_context::<PlayerContext>();

    let mut playing_state = player_state.playing_state.clone();
    let current_song = use_memo(move || playing_state.read().current_song());
    
    let progress_ms = use_memo(move || playing_state.read().progress());
    let song_len_ms = use_memo(
        move || current_song().map_or(0, |song| song.duration_seconds * 1000)
    );

    let value = use_memo(
        move || {
            let p = progress_ms() as f32;
            let d = song_len_ms() as f32;
            let percentage = if d > 0.0 { (p / d) * 100.0 } else { 0.0 };
            percentage
        }
    );

    let set_progress = move || {
        let playing_state = player_state.playing_state.clone();

        player_state.set_current_progress(playing_state.read().progress());
    };

    let style_str = use_memo(
        move || {
            format!("--slider-progress: {}%;", value())
        }
    );
    rsx! { 
        div { class: "slider-container",
            style: style_str,
            input {
                class: "slider",
                r#type: "range",
                min: "0",
                max: "100",
                value: value,
                oninput: move |e| {
                    if let Ok(val) = e.value().parse::<u64>() {
                        if let Some(progress) = playing_state.clone().write().progress_mut() {
                            *progress = val * song_len_ms() / 100;
                        }
                    }
                    set_progress();
                },
                disabled: playing_state.read().current_song().is_none().to_string(),
            }
            song_length_marker{ len: song_len_ms,  prog: progress_ms }
            playback_controls { }
        }
    }
}

#[component]
pub fn song_length_marker(len: ReadSignal<u64>, prog: ReadSignal<u64>) -> Element {
    let remaining = use_memo(move || {
        let len = *len.read();
        let prog = *prog.read();
        let len =  if prog > len { prog } else { len }; //prevent overflow when len is not updated yet
        let remaining = len / 1000 - prog / 1000;       //happens for one frame only when song finished and another song is loaded 
        let minutes = remaining / 60;
        let seconds = remaining % 60;
        format!("-{:02}:{:02}", minutes, seconds)
    });

    let progressed = use_memo(move || {
        let progressed_seconds = *prog.read() / 1000;
        let minutes = progressed_seconds / 60;
        let seconds = progressed_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    });

    rsx! {
        div { class: "song-length-marker",
            span { { progressed } }
            span { { remaining } }
        }
    }
}

#[component]
pub fn playback_controls() -> Element {
    let player_state = use_context::<PlayerContext>();
    
    let is_playing = use_memo(move || player_state.playing_state.read().is_playing());

    let mut prev_song = {
        let mut ps = player_state.clone();
        move || {
            ps.previous_song();
        }
    };

    let mut next_song = {
        let mut ns = player_state.clone();
        move || {
            ns.next_song();
        }
    };

    let mut toggle_playback = {
        let mut ps = player_state.clone();
        move || {
            if is_playing() {
                ps.pause();
            } else {
                ps.play();
            }
        }
    };

    let mut toggle_mode = {
        let mut ps = player_state.clone();
        let mode = *ps.mode.read();
        move || {
            match mode {
                PlaybackMode::Normal => {
                    *ps.mode.write() = PlaybackMode::Shuffle;
                }
                PlaybackMode::Shuffle => {
                    *ps.mode.write() = PlaybackMode::Loop;
                }
                PlaybackMode::Loop => {
                    *ps.mode.write() = PlaybackMode::Normal;
                }
            }
        }
    };
    
    rsx! {
        div { class: "controls-container",
            div { class: "common-button",
                button {
                    onclick: move |_| {
                        prev_song();
                    },
                    {"⏮"}
                }
                button {
                    onclick: move |_| {
                        toggle_playback();
                    },
                    if is_playing() { "⏸" } else { "▶" }
                }
                button {
                    onclick: move |_| {
                        next_song();
                    },
                    {"⏭"}
                }
            }
            div { class: "right-controls",
                button {
                    onclick: move |_| {
                        toggle_mode();
                    },
                    match *player_state.mode.read() {
                        PlaybackMode::Normal => "→",
                        PlaybackMode::Shuffle => "⚂",
                        PlaybackMode::Loop => "↻",
                    }
                }
            }
        }
    }
}