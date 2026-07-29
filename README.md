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

`if-none-match` is defined to use the weak comparison function, so `EtagIfNoneMatch::weak_eq` is the method a cache revalidation route wants. `EtagIfNoneMatch::strong_eq` is kept for the rare route that needs the strong one instead.

The header may carry more than one entity-tag, and it may also be `*`, which matches whatever representation the server currently has. Both forms are understood, so the two methods above answer whether *any* of the requested entity-tags matches.

## Crates.io

https://crates.io/crates/rocket-etag-if-none-match

## Documentation

https://docs.rs/rocket-etag-if-none-match

## License

[MIT](LICENSE)