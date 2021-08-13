#[macro_use]
extern crate rocket;

extern crate rocket_etag_if_none_match;

extern crate chrono;

use std::borrow::Cow;
use std::io::Cursor;

use rocket_etag_if_none_match::{entity_tag::EntityTag, EtagIfNoneMatch};

use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response, Result};

use chrono::prelude::*;

static MY_ETAG: EntityTag = unsafe { EntityTag::new_unchecked(true, Cow::Borrowed("MAGIC")) };

struct MyResponse<'r>(Response<'r>);

impl<'r, 'o: 'r> Responder<'r, 'o> for MyResponse<'o> {
    #[inline]
    fn respond_to(self, _: &'r Request<'_>) -> Result<'o> {
        Ok(self.0)
    }
}

impl<'r> From<Response<'r>> for MyResponse<'r> {
    #[inline]
    fn from(res: Response<'r>) -> Self {
        MyResponse(res)
    }
}

#[get("/")]
fn index(etag_if_none_match: EtagIfNoneMatch) -> MyResponse<'static> {
    if etag_if_none_match.weak_eq(&MY_ETAG) {
        println!("Cached!");
        Response::build().status(Status::NotModified).finalize().into()
    } else {
        let body = format!("Current Time: {}\n\nTry to re-open this page repeatedly without pressing the forced-refresh(Ctrl+F5) button.", Utc::now().to_rfc3339());

        let size = body.len();

        Response::build()
            .raw_header("Etag", MY_ETAG.to_string())
            .raw_header("Content-Type", "text/plain; charset=utf-8")
            .sized_body(size, Cursor::new(body))
            .finalize()
            .into()
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
