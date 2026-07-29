//! Tests for the `EtagIfNoneMatch` request guard and the way it compares the requested entity-tags with the one of a resource.

use rocket::{
    Build, Rocket, get,
    http::{Header, Status},
    local::blocking::Client,
    response::Responder,
    routes,
};
use rocket_etag_if_none_match::{EtagIfNoneMatch, entity_tag::EntityTag};

/// The entity-tag of the resource that `/weak` and `/borrowed` serve.
fn weak_etag() -> EntityTag<'static> {
    EntityTag::with_str(true, "MAGIC").unwrap()
}

/// The entity-tag of the resource that `/strong` serves.
fn strong_etag() -> EntityTag<'static> {
    EntityTag::with_str(false, "MAGIC").unwrap()
}

#[derive(Responder)]
enum CachedPage {
    Fresh(&'static str, Header<'static>),
    #[response(status = 304)]
    NotModified(()),
}

fn cached_page(etag: &EntityTag<'_>, matched: bool) -> CachedPage {
    if matched {
        CachedPage::NotModified(())
    } else {
        CachedPage::Fresh("Hello world!", Header::new(EntityTag::HEADER_NAME, etag.to_string()))
    }
}

#[get("/weak")]
fn weak(etag_if_none_match: EtagIfNoneMatch) -> CachedPage {
    cached_page(&weak_etag(), etag_if_none_match.weak_eq(&weak_etag()))
}

#[get("/strong")]
fn strong(etag_if_none_match: EtagIfNoneMatch) -> CachedPage {
    cached_page(&strong_etag(), etag_if_none_match.strong_eq(&strong_etag()))
}

#[get("/borrowed")]
fn borrowed(etag_if_none_match: &EtagIfNoneMatch) -> CachedPage {
    cached_page(&weak_etag(), etag_if_none_match.weak_eq(&weak_etag()))
}

#[get("/cached")]
fn cached(first: &EtagIfNoneMatch, second: &EtagIfNoneMatch) -> String {
    std::ptr::eq(first, second).to_string()
}

fn rocket_with_routes() -> Rocket<Build> {
    rocket::build().mount("/", routes![weak, strong, borrowed, cached])
}

/// Send a `GET` request to `path`, with an `if-none-match` header when `raw_etag` is given.
fn get(client: &Client, path: &str, raw_etag: Option<&str>) -> (Status, Option<String>) {
    let mut request = client.get(path.to_string());

    if let Some(raw_etag) = raw_etag {
        request = request.header(Header::new("if-none-match", raw_etag.to_string()));
    }

    let response = request.dispatch();

    (response.status(), response.headers().get_one("ETag").map(|etag| etag.to_string()))
}

// A request without the header gets the whole resource, together with its entity-tag.
#[test]
fn a_request_without_the_header_gets_the_resource() {
    let client = Client::tracked(rocket_with_routes()).unwrap();

    assert_eq!((Status::Ok, Some(r#"W/"MAGIC""#.to_string())), get(&client, "/weak", None));
}

// The entity-tag the resource has just been served with revalidates the cached copy.
#[test]
fn a_matching_entity_tag_revalidates_the_cache() {
    let client = Client::tracked(rocket_with_routes()).unwrap();

    assert_eq!((Status::NotModified, None), get(&client, "/weak", Some(r#"W/"MAGIC""#)));
}

// An entity-tag of another resource does not revalidate anything.
#[test]
fn a_different_entity_tag_does_not_match() {
    let client = Client::tracked(rocket_with_routes()).unwrap();

    assert_eq!(
        (Status::Ok, Some(r#"W/"MAGIC""#.to_string())),
        get(&client, "/weak", Some(r#"W/"OTHER""#))
    );
}

// Weak comparison ignores the weakness indicator, strong comparison does not.
#[test]
fn the_two_comparison_functions_treat_the_weakness_indicator_differently() {
    let client = Client::tracked(rocket_with_routes()).unwrap();

    assert_eq!((Status::NotModified, None), get(&client, "/weak", Some(r#""MAGIC""#)));
    assert_eq!((Status::NotModified, None), get(&client, "/strong", Some(r#""MAGIC""#)));

    assert_eq!(
        (Status::Ok, Some(r#""MAGIC""#.to_string())),
        get(&client, "/strong", Some(r#"W/"MAGIC""#))
    );
}

// The header may hold a list, and any of its members is enough to match.
#[test]
fn a_list_matches_when_any_of_its_members_does() {
    let client = Client::tracked(rocket_with_routes()).unwrap();

    assert_eq!(
        (Status::NotModified, None),
        get(&client, "/weak", Some(r#""OTHER", W/"MAGIC", "ANOTHER""#))
    );

    assert_eq!(
        (Status::Ok, Some(r#"W/"MAGIC""#.to_string())),
        get(&client, "/weak", Some(r#""OTHER", "ANOTHER""#))
    );
}

// `*` matches whatever representation the server currently has.
#[test]
fn an_asterisk_matches_the_current_representation() {
    let client = Client::tracked(rocket_with_routes()).unwrap();

    assert_eq!((Status::NotModified, None), get(&client, "/weak", Some("*")));
    assert_eq!((Status::NotModified, None), get(&client, "/strong", Some("*")));
}

// The borrowed guard answers the same request the same way, out of one cached value.
#[test]
fn the_borrowed_guard_answers_the_same_way() {
    let client = Client::tracked(rocket_with_routes()).unwrap();

    assert_eq!((Status::NotModified, None), get(&client, "/borrowed", Some(r#""MAGIC""#)));
    assert_eq!(
        (Status::Ok, Some(r#"W/"MAGIC""#.to_string())),
        get(&client, "/borrowed", Some(r#""OTHER""#))
    );

    assert_eq!("true", client.get("/cached").dispatch().into_string().unwrap());
}
