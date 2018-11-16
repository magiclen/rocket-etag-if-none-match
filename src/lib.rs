/*!
# Etag `if-none-match` Request Guard for Rocket Framework

This crate provides a request guard used for getting `if-none-match` header.

See `examples`.
*/

extern crate rocket;

use rocket::Outcome;
use rocket::request::{self, Request, FromRequest};
pub use rocket::http::hyper::header::EntityTag;

/// The request guard used for getting `if-none-match` header.
#[derive(Debug, Clone)]
pub struct EtagIfNoneMatch {
    pub etag: Option<EntityTag>,
}

impl<'a, 'r> FromRequest<'a, 'r> for EtagIfNoneMatch {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<EtagIfNoneMatch, ()> {
        let raw_etag: Option<&str> = request.headers().get("if-none-match").next(); // Only fetch the first one.

        match raw_etag {
            Some(raw_etag) => match raw_etag.parse::<EntityTag>() {
                Ok(etag) => {
                    Outcome::Success(EtagIfNoneMatch {
                        etag: Some(etag)
                    })
                }
                Err(_) => {
                    Outcome::Success(EtagIfNoneMatch {
                        etag: None
                    })
                }
            }
            None => {
                Outcome::Success(EtagIfNoneMatch {
                    etag: None
                })
            }
        }
    }
}

impl EtagIfNoneMatch {
    /// For weak comparison two entity-tags are equivalent if their opaque-tags match character-by-character, regardless of either or both being tagged as "weak".
    pub fn weak_eq(&self, other_etag: &EntityTag) -> bool {
        match &self.etag {
            Some(etag) => {
                etag.weak_eq(other_etag)
            }
            None => {
                false
            }
        }
    }

    /// For strong comparison two entity-tags are equivalent if both are not weak and their opaque-tags match character-by-character.
    pub fn strong_eq(&self, other_etag: &EntityTag) -> bool {
        match &self.etag {
            Some(etag) => {
                etag.strong_eq(other_etag)
            }
            None => {
                false
            }
        }
    }
}