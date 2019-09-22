#![recursion_limit = "1024"]
#![feature(proc_macro_hygiene, decl_macro)]

use maud::DOCTYPE;
use maud::{html, Markup};
use rocket::{get, routes};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::prelude::*;

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
            Env(std::env::VarError);
            Core(std::num::ParseIntError);
            Reqwest(reqwest::Error);
            Json(serde_json::Error);
            OptionNone(NoneOptionError);
        }
        errors { RandomResponseError(t: String) }
    }

    #[derive(Debug, Clone)]
    pub struct NoneOptionError;
    impl std::fmt::Display for NoneOptionError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "Option::None was assumed to be something")
        }
    }
    impl std::error::Error for NoneOptionError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            // Generic error, underlying cause isn't tracked.
            None
        }
    }
}
use errors::*;

use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

struct SharedData {
    response_cache: Mutex<(Instant, Markup)>,
}

fn get_server_cache_duration() -> Duration {
    return Duration::from_secs(300u64);
}

#[get("/")]
fn index(shared: rocket::State<SharedData>) -> Markup {
    let method_start = Instant::now();
    let mutex: &Mutex<(Instant, Markup)> = &(shared.response_cache);
    let mut cache_value = mutex.lock().expect("lock shared data");
    let now: Instant = Instant::now();
    let before: Instant = cache_value.0;
    if (before + get_server_cache_duration()) < now {
        let val = render_html_with_errors();
        let now = Instant::now();
        let return_value = html! {(val)
        div style="display:none" { (fmt_duration(method_start, now)) " for uncached" }};
        *cache_value = (now, val);
        return return_value;
    } else {
        let now = Instant::now();
        let c = html! {
        (cache_value.1)
        div style="display:none" { (fmt_duration(method_start, now)) " for cached" }
        };
        return c;
    }
}

fn fmt_duration(instant1: Instant, instant2: Instant) -> String {
    let d = instant2 - instant1;
    return format!(
        "{} s, {} ms, {} micros",
        d.as_secs(),
        d.subsec_millis(),
        d.subsec_micros()
    );
}

