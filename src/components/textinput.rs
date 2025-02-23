use dioxus::{logger::tracing::debug, prelude::*};

#[component]
pub fn TextInput(value: Signal<String>, default_value: String) -> Element {
    rsx!(
        input {
            value,
            initial_value: default_value,
            oninput: move |event| async move {
                debug!("Setting new value {event:?}");
                value.set(event.value())
            },
        }
    )
}
