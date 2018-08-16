//! # Etag `if-none-match` Request Guard for Rocket Framework
//! This crate provides a request guard used for getting `if-none-match` header.

extern crate rocket;

use rocket::Outcome;
use rocket::request::{self, Request, FromRequest};
use rocket::http::hyper::header::EntityTag;

/// The request guard used for getting `if-none-match` header.
pub struct EtagIfNoneMatch {
    pub etag: Option<EntityTag>,
}

impl<'a, 'r> FromRequest<'a, 'r> for EtagIfNoneMatch {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<EtagIfNoneMatch, ()> {
        let keys: Vec<_> = request.headers().get("if-none-match").collect();

        if keys.len() < 1 {
            return Outcome::Success(EtagIfNoneMatch {
                etag: None
            });
        }

        let key = keys[0];

        let etag = match key.parse::<EntityTag>() {
            Ok(etag) => etag,
            Err(_) => return Outcome::Success(EtagIfNoneMatch {
                etag: None
            })
        };

        Outcome::Success(EtagIfNoneMatch {
            etag: Some(etag)
        })
    }
}