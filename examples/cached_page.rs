#[macro_use]
extern crate rocket;

extern crate rocket_etag_if_none_match;

extern crate chrono;

extern crate once_cell;

use std::io::Cursor;

use rocket_etag_if_none_match::{etag::EntityTag, EtagIfNoneMatch};

use rocket::http::Status;
use rocket::response::{Response, Result};

use once_cell::sync::Lazy;

use chrono::prelude::*;

static MY_ETAG: Lazy<EntityTag> = Lazy::new(|| EntityTag::new(true, "MAGIC"));

#[get("/")]
fn index(etag_if_none_match: &EtagIfNoneMatch) -> Result<'static> {
    if etag_if_none_match.weak_eq(&MY_ETAG) {
        println!("Cached!");
        Response::build().status(Status::NotModified).ok()
    } else {
        let body = format!("Current Time: {}\n\nTry to re-open this page repeatedly without pressing the forced-refresh(Ctrl+F5) button.", Utc::now().to_rfc3339());

        let size = body.len();

        Response::build()
            .raw_header("etag", MY_ETAG.to_string())
            .raw_header("Content-Type", "text/plain; charset=utf-8")
            .sized_body(size, Cursor::new(body))
            .ok()
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
