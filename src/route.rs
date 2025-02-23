use dioxus::prelude::*;

use crate::components::Navbar;
use crate::views::{Config, Mtrack};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
    #[route("/mtrack")]
    Mtrack {},
    #[route("/config")]
    Config { },
}
