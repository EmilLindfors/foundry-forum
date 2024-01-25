use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Form,
};
use axum_htmx::HxBoosted;
use axum_login::AuthSession;
use axum_messages::Messages;
use minijinja::context;
use serde::Deserialize;

use crate::{
    auth::{Backend, Credentials},
    AppState,
};

// This allows us to extract the "next" field from the query string. We use this
// to redirect after log in.
#[derive(Debug, Deserialize)]
pub struct NextUrl {
    next: Option<String>,
}

pub async fn login(
    boosted: HxBoosted,
    state: State<Arc<AppState>>,
    Query(NextUrl { next }): Query<NextUrl>,
) -> impl IntoResponse {
    state.render_with_context(
        boosted,
        "login.html",
        context! {
            next
        },
    )
}

pub async fn logout(mut auth_session: AuthSession<Backend>, messages: Messages) -> impl IntoResponse {
    match auth_session.logout().await {
        Ok(_) => {
            messages.success("Successfully logged out.".to_string());
            Redirect::to("/").into_response()},
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn register(
    boosted: HxBoosted,
    state: State<Arc<AppState>>,
) -> impl IntoResponse {
    state.render(
        boosted,
        "register.html"
    )
}

pub async fn post_login(
    boosted: HxBoosted,
    state: State<Arc<AppState>>,
    messages: Messages,
    mut auth_session: AuthSession<Backend>,
    Form(creds): Form<Credentials>,
) -> impl IntoResponse {
    let user = match auth_session.authenticate(creds.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return state
                .render_with_context(
                    boosted,
                    "login.html",
                    context! {
                        message => Some("Invalid credentials.".to_string()),
                        next => creds.next,
                    },
                )
                .into_response()
        }
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    messages.success(format!("Successfully logged in as {}", user.0.username));

    if let Some(ref next) = creds.next {
        Redirect::to(next).into_response()
    } else {
        Redirect::to("/").into_response()
    }
}

pub async fn index(boosted: HxBoosted, state: State<Arc<AppState>>, messages: Messages) -> impl IntoResponse {

    state.render_with_context(boosted, "index.html", context! {
        message => messages
        .into_iter()
        .map(|message| format!("{}: {}", message.level, message))
        .collect::<Vec<_>>()
        .join(", "),
    })
}
