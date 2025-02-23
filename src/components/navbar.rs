use crate::route::Route;
use dioxus::prelude::*;

#[component]
fn NavLink(to: Route, children: Element) -> Element {
    rsx! {
        Link { class: "navlink w-48 text-center", to, children }
    }
}

#[component]
pub fn Navbar() -> Element {
    rsx! {
        div { id: "navbar",
            NavLink { to: Route::Mtrack {}, "mtrack" }
            NavLink { to: Route::Config {}, "config" }
        }

        Outlet::<Route> {}
    }
}