fn render_html_with_errors() -> Markup {
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

fn read_file(filepath: &str) -> Result<String> {
    let contents = fs::read_to_string(filepath)?;
    return Ok(contents);
}

fn write_file(filepath: &str, content: &str) -> Result<()> {
    let mut f = std::fs::File::create(filepath)?;
    let cp = content.to_owned();
    let cp_bytes = cp.as_bytes();
    f.write_all(cp_bytes)?;
    return Ok(());
}

fn query_sewobe(id: u16) -> Result<String> {
    let params = [
        ("USERNAME", env::var("SEWOBEUSER")?),
        ("PASSWORT", env::var("SEWOBEPASSWORD")?),
        ("AUSWERTUNG_ID", id.to_string()),
    ];
    let client = reqwest::Client::new();
    let url = env::var("SEWOBEURL")?;
    let mut res = client.post(&url).form(&params).send()?;
    let txt = res.text()?;
    return Ok(txt);
}

fn main() {
    rocket::ignite()
        .manage(SharedData {
            response_cache: Mutex::new((Instant::now(), render_html_with_errors())),
        })
        .mount("/", routes![index])
        .launch();
}

fn check_if_json(s: &str) -> bool {
    let opt: serde_json::Result<serde_json::Value> = serde_json::from_str(&s);
    return opt.is_ok();
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
        html lang="de" {
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
        meta name = "viewport" content = "width=device-width, initial-scale=1, maximum-scale=10" {}
        link rel="preload" href="https://stackpath.bootstrapcdn.com/bootstrap/4.3.1/css/bootstrap.min.css" as="style" onload="this.onload=null;this.rel='stylesheet'" {}
        noscript {
            link  rel = "stylesheet" type="text/css" crossorigin="anonymous" href = "https://stackpath.bootstrapcdn.com/bootstrap/4.3.1/css/bootstrap.min.css" {}
        }
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
    let users = get_elected_users()?;
    let content = html! {
        h2 { "Ämterliste" }
        p { "Diese Liste wird auf Basis der SEWOBE-Datenbank jede Nacht neu erstellt. Unbesetzte Ämter werden nicht angezeigt." }
        div {
            div class = "table-responsive" {
                table class = "table" {
                    thead {
                        tr {
                            th { "Amt" }
                            th { "Amtsträger" }
                            th { "Neuwahl" }
                        }
                    }
                    tbody {
                        @for user in users {
                            tr {
                                td { (user.job_title) }
                                td {
                                    @if user.first_name == VAKANT {
                                        (VAKANT)
                                    } @else {
                                        a href=(format!("mailto:{}", user.email)) target="_top" {(user.first_name) " (" (user.nick_name) ") " (user.sur_name)}
                                    }
                                }
                                td { (user.reelection_date) }
                            }
                        }
                    }
                }
            }
        }
    };
    return Ok(content);
}

fn separate_file_html(title: &str, description: &str, file_id: TxtFiles) -> Result<Markup> {
    let content = html! {
        h2 { (title) }
        p { (description) }
        pre { (load(file_id)?) }
    };
    return Ok(content);
}

#[derive(Clone, Debug)]
enum TxtFiles {
    ActiveRedirections,
    JobRedirections,
    MailmanLists,
    ElectedUserJson,
    SecondaryElectionJson,
}

fn load(file_id: TxtFiles) -> Result<String> {
    match file_id {
        TxtFiles::JobRedirections => read_file("./data/aemtermails.txt"),
        TxtFiles::MailmanLists => read_file("./data/mailmanmails.txt"),
        TxtFiles::ActiveRedirections => read_file("./data/mails.txt"),
        TxtFiles::ElectedUserJson => read_query_with_fallback_file("./tmp/aemter.json", 170u16),
        TxtFiles::SecondaryElectionJson => {
            read_query_with_fallback_file("./tmp/aemter27.json", 27u16)
        }
    }
}

/* first query sewobe, but if no good result, use local file (which stores previous successful sewobe calls) */
fn read_query_with_fallback_file(filepath: &str, sewobe_id: u16) -> Result<String> {
    let res = query_sewobe(sewobe_id)?;
    if check_if_json(&res) {
        //write to file
        write_file(filepath, &res)?;
        //return value
        return Ok(res);
    } else {
        return read_file(filepath);
    }
}

fn get_elected_users() -> Result<Vec<ElectedUser>> {
    let primary_json = load(TxtFiles::ElectedUserJson)?;
    let sec_json = load(TxtFiles::SecondaryElectionJson)?;

    let mut offices: HashMap<String, Vec<ElectedUser>> = HashMap::new();
    //parse found json manually into elected user structs
    let users_raw: serde_json::Value = serde_json::from_str(&primary_json)?;
    for datensatz_raw in users_raw
        .as_object()
        .ok_or_else(|| NoneOptionError)?
        .values()
    {
        let raw_user = datensatz_raw.as_object().ok_or_else(|| NoneOptionError)?["DATENSATZ"]
            .as_object()
            .ok_or_else(|| NoneOptionError)?;
        let all_amt = raw_user["AMT"]
            .as_str()
            .ok_or_else(|| NoneOptionError)?
            .to_owned();
        let aemter: Vec<String> = all_amt.split(",").map(|k| k.to_owned()).collect();

        for amt in aemter {
            if amt.trim().len() > 0 {
                if !offices.contains_key(&amt) {
                    offices.insert(amt.to_owned(), vec![]);
                }
                let ls: &mut Vec<ElectedUser> =
                    offices.get_mut(&amt).ok_or_else(|| NoneOptionError)?;
                ls.push(ElectedUser {
                    job_title: amt.to_owned(),
                    email: raw_user["E-MAIL"]
                        .as_str()
                        .ok_or_else(|| NoneOptionError)?
                        .to_owned(),
                    first_name: raw_user["VORNAME-PRIVATPERSON"]
                        .as_str()
                        .ok_or_else(|| NoneOptionError)?
                        .to_owned(),
                    sur_name: raw_user["NACHNAME-PRIVATPERSON"]
                        .as_str()
                        .ok_or_else(|| NoneOptionError)?
                        .to_owned(),
                    nick_name: raw_user["BIERNAME"]
                        .as_str()
                        .ok_or_else(|| NoneOptionError)?
                        .to_owned(),
                    reelection_date: format!(
                        "{} {}",
                        raw_user["NEUWAHL"]
                            .as_str()
                            .ok_or_else(|| NoneOptionError)?
                            .to_owned(),
                        raw_user["JAHR"]
                            .as_str()
                            .ok_or_else(|| NoneOptionError)?
                            .to_owned()
                    ),
                });
            }
        }
    }

    //after: fill up unfilled offices with VAKANT
    let aemter_raw: serde_json::Value = serde_json::from_str(&sec_json)?;
    for datensatz_raw in aemter_raw
        .as_object()
        .ok_or_else(|| NoneOptionError)?
        .values()
    {
        let datensatz = datensatz_raw.as_object().ok_or_else(|| NoneOptionError)?;
        let amt: String = datensatz["DATENSATZ"]
            .as_object()
            .ok_or_else(|| NoneOptionError)?["AMT"]
            .as_str()
            .ok_or_else(|| NoneOptionError)?
            .to_owned();
        if !offices.contains_key(&amt) {
            offices.insert(
                amt.to_owned(),
                vec![ElectedUser {
                    job_title: amt.to_owned(),
                    email: "".to_owned(),
                    first_name: VAKANT.to_owned(),
                    sur_name: "".to_owned(),
                    nick_name: "".to_owned(),
                    reelection_date: "N/A".to_owned(),
                }],
            );
        }
    }
    let mut offices_list: Vec<(String, Vec<ElectedUser>)> = offices
        .iter()
        .map(|k| (k.0.to_owned(), k.1.clone()))
        .collect();
    offices_list.sort_unstable_by_key(|k| k.0.to_owned());
    let offices_flat: Vec<ElectedUser> = offices_list
        .iter()
        .flat_map(|k| {
            let mut v = k.1.clone();
            v.sort_unstable_by_key(|u| u.first_name.to_owned());
            return v.into_iter();
        })
        .map(|u| u.clone())
        .collect();

    return Ok(offices_flat);
}

#[derive(Clone, Debug)]
struct ElectedUser {
    job_title: String,
    email: String,
    first_name: String,
    sur_name: String,
    nick_name: String,
    reelection_date: String,
}
