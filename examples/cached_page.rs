#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

extern crate rocket_etag_if_none_match;

#[macro_use]
extern crate lazy_static;

use std::io::Cursor;

use rocket_etag_if_none_match::{EtagIfNoneMatch, EntityTag};

use rocket::response::{Result, Response};
use rocket::http::Status;
use rocket::http::hyper::header::ETag;

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
        Response::build().header(ETag(MY_ETAG.clone())).sized_body(Cursor::new("Hello!")).ok()
    }
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();
}