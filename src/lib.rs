/*!
# Etag `if-none-match` Request Guard for Rocket Framework

This crate provides a request guard used for getting `if-none-match` header.

See `examples`.
*/

pub extern crate entity_tag;

use entity_tag::EntityTag;

use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};

/// The request guard used for getting `if-none-match` header.
#[derive(Debug, Clone, Default)]
pub struct EtagIfNoneMatch<'r> {
    pub etag: Option<EntityTag<'r>>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for EtagIfNoneMatch<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let raw_etag: Option<&str> = request.headers().get("if-none-match").next(); // Only fetch the first one.

        let etag = raw_etag.map(|raw_etag| EntityTag::from_str(raw_etag).ok()).unwrap_or(None);

        Outcome::Success(EtagIfNoneMatch {
            etag,
        })
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r EtagIfNoneMatch<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        Outcome::Success(request.local_cache(|| {
            let raw_etag: Option<&str> = request.headers().get("if-none-match").next(); // Only fetch the first one.

            let etag =
                raw_etag.map(|raw_etag| EntityTag::from_string(raw_etag).ok()).unwrap_or(None);

            EtagIfNoneMatch {
                etag,
            }
        }))
    }
}

impl<'r> EtagIfNoneMatch<'r> {
    /// For weak comparison two entity-tags are equivalent if their opaque-tags match character-by-character, regardless of either or both being tagged as "weak".
    pub fn weak_eq(&self, other_etag: &EntityTag) -> bool {
        match &self.etag {
            Some(etag) => etag.weak_eq(other_etag),
            None => false,
        }
    }

    /// For strong comparison two entity-tags are equivalent if both are not weak and their opaque-tags match character-by-character.
    pub fn strong_eq(&self, other_etag: &EntityTag) -> bool {
        match &self.etag {
            Some(etag) => etag.strong_eq(other_etag),
            None => false,
        }
    }
}
