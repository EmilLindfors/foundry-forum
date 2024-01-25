use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{
        header::{ACCEPT, CONNECTION, CONTENT_ENCODING, CONTENT_TYPE},
        HeaderMap, HeaderName, HeaderValue, Method, StatusCode,
    },
    response::{sse::Event, Html, IntoResponse, Sse},
    routing::{delete, get},
    Extension, Router,
};
use axum_cc::{CacheControlLayer, MimeType};

use crate::AppState;

pub fn static_file_handler(state: Arc<AppState>) -> Router {
    const PRECOMPRESSED_MIME_TYPES: &[MimeType; 2] = &[MimeType::CSS, MimeType::JS];

    Router::new()
        .route(
            "/:file",
            get(
                |state: State<Arc<AppState>>, path: Path<String>| async move {
                    let Some(asset) = state.asset_cache.get_from_path(&path) else {
                        return StatusCode::NOT_FOUND.into_response();
                    };

                    let mut headers = HeaderMap::new();

                    // We set the content type explicitly here as it will otherwise
                    // be inferred as an `octet-stream`
                    headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_static(asset.content_type.as_str()),
                    );

                    if PRECOMPRESSED_MIME_TYPES.contains(&asset.content_type) {
                        headers.insert(CONTENT_ENCODING, HeaderValue::from_static("br"));
                    }

                    // `bytes::Bytes` clones are cheap
                    (headers, asset.contents.clone()).into_response()
                },
            ),
        )
        .layer(CacheControlLayer::default())
        .with_state(state)
}
