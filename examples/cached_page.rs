#[macro_use]
extern crate rocket;

use std::{borrow::Cow, io::Cursor};

use chrono::prelude::*;
use rocket::{
    http::Status,
    request::Request,
    response::{Responder, Response, Result},
};
use rocket_etag_if_none_match::{EtagIfNoneMatch, entity_tag::EntityTag};

// SAFETY: `MAGIC` is a valid unquoted entity-tag value. `new_unchecked` is the only constructor that can be called in a `static`.
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
    let mut response = Response::build();
    response.raw_header(EntityTag::HEADER_NAME, MY_ETAG.to_string());

    if etag_if_none_match.weak_eq(&MY_ETAG) {
        println!("Cached!");
        response.status(Status::NotModified);
    } else {
        let body = format!(
            "Current Time: {}\n\nTry to re-open this page repeatedly without pressing the \
             forced-refresh(Ctrl+F5) button.",
            Utc::now().to_rfc3339()
        );

        let size = body.len();

        response
            .raw_header("Content-Type", "text/plain; charset=utf-8")
            .sized_body(size, Cursor::new(body));
    }

    response.finalize().into()
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
