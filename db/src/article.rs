use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Row, Sqlite, SqliteConnection};

#[derive(Clone, Serialize, Deserialize, FromRow, Debug)]
pub struct Article {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub editor_content: serde_json::Value,
    pub content: String,
    //pub created_at: chrono::NaiveDateTime,
    //pub updated_at: chrono::NaiveDateTime,
}

impl Article {
    pub async fn new(
        user_id: i64,
        title: String,
        editor_content: serde_json::Value,
        content: String,
        pool: &Pool<Sqlite>,
    ) -> Result<Article, sqlx::Error> {
        let article = sqlx::query_as!(
            Article,
            r#"INSERT INTO articles (user_id, title, editor_content, content) VALUES (?, ?, ?, ?) RETURNING *"#,
            user_id,
            title,
            editor_content,
            content
        )
        .fetch_one(pool)
        .await?;
        Ok(article)
    }

    pub async fn upsert(
        user_id: i64,
        title: String,
        editor_content: serde_json::Value,
        content: String,
        pool: &Pool<Sqlite>,
    ) -> Result<Article, sqlx::Error> {
        let article = sqlx::query_as!(
            Article,
            r#"INSERT INTO articles (user_id, title, editor_content, content) VALUES (?, ?, ?, ?) ON CONFLICT (user_id, title) DO UPDATE SET editor_content = ?, content = ? RETURNING *"#,
            user_id,
            title,
            editor_content,
            content,
            editor_content,
            content
        )
        .fetch_one(pool)
        .await?;
        Ok(article)
    }

    pub async fn find_by_id(id: i64, pool: &Pool<Sqlite>) -> Result<Article, sqlx::Error> {
        let article = sqlx::query_as!(Article, r#"SELECT * FROM articles WHERE id = ?"#, id)
            .fetch_one(pool)
            .await?;
        Ok(article)
    }

    pub async fn find_all(pool: &Pool<Sqlite>) -> Result<Vec<Article>, sqlx::Error> {
        let articles = sqlx::query_as!(Article, r#"SELECT * FROM articles"#)
            .fetch_all(pool)
            .await?;
        Ok(articles)
    }

    pub async fn update(
        id: i64,
        title: String,
        editor_content: serde_json::Value,
        content: String,
        pool: &Pool<Sqlite>,
    ) -> Result<Article, sqlx::Error> {
        let article = sqlx::query_as!(
            Article,
            r#"UPDATE articles SET title = ?, editor_content = ?, content = ? WHERE id = ? RETURNING *"#,
            title,
            editor_content,
            content,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(article)
    }

    pub async fn delete(id: i64, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM articles WHERE id = ?", id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub fn from_row(row: sqlx::sqlite::SqliteRow) -> Self {
        Self {
            id: row.get(0),
            user_id: row.get(1),
            title: row.get(2),
            editor_content: row.get(3),
            content: row.get(4),
        }
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }
}
