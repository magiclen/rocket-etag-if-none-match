/*!
# Etag `if-none-match` Request Guard for Rocket Framework

This crate provides a request guard used for getting `if-none-match` header.

```rust
use rocket::{get, http::Header, response::Responder};
use rocket_etag_if_none_match::{EtagIfNoneMatch, entity_tag::EntityTag};

#[derive(Responder)]
enum CachedPage {
    Fresh(&'static str, Header<'static>),
    #[response(status = 304)]
    NotModified(()),
}

#[get("/")]
fn index(etag_if_none_match: &EtagIfNoneMatch) -> CachedPage {
    let etag = EntityTag::with_str(true, "MAGIC").unwrap();

    if etag_if_none_match.weak_eq(&etag) {
        CachedPage::NotModified(())
    } else {
        CachedPage::Fresh("Hello world!", Header::new(EntityTag::HEADER_NAME, etag.to_string()))
    }
}
```

`if-none-match` is defined to use the weak comparison function, so [`EtagIfNoneMatch::weak_eq`] is the method a cache revalidation route wants. [`EtagIfNoneMatch::strong_eq`] is kept for the rare route that needs the strong one instead.

The header may carry more than one entity-tag, and it may also be `*`, which matches whatever representation the server currently has. Both forms are understood, so the two methods above answer whether *any* of the requested entity-tags matches.
*/

pub extern crate entity_tag;
pub extern crate rocket;

use entity_tag::EntityTag;
use rocket::{
    outcome::Outcome,
    request::{self, FromRequest, Request},
};

/// The request guard used for getting `if-none-match` header.
#[derive(Debug, Clone, Default)]
pub struct EtagIfNoneMatch<'r> {
    /// The requested entity-tags, in the order they appear in the header. It is empty when the header is absent, is `*`, or cannot be parsed.
    pub etags: Vec<EntityTag<'r>>,
    /// Whether the header is `*`, which matches any representation the server currently has.
    pub any:   bool,
}

/// Parse the `#entity-tag` list defined in [RFC 9110, section 8.8.3](https://www.rfc-editor.org/rfc/rfc9110.html#section-8.8.3).
fn parse_etags(mut raw_etag: &str) -> Vec<EntityTag<'_>> {
    let mut etags = Vec::new();

    loop {
        raw_etag = raw_etag.trim_start_matches([' ', '\t', ',']);

        if raw_etag.is_empty() {
            break;
        }

        let (weak, remaining) = match raw_etag.strip_prefix("W/") {
            Some(remaining) => (true, remaining),
            None => (false, raw_etag),
        };

        // An opaque-tag has no escape sequence, so it simply ends at the next double quote.
        let Some(remaining) = remaining.strip_prefix('"') else {
            break;
        };

        let Some(index) = remaining.find('"') else {
            break;
        };

        let (tag, remaining) = remaining.split_at(index);

        match EntityTag::with_str(weak, tag) {
            Ok(etag) => etags.push(etag),
            // The rest of the header cannot be trusted once an element is malformed.
            Err(_) => break,
        }

        raw_etag = &remaining[1..];
    }

    etags
}

#[inline]
fn from_request<'r>(request: &'r Request<'_>) -> EtagIfNoneMatch<'r> {
    let raw_etag: Option<&str> = request.headers().get("if-none-match").next(); // Only fetch the first one.

    match raw_etag {
        Some(raw_etag) => {
            let raw_etag = raw_etag.trim_matches([' ', '\t']);

            if raw_etag == "*" {
                EtagIfNoneMatch {
                    etags: Vec::new(), any: true
                }
            } else {
                EtagIfNoneMatch {
                    etags: parse_etags(raw_etag), any: false
                }
            }
        },
        None => EtagIfNoneMatch::default(),
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for EtagIfNoneMatch<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        Outcome::Success(from_request(request))
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for &'r EtagIfNoneMatch<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        Outcome::Success(request.local_cache(|| from_request(request).into_owned()))
    }
}

impl EtagIfNoneMatch<'_> {
    /// For weak comparison two entity-tags are equivalent if their opaque-tags match character-by-character, regardless of either or both being tagged as "weak".
    ///
    /// It returns `true` when the header is `*`, and `false` when the request has no `if-none-match` header.
    #[inline]
    pub fn weak_eq(&self, other_etag: &EntityTag) -> bool {
        self.any || self.etags.iter().any(|etag| etag.weak_eq(other_etag))
    }

    /// For strong comparison two entity-tags are equivalent if both are not weak and their opaque-tags match character-by-character.
    ///
    /// It returns `true` when the header is `*`, and `false` when the request has no `if-none-match` header.
    #[inline]
    pub fn strong_eq(&self, other_etag: &EntityTag) -> bool {
        self.any || self.etags.iter().any(|etag| etag.strong_eq(other_etag))
    }

    /// Extracts the owned data.
    #[inline]
    pub fn into_owned(self) -> EtagIfNoneMatch<'static> {
        EtagIfNoneMatch {
            etags: self.etags.into_iter().map(EntityTag::into_owned).collect(),
            any:   self.any,
        }
    }
}
