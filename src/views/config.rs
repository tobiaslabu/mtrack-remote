use dioxus::prelude::*;

use crate::components;

#[component]
pub fn Config() -> Element {
    rsx! {
        components::ConfigComponent {}
    }
}
