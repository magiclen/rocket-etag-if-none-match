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
    NotModified((), Header<'static>),
}

#[get("/")]
fn index(etag_if_none_match: &EtagIfNoneMatch) -> CachedPage {
    let etag = EntityTag::with_str(true, "MAGIC").unwrap();
    let header = Header::new(EntityTag::HEADER_NAME, etag.to_string());

    if etag_if_none_match.weak_eq(&etag) {
        CachedPage::NotModified((), header)
    } else {
        CachedPage::Fresh("Hello world!", header)
    }
}
```

`If-None-Match` uses weak comparison, so use [`EtagIfNoneMatch::weak_eq`] to check this condition.
[`EtagIfNoneMatch::strong_eq`] is available for custom strong comparisons.

The field can list several entity-tags across one or more lines.
A single `*` matches any current representation.
The guard ignores the whole field if any line it receives is invalid.
Rocket drops field lines with invalid UTF-8 before the guard sees them.
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
    /// The requested entity-tags in the order they appear across the field lines.
    /// The list is empty when the field is missing, is `*`, or has an invalid value.
    pub etags: Vec<EntityTag<'r>>,
    /// Whether the field is a single `*`, which matches any current representation.
    pub any:   bool,
}

/// Parse one field line of the [`If-None-Match` list](https://www.rfc-editor.org/rfc/rfc9110.html#section-13.1.2).
fn parse_etags<'r>(mut value: &'r str, etags: &mut Vec<EntityTag<'r>>) -> Option<()> {
    loop {
        value = value.trim_start_matches([' ', '\t', ',']);

        if value.is_empty() {
            return Some(());
        }

        let (weak, remaining) = match value.strip_prefix("W/") {
            Some(remaining) => (true, remaining),
            None => (false, value),
        };

        // An opaque-tag has no escape sequence, so it ends at the next double quote.
        let (tag, remaining) = remaining.strip_prefix('"')?.split_once('"')?;
        etags.push(EntityTag::with_str(weak, tag).ok()?);

        let remaining = remaining.trim_start_matches([' ', '\t']);

        if remaining.is_empty() {
            return Some(());
        }

        value = remaining.strip_prefix(',')?;
    }
}

#[inline]
fn from_request<'r>(request: &'r Request<'_>) -> EtagIfNoneMatch<'r> {
    let mut values = request.headers().get("if-none-match");
    let Some(first) = values.next() else {
        return EtagIfNoneMatch::default();
    };

    if first.trim_matches([' ', '\t']) == "*" {
        // "*" must be the only value of the field, or the whole field is invalid.
        return EtagIfNoneMatch {
            any: values.next().is_none(),
            ..Default::default()
        };
    }

    let mut etags = Vec::new();

    for value in std::iter::once(first).chain(values) {
        if parse_etags(value, &mut etags).is_none() {
            return EtagIfNoneMatch::default();
        }
    }

    EtagIfNoneMatch {
        etags,
        any: false,
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
    /// Checks if any requested tag has the same value, even if either tag is weak.
    ///
    /// Use this method to check `If-None-Match`.
    /// It returns `true` for `*` and `false` when the field is missing or invalid.
    #[inline]
    pub fn weak_eq(&self, other_etag: &EntityTag) -> bool {
        self.any || self.etags.iter().any(|etag| etag.weak_eq(other_etag))
    }

    /// Checks if any requested tag has the same value and both tags are strong.
    ///
    /// Use this method for custom strong comparisons.
    /// It returns `true` for `*` and `false` when the field is missing or invalid.
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
