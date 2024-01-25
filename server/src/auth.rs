use std::collections::HashSet;

use axum_login::{AuthUser, AuthnBackend, AuthzBackend, UserId};
use db::{
    error::DbError,
    sqlx,
    user::{DbPermission, DbUser},
    SqlitePool,
};
use password_auth::verify_password;
use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct User(pub DbUser);

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.0.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.0.password.as_bytes() // We use the password hash as the auth
                                   // hash--what this means
                                   // is when the user changes their password the
                                   // auth session becomes invalid.
    }
}

impl AsRef<DbUser> for User {
    fn as_ref(&self) -> &DbUser {
        &self.0
    }
}

// This allows us to extract the authentication fields from forms. We use this
// to authenticate requests with the backend.
#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub next: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Backend {
    db: SqlitePool,
}

impl Backend {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

#[axum::async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = DbError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user: Option<DbUser> = sqlx::query_as("select * from users where username = ? ")
            .bind(creds.username)
            .fetch_optional(&self.db)
            .await?;

        let res = user.filter(|user| {
            verify_password(creds.password, &user.password)
                .ok()
                .is_some() // We're using password-based authentication--this
                           // works by comparing our form input with an argon2
                           // password hash.
        });

        Ok(res.map(User))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user: Option<DbUser> = sqlx::query_as("select * from users where id = ?")
            .bind(user_id)
            .fetch_optional(&self.db)
            .await?;

        Ok(user.map(User))
    }
}

#[axum::async_trait]
impl AuthzBackend for Backend {
    type Permission = DbPermission;

    async fn get_group_permissions(
        &self,
        user: &Self::User,
    ) -> Result<HashSet<Self::Permission>, Self::Error> {
        let permissions: Vec<Self::Permission> = sqlx::query_as(
            r#"
            select distinct permissions.name
            from users
            join users_groups on users.id = users_groups.user_id
            join groups_permissions on users_groups.group_id = groups_permissions.group_id
            join permissions on groups_permissions.permission_id = permissions.id
            where users.id = ?
            "#,
        )
        .bind(user.0.id)
        .fetch_all(&self.db)
        .await?;

        Ok(permissions.into_iter().collect())
    }
}


