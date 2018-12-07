#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

extern crate rocket_etag_if_none_match;

extern crate chrono;

#[macro_use]
extern crate lazy_static;

use std::io::Cursor;

use rocket_etag_if_none_match::{EtagIfNoneMatch, EntityTag};

use rocket::response::{Result, Response};
use rocket::http::Status;
use rocket::http::hyper::header::ETag;

use chrono::prelude::*;

lazy_static! {
    static ref MY_ETAG: EntityTag = {
        EntityTag::new(true, "MAGIC".to_string())
    };
}

#[get("/")]
fn index(etag_if_none_match: EtagIfNoneMatch) -> Result<'static> {
    if etag_if_none_match.weak_eq(&MY_ETAG) {
        println!("Cached!");
        Response::build().status(Status::NotModified).ok()
    } else {
        Response::build().header(ETag(MY_ETAG.clone()))
            .raw_header("Content-Type", "text/plain; charset=utf-8")
            .sized_body(Cursor::new(format!("Current Time: {}\n\nTry to re-open this page repeatedly without pressing the forced-refresh(Ctrl+F5) button.", Utc::now().to_rfc3339())))
            .ok()
    }
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();
}