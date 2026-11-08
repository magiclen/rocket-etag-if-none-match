Etag `if-none-match` Request Guard for Rocket Framework
====================

[![CI](https://github.com/magiclen/rocket-etag-if-none-match/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/rocket-etag-if-none-match/actions/workflows/ci.yml)

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

`If-None-Match` uses weak comparison, so use `EtagIfNoneMatch::weak_eq` to check this condition.
`EtagIfNoneMatch::strong_eq` is available for custom strong comparisons.

The field can list several entity-tags across one or more lines.
A single `*` matches any current representation.
The guard ignores the whole field if any line it receives is invalid.
Rocket drops field lines with invalid UTF-8 before the guard sees them.

## Crates.io

https://crates.io/crates/rocket-etag-if-none-match

## Documentation

https://docs.rs/rocket-etag-if-none-match

## License

[MIT](LICENSE)