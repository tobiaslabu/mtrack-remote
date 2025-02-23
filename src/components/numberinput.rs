use dioxus::{
    logger::tracing::{debug, warn},
    prelude::*,
};

#[component]
pub fn NumberInput(value: Signal<u16>, default_value: u16) -> Element {
    rsx!(
        input {
            value,
            initial_value: default_value,
            r#type: "number",
            oninput: move |event| async move {
                match event.parsed::<u16>() {
                    Ok(v) => {
                        debug!("Setting new value {event:?}");
                        value.set(v)
                    }
                    Err(e) => {
                        warn!("Not setting new value! {e}");
                    }
                };
            },
        }
    )
}
