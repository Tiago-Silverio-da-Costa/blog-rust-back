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
    pub async fn select_post() -> impl IntoResponse {
        let query = "SELECT * FROM posts";

        match HelperMySql::execute_select(query).await {
            Ok(results) => {
                let posts: Vec<Post> = results
                    .iter()
                    .map(|row| Post {
                        id: row.get("id"),
                        author_id: row.get("author_id"),
                        category_id: row.get("category_id"),
                        title: row.get("title"),
                        description: row.get("description"),
                        publication_date: row.get("publication_date"),
                        post_image_url: row.get("post_image_url"),
                        content: row.get("content"),
                        created_at: row.get("created_at"),
                        updated_at: row.get("updated_at"),
                    })
                    .collect();

                (
                    StatusCode::OK,
                    Json(json!({
                        "status": true,
                        "message": "Posts retrieved successfully",
                        "data": posts
                    })),
                )
                    .into_response()
            }
            Err(_e) => ApiError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "Error fetching posts".to_string(),
            }
            .into_response(),
        }
    }
}
