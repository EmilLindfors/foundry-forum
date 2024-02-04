#[derive(Debug)]
pub enum DbError {
    NotFound,
    UserNotFound,
    PasswordIncorrect,
    Sqlx(sqlx::Error),
    Other(anyhow::Error),
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::NotFound => write!(f, "Not found"),
            DbError::Sqlx(err) => write!(f, "Sqlx error: {}", err),
            DbError::Other(err) => write!(f, "Other error: {}", err),
            DbError::UserNotFound => write!(f, "User not found"),
            DbError::PasswordIncorrect => write!(f, "Password incorrect"),
        }
    }
}

impl std::error::Error for DbError {}

impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        Self::Sqlx(err)
    }
}
