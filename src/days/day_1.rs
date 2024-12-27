use salvo::{
    http::{header::LOCATION, HeaderMap},
    prelude::*,
};

#[handler]
async fn seek(res: &mut Response) {
    res.status_code(StatusCode::FOUND);
    let mut headers = HeaderMap::new();
    headers.insert(
        LOCATION,
        "https://www.youtube.com/watch?v=9Gc4QTqslN4"
            .parse()
            .unwrap(),
    );
    res.set_headers(headers);
}

pub fn get_router() -> Router {
    Router::with_path("/-1/seek").get(seek)
}
