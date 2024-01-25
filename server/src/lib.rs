mod api_error;
mod asset_cache;
mod auth;
mod base_template;
mod routes;
mod static_file_handler;
use crate::{
    auth::Backend,
    routes::{index, login, logout, post_login, register},
};
use api_error::ApiError;
use asset_cache::{AssetCache, SharedAssetCache};
use axum::{http::{
    header::{ACCEPT, CONNECTION, CONTENT_TYPE},
    HeaderName, Method,
}, routing::post};
use axum::{response::Html, routing::get, serve, Router};
use axum_cc::CacheControlLayer;
use axum_htmx::HxBoosted;
use axum_login::{
    tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use axum_messages::MessagesManagerLayer;
use base_template::{BaseTemplateData, SharedBaseTemplateData};
use db::DbPool;
use minijinja::{context, Value};
use static_file_handler::{import_templates, static_file_handler};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::{
    compression::{predicate::SizeAbove, CompressionLayer},
    cors::CorsLayer,
    CompressionLevel,
};
use tower_sessions_sqlx_store::SqliteStore;

pub type BoxedError = Box<dyn std::error::Error>;

#[derive(Clone)]
pub struct AppState {
    db: db::DbPool,
    pub env: minijinja::Environment<'static>,
    asset_cache: SharedAssetCache,
    base_template_data: SharedBaseTemplateData,
}

impl AppState {
    pub fn get_db(&self) -> db::DbPool {
        self.db.clone()
    }

    pub fn render(
        &self,
        HxBoosted(boosted): HxBoosted,
        template: &str,
    ) -> Result<Html<String>, ApiError> {
        let template = self
            .env
            .get_template(template)
            .map_err(|_| ApiError::TemplateNotFound(template.into()))?;

        if boosted {
            match template.render(context! {}) {
                Ok(rendered) => return Ok(Html(rendered)),
                Err(_) => return Err(ApiError::TemplateRender(template.name().into())),
            }
        }

        match template.render(context! {
            base => Some(self.base_template_data )
        }) {
            Ok(rendered) => Ok(Html(rendered)),
            Err(_) => Err(ApiError::TemplateRender(template.name().into())),
        }
    }

    pub fn render_with_context(
        &self,
        HxBoosted(boosted): HxBoosted,
        template: &str,
        ctx: Value,
    ) -> Result<Html<String>, ApiError> {
        let template = self
            .env
            .get_template(template)
            .map_err(|_| ApiError::TemplateNotFound(template.into()))?;

        if boosted {
            let rendered = template
                .render(ctx)
                .map_err(|_| ApiError::TemplateRender(template.name().into()))?;

            return Ok(Html(rendered));
        }

        match template.render(context! {
            base => Some(self.base_template_data), ..ctx
        }) {
            Ok(rendered) => Ok(Html(rendered)),
            Err(_) => Err(ApiError::TemplateRender(template.name().into())),
        }
    }
}

pub struct Server {
    pub session_store: SqliteStore,
    pub listener: TcpListener,
    pub state: Arc<AppState>,
}

impl Server {
    pub async fn new(listener: &str) -> anyhow::Result<Self> {
        let db = db::pool().await?;
        db::migrate(db.acquire().await.unwrap()).await.unwrap();

        let session_store = SqliteStore::new(db.clone());
        session_store.migrate().await?;
        let asset_cache = AssetCache::load_static().await;
        let listener = TcpListener::bind(listener).await?;
        let env = import_templates()?;
        let state = AppState {
            db,
            env,
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

        let main_router = Router::new()
            .route("/", get(index))
            .route("/login", get(login).post(post_login))
            .route("/logout", get(logout))
            .route("/register", get(register))
            .layer(MessagesManagerLayer)
            .layer(auth_layer)
            .with_state(self.state.clone())
            .layer(CacheControlLayer::new());

        let router = Router::new()
            .merge(main_router)
            .nest("/assets", static_file_handler(self.state));

        tokio::spawn(async move {
            serve(
                self.listener,
                router
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
