use std::{sync::Arc, vec};

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
    auth::{self, Backend, Credentials},
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
        Err(e) => {
            match e {
                axum_login::Error::Backend(e) => {
                    match e {
                        db::error::DbError::UserNotFound => {
                            return state
                                .render_with_context(
                                    boosted,
                                    "login.html",
                                    context! {
                                        username_error => Some("Username does not exist".to_string()),
                                        next => creds.next,
                                    },
                                )
                                .into_response()
                        }
                        db::error::DbError::PasswordIncorrect => {
                            return state
                                .render_with_context(
                                    boosted,
                                    "login.html",
                                    context! {
                                        password_error => Some("Invalid password".to_string()),
                                        next => creds.next,
                                    },
                                )
                                .into_response()
                        }
                        _ => {
                            return StatusCode::INTERNAL_SERVER_ERROR.into_response()
                        }
                    }
                }
                axum_login::Error::Session(_) => {
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }

        }
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

pub async fn index(auth_session: AuthSession<Backend>, boosted: HxBoosted, state: State<Arc<AppState>>, messages: Messages) -> impl IntoResponse {

    let mut success_messages = vec![];
    let mut error_messages = vec![];
    let mut debug_messages = vec![];
    let mut info_messages = vec![];
    let mut warn_messages = vec![];


    messages
    .into_iter()
    .for_each(|message| match message.level {
        axum_messages::Level::Success => success_messages.push(message),
        axum_messages::Level::Error => error_messages.push(message),
        axum_messages::Level::Debug => debug_messages.push(message),
        axum_messages::Level::Info => info_messages.push(message),
        axum_messages::Level::Warning => warn_messages.push(message),
    });


    state.render_with_context(boosted, "index.html", context! {
        user => auth_session.user.map(|user| user.0),
        success_messages,
        error_messages,
        debug_messages,
        info_messages,
        warn_messages,

    })
}


pub async fn about(auth_session: AuthSession<Backend>, boosted: HxBoosted, state: State<Arc<AppState>>) -> impl IntoResponse {
    return state.render_with_editor(
        boosted,
        "about.html",
        context! {
            user => auth_session.user.map(|user| user.0),
        },
    )
}