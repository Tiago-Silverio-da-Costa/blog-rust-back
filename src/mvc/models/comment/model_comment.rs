use chrono::{DateTime, Duration, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use sqlx::Row;

use crate::helpers::db::helpers_mysql::HelperMySql;

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentRequest {
    pub comment: CommentRequestSchema,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentRequestSchema {
    post_id: i32,
    user_id: i32,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    id: i32,
    post_id: i32,
    user_id: i32,
    content: String,
    is_deleted: bool,
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

pub struct ModelComment;

impl ModelComment {
    pub async fn insert_comment(new_comment: CommentRequest) -> Result<(), ApiError> {
        let sao_paulo_offset = Duration::hours(-3);
        let now_sao_paulo = Utc::now() + sao_paulo_offset;

        let query = r#"
        INSERT INTO comments (post_id, user_id, content, is_deleted, created_at, updated_at) 
        VALUES (?, ?, ?, ?, ?, ?)
        "#;
        
        let params: Vec<String> = vec![
            new_comment.comment.post_id.to_string(),
            new_comment.comment.user_id.to_string(),
            new_comment.comment.content.clone(),
            0.to_string(),
            now_sao_paulo.to_rfc3339(),
            now_sao_paulo.to_rfc3339(),
        ];

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(_) => Ok(()),
            Err(err) => Err(ApiError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("Erro ao inserir comentário: {}", err),
            }),
        }
    }

    pub async fn select_comment_by_post(post_id: i32) -> Result<serde_json::Value, ApiError> {
        let query = r#"
        SELECT 
            u.id AS user_id,
            u.name AS user_name,
            c.*
        FROM 
            comments c
        LEFT JOIN 
            users u ON c.user_id = u.id
        WHERE 
            c.post_id = ? AND c.is_deleted = 0;
        "#;

        let params: Vec<i32> = vec![post_id];
        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(rows) => {
               let sao_paulo_offset = FixedOffset::west_opt(3 * 3600).expect("Offset inválido");

                let comments: Vec<serde_json::Value> = rows
                    .iter()
                    .map(|row| {

                    let created_at_utc: DateTime<Utc> = row.try_get::<DateTime<Utc>, _>("created_at").unwrap_or_else(|_| Utc::now());
                    let updated_at_utc: DateTime<Utc> = row.try_get::<DateTime<Utc>, _>("updated_at").unwrap_or_else(|_| Utc::now());
                    let created_at_sp = created_at_utc.with_timezone(&sao_paulo_offset);
                    let updated_at_sp = updated_at_utc.with_timezone(&sao_paulo_offset);
                        
                        json!({
                             "id": row.try_get::<i32, _>("id").unwrap_or_default(),
                            "post_id": row.try_get::<i32, _>("post_id").unwrap_or_default(),
                            "user_id": row.try_get::<i32, _>("user_id").unwrap_or_default(),
                            "user_name": row.try_get::<Option<String>, _>("user_name").unwrap_or(None),
                            "content": row.try_get::<String, _>("content").unwrap_or_default(),
                            "is_deleted": row.try_get::<bool, _>("is_deleted").unwrap_or(false),
                            "created_at": created_at_sp.to_rfc3339(),
                            "updated_at": updated_at_sp.to_rfc3339(),
                        })
                    })
                    .collect();

                Ok(json!({"comments": comments}))
            }
            Err(err) => Err(ApiError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("Erro ao buscar comentários: {}", err),
            }),
        }
    }
}
