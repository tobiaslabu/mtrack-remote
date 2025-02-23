use dioxus::{logger::tracing::debug, prelude::*};

#[component]
pub fn Song(song: String, is_current: bool) -> Element {
    let class = match is_current {
        true => "current_song",
        false => "song",
    };

    rsx!(
        div { class, "{song}" }
    )
}

#[component]
pub fn Playlist(songs: Vec<String>, current_song: String) -> Element {
    use_effect(|| {
        debug!("Now I'd like to scroll to the current song..");
    });
    rsx!(
        div { class: "h-512 overflow-auto",
            ol {
                for (_i , element) in songs.iter().enumerate() {
                    li {
                        key: _i,
                        text_anchor: "{current_song}",
                        Song {
                            song: format!("{element}"),
                            is_current: element.ends_with(&current_song),
                        }
                    }
                }
            }
        }
    )
}
