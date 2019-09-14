#![feature(proc_macro_hygiene, decl_macro)]

use maud::{html, Markup};
use rocket::{get, routes};
use std::borrow::Cow;

extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world from changed binary!"
}

#[get("/<name>")]
fn hello<'a>(name: Cow<'a, str>) -> Markup {
    html! {
        h1 { "Hello, " (name) "!" }
        p { "This is a paragraph!" }
        p { "Hello World" }
    }
}

fn main() {
    rocket::ignite().mount("/", routes![index, hello]).launch();
}
