#[derive(Debug)]
pub enum DbError {
    NotFound,
    Sqlx(sqlx::Error),
    Other(anyhow::Error),
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::NotFound => write!(f, "Not found"),
            DbError::Sqlx(err) => write!(f, "Sqlx error: {}", err),
            DbError::Other(err) => write!(f, "Other error: {}", err),
        }
    }
}

impl std::error::Error for DbError {}

impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        Self::Sqlx(err)
    }
}
