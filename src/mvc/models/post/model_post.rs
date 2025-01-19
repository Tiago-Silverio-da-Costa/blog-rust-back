use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use sqlx::Row;

use crate::helpers::db::helpers_mysql::HelperMySql;

pub struct ModelPost;

#[derive(Debug, Serialize, Deserialize)]
pub struct PostRequest {
    pub post: Post,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    id: i32,
    author_id: i32,
    category_id: i32,
    title: String,
    description: String,
    publication_date: NaiveDateTime,
    post_image_url: Option<String>,
    content: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

pub struct ApiError {
    status_code: StatusCode,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status_code,
            Json(json!({
                "status": false,
                "message": self.message
            })),
        )
            .into_response()
    }
}

impl ModelPost {
    pub async fn select_post() -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let query = "SELECT * FROM posts"; // Ajuste para a sua tabela
        match HelperMySql::execute_select(query).await {
            Ok(rows) => {
                // Convertendo linhas para JSON
                let posts: Vec<serde_json::Value> = rows
                    .into_iter()
                    .map(|row| {
                        json!({
                            "id": row.try_get::<i32, _>("id").unwrap_or_default(),
                            "title": row.try_get::<String, _>("title").unwrap_or_default(),
                            "content": row.try_get::<String, _>("content").unwrap_or_default(),
                        })
                    })
                    .collect();
                Ok(posts)
            }
            Err(err) => Err(err),
        }
    }
}