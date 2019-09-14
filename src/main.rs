#![recursion_limit = "1024"]
#![feature(proc_macro_hygiene, decl_macro)]

use maud::{html, Markup};
use rocket::{get, routes};
use std::borrow::Cow;
use std::env;
use std::fs;

extern crate reqwest;
extern crate rocket;

#[macro_use]
extern crate error_chain;

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {
        foreign_links {
            Io(std::io::Error);
            Core(std::num::ParseIntError);
            Reqwest(reqwest::Error);
        }
        errors { RandomResponseError(t: String) }
    }
}
use errors::*;

#[get("/")]
fn index() -> Markup {
    return match build_html() {
        Ok(result) => result,
        std::result::Result::Err(error) => match *error.kind() {
            _ => {
                let e: std::string::String = format!("Other error: {:?}", error);
                html! {h1 {  "CRITICAL: " (e) " please help me deal with this!"}}
            }
        },
    };
}

fn build_html() -> Result<Markup> {
    let content = html! {
        h1 { "Hello, " ("name") ("4".parse::<u32>()?) "!" }
        p { "This is a paragraph!" }
        p { "Hello World" }
    };
    return Ok(content);
}

fn read_file(filepath: &str) -> Result<String> {
    let contents = fs::read_to_string(filepath)?;
    return Ok(contents);
}

fn query_sewobe(id: u16) -> Result<String> {
    let params = [("foo", "bar"), ("baz", "quux")];
    let client = reqwest::Client::new();
    let mut res = client
        .post("http://httpbin.org/post")
        .form(&params)
        .send()?;
    let txt = res.text()?;
    return Ok(txt);
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
