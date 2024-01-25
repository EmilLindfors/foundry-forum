mod asset_cache;
mod auth;
mod base_template;
mod static_file_handler;

use asset_cache::{AssetCache, SharedAssetCache};
use axum::{serve, Router};
use axum_login::{
    login_required,
    tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use axum_messages::MessagesManagerLayer;
use base_template::{BaseTemplateData, SharedBaseTemplateData};
use db::DbPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_sessions_sqlx_store::SqliteStore;

use crate::auth::Backend;
use axum::{
    extract::{Path, State},
    http::{
        header::{ACCEPT, CONNECTION, CONTENT_ENCODING, CONTENT_TYPE},
        HeaderMap, HeaderName, HeaderValue, Method, StatusCode,
    },
    response::{sse::Event, Html, IntoResponse, Sse},
    routing::{delete, get},
    Extension,
};
use static_file_handler::static_file_handler;
use tower_http::{
    compression::{predicate::SizeAbove, CompressionLayer},
    cors::CorsLayer,
    CompressionLevel,
};

pub type BoxedError = Box<dyn std::error::Error>;

#[derive(Clone)]
pub struct AppState {
    db: db::DbPool,
    asset_cache: SharedAssetCache,
    base_template_data: SharedBaseTemplateData,
}

impl AppState {
    pub fn get_db(&self) -> db::DbPool {
        self.db.clone()
    }
}

pub struct Server {
    pub session_store: SqliteStore,
    pub listener: TcpListener,
    pub state: Arc<AppState>,
}

impl Server {
    pub async fn new(listener: &str) -> anyhow::Result<Self> {
        let db = DbPool::connect("sqlite:db.sqlite3").await?;
        db::migrate(db.acquire().await.unwrap()).await.unwrap();

        let session_store = SqliteStore::new(db.clone());
        session_store.migrate().await?;
        let asset_cache = AssetCache::load_static().await;
        let listener = TcpListener::bind(listener).await?;
        let state = AppState {
            db: db,
            asset_cache,
            base_template_data: BaseTemplateData::load_static(asset_cache),
        };

        Ok(Self {
            session_store,
            listener,
            state: Arc::new(state),
        })
    }

    pub fn get_delete_task(
        &self,
    ) -> tokio::task::JoinHandle<Result<(), tower_sessions::session_store::Error>> {
        tokio::task::spawn(
            self.session_store
                .clone()
                .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
        )
    }

    pub fn serve(self) -> tokio::task::JoinHandle<Result<(), std::io::Error>> {
        tracing::debug!("listening on {}", self.listener.local_addr().unwrap());

        let session_layer = SessionManagerLayer::new(self.session_store.clone())
            .with_secure(false)
            .with_expiry(Expiry::OnInactivity(time::Duration::hours(1)));

        let backend = Backend::new(self.state.get_db());
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        let router = Router::new().nest("/assets", static_file_handler(self.state));

        tokio::spawn(async move {
            serve(
                self.listener,
                router
                    .layer(MessagesManagerLayer)
                    .layer(auth_layer)
                    .layer(
                        CorsLayer::new()
                            .allow_credentials(true)
                            .allow_headers([
                                ACCEPT,
                                CONTENT_TYPE,
                                CONNECTION,
                                HeaderName::from_static("csrf-token"),
                            ])
                            .max_age(Duration::from_secs(86400))
                            //.allow_origin(config.cors_origin)
                            .allow_methods([
                                Method::GET,
                                Method::POST,
                                Method::PUT,
                                Method::DELETE,
                                Method::OPTIONS,
                                Method::HEAD,
                                Method::PATCH,
                                Method::CONNECT,
                            ]),
                    )
                    .layer(
                        CompressionLayer::new()
                            .quality(CompressionLevel::Precise(4))
                            .compress_when(SizeAbove::new(512)),
                    )
                    .into_make_service(),
            )
            .await
        })
    }
}
