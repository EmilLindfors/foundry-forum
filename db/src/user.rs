use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct DbUser {
    pub id: i64,
    pub username: String,
    pub password: String,
}

// Here we've implemented `Debug` manually to avoid accidentally logging the
// password hash.
impl std::fmt::Debug for DbUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("password", &"[redacted]")
            .finish()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, FromRow)]
pub struct DbPermission {
    pub name: String,
}

impl From<&str> for DbPermission {
    fn from(name: &str) -> Self {
        DbPermission {
            name: name.to_string(),
        }
    }
}
