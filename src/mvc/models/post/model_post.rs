use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use sqlx::Row;

use crate::helpers::{db::helpers_mysql::HelperMySql};

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
                            "author_id": row.try_get::<i32, _>("author_id").unwrap_or_default(),
                            "category_id": row.try_get::<i32, _>("category_id").unwrap_or_default(),
                            "description": row.try_get::<String, _>("description").unwrap_or_default(),
                            "publication_date": row.try_get::<NaiveDateTime, _>("publication_date").unwrap_or_default(),
                            "post_image_url": row.try_get::<String, _>("post_image_url").unwrap_or_default(),
                        })
                    })
                    .collect();
                Ok(posts)
            }
            Err(err) => Err(err),
        }
    }

    pub async fn select_post_by_id(post_id: i32) -> Result<serde_json::Value, ApiError> {
        let query = r#"
            SELECT 
                posts.*,
                categories.name AS category_name,
                users.name AS author_name
            FROM 
                posts
            LEFT JOIN 
                categories ON posts.category_id = categories.id
            LEFT JOIN 
                users ON posts.author_id = users.id
            WHERE 
                posts.id = ?
        "#;

        // Executa a consulta ao banco de dados com o ID como parâmetro
        let params: Vec<i32> = vec![post_id.into()];
        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(rows) => {
                // Verifica se houve retorno
                if rows.is_empty() {
                    return Err(ApiError {
                        status_code: StatusCode::NOT_FOUND,
                        message: "Post não encontrado".to_string(),
                    });
                }

                // Mapeia os dados do primeiro resultado para JSON
                let row = &rows[0];
                let post = json!({
                    "id": row.try_get::<i32, _>("id").unwrap_or_default(),
                    "author_id": row.try_get::<i32, _>("author_id").unwrap_or_default(),
                    "author_name": row.try_get::<String, _>("author_name").unwrap_or_default(),
                    "category_id": row.try_get::<i32, _>("category_id").unwrap_or_default(),
                    "category_name": row.try_get::<String, _>("category_name").unwrap_or_default(),
                    "title": row.try_get::<String, _>("title").unwrap_or_default(),
                    "description": row.try_get::<String, _>("description").unwrap_or_default(),
                    "publication_date": row.try_get::<NaiveDateTime, _>("publication_date").unwrap_or_default().format("%Y-%m-%d").to_string(),
                    "post_image_url": row.try_get::<Option<String>, _>("post_image_url").unwrap_or(None),
                    "content": row.try_get::<String, _>("content").unwrap_or_default(),
                    "created_at": row.try_get::<DateTime<Utc>, _>("created_at").unwrap_or_default(),
                    "updated_at": row.try_get::<DateTime<Utc>, _>("updated_at").unwrap_or_default(),
                });

                Ok(post)
            }
            Err(err) => {
                Err(ApiError {
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                    message: format!("Erro ao buscar o post: {}", err),
                })
            }
        }
    }
}