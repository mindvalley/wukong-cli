use reqwest::header;

pub mod pipeline;

pub(crate) fn auth_headers() -> header::HeaderMap {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_static("Bearer valid"),
    );

    headers
}
