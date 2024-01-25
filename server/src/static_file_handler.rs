use std::{ffi::OsStr, sync::Arc};

use axum::{
    extract::{Path, State},
    http::{
        header::{CONTENT_ENCODING, CONTENT_TYPE},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::IntoResponse,
    routing::get, Router,
};
use axum_cc::{CacheControlLayer, MimeType};
use minijinja::Environment;
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


pub fn import_templates() -> anyhow::Result<Environment<'static>> {
    let mut env = Environment::new();

    for entry in std::fs::read_dir("server/templates")?.filter_map(Result::ok) {
        let path = entry.path();

        if path.is_file() && path.extension() == Some(OsStr::new("html")) {
            let name = path
                .file_name()
                .and_then(OsStr::to_str)
                .ok_or_else(|| anyhow::anyhow!("failed to get filename"))?
                .to_owned();

            let data = std::fs::read_to_string(&path)?;

            env.add_template_owned(name, data)?;
        }
    }

    Ok(env)
}