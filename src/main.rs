#![recursion_limit = "1024"]
#![feature(proc_macro_hygiene, decl_macro)]

use maud::DOCTYPE;
use maud::{html, Markup};
use rocket::{get, routes};
use std::borrow::Cow;
use std::env;
use std::fs;

extern crate reqwest;
extern crate rocket;
extern crate serde;
extern crate serde_json;

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
    return match generate_full_html() {
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

fn check_if_json(s: &str) -> bool {
    let opt: serde_json::Result<serde_json::Value> = serde_json::from_str(&s);
    return opt.is_ok();
}

fn millis_duration_between_caching() -> u32 {
    return 5u32 * 60u32 * 1_000u32;
}

const RESERVIERUNGEN_LINKS_TITLE: &'static [&'static str] = &[
    "Gästezimmer Karlsruhe (Waldhornstraße)",
    "Saal Karlsruhe",
    "Skihütte",
    "Gästezimmer Berlin (Carmerstraße)",
];
const RESERVIERUNGEN_LINKS_URL: &'static [&'static str] = &[
    "https://reservierungen.av-huette.de/v2/index.html?selected_key=gaestezimmer_ka",
    "https://reservierungen.av-huette.de/v2/index.html?selected_key=saal_ka",
    "https://reservierungen.av-huette.de/v2/index.html?selected_key=skihuette",
    "https://reservierungen.av-huette.de/v2/index.html?selected_key=carmerstrasze",
];
const SONSTIGE_LINKS_TITLE: &'static [&'static str] = &[
    "Webseite",
    "SEWOBE Mitgliederportal",
    "SEWOBE Ämterportal (nur für relevante Amtsträger)",
];
const SONSTIGE_LINKS_URL: &'static [&'static str] = &[
    "https://www.av-huette.de/",
    "https://server30.der-moderne-verein.de/portal/index.php",
    "https://server30.der-moderne-verein.de/module/login.php",
];

const VAKANT: &'static str = "vakant";

fn generate_full_html() -> Result<Markup> {
    //add header
    //add body
    let content = html! {
        (DOCTYPE)
        html {
            head {
                (generate_header_html())
            }
            body {
                (generate_body_html()?)
            }
        }
    };
    return Ok(content);
}

fn generate_header_html() -> Markup {
    return html! {
        meta charset="utf-8" {}
        meta name = "viewport" content = "width=device-width, initial-scale=1, maximum-scale=1" {}
        link  rel = "stylesheet" type="text/css" href = "https://stackpath.bootstrapcdn.com/bootstrap/4.3.1/css/bootstrap.min.css" {}
        title { "AVH Portal" }
    };
}

fn generate_body_html() -> Result<Markup> {
    let content = html! {
        div class="container" {
            div class="preamble" {
                h1 {"AVH Portal"}
                p { "Willkommen auf dem Selbstbedienungsportal des Akademischen Verein Hütte" }
                h2 {"Reservierungen"}
                (make_link_list(RESERVIERUNGEN_LINKS_TITLE, RESERVIERUNGEN_LINKS_URL))
                h2 {"Sonstige Links"}
                (make_link_list(SONSTIGE_LINKS_TITLE, SONSTIGE_LINKS_URL))
            }
            (elected_user_html()?)
            hr {}
            (separate_file_html("Mailinglisten / aktive Weiterleitungen", r#"Dies sind die aktiven Mail-Weiterleitungen auf dem av-huette-Mailserver. Diese Liste ist im Format "x:y" wobei alle Mails an "x@av-huette.de" an Adresse "y" weitergeleitet werden. Diese Liste wird jeden Tag um 2 Uhr nachts automatisch neu generiert auf Basis der SEWOBE Datenbank."#, TxtFiles::ActiveRedirections)?)
            (separate_file_html("Mailadressen der Ämter", r#"Dies sind die aktiven Mail-Weiterleitungen der Ämter auf dem av-huette-Mailserver. Diese Liste ist im Format "x:y" wobei alle Mails an "x@av-huette.de" an Adresse "y" weitergeleitet werden. Diese Liste wird jeden Tag um 2 Uhr nachts automatisch neu generiert auf Basis der SEWOBE Datenbank."#, TxtFiles::JobRedirections)?)
            (separate_file_html("Aktive Mailman-Verteiler", r#"Dies sind die aktiven Mailman-Verteilerlisten ( = was für Verteiler gibt es überhaupt) auf dem av-huette-Mailserver. Diese Liste ist im Format "x:y" wobei alle Mails an "x@av-huette.de" an Adresse "y" weitergeleitet werden."#, TxtFiles::MailmanLists)?)
        }
    };
    return Ok(content);
}

fn make_link_list(titles: &[&'static str], urls: &[&'static str]) -> Markup {
    //let indices = (0..(titles.len())).collect::<Vec<usize>>();
    return html! {
        ul {
            @for index in 0..(titles.len()) {
                li  {
                    a href=(urls[index]) {
                        (titles[index])
                    }
                }
            }
        }
    };
}

fn elected_user_html() -> Result<Markup> {
    let content = html! {
        "Not yet implemented"
    };
    return Ok(content);
}

fn separate_file_html(title: &str, description: &str, file_id: TxtFiles) -> Result<Markup> {
    let content = html! {
        "Not yet implemented"
    };
    return Ok(content);
}

enum TxtFiles {
    ActiveRedirections,
    JobRedirections,
    MailmanLists,
}
