use crate::{
    helpers::middleware::token::Claims,
    mvc::models::comment::model_comment::{CommentRequest, ModelComment},
};
use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;

pub struct ControllerComment;

impl ControllerComment {
    pub async fn get_all_comments_by_post(Path(post_id): Path<i32>) -> impl IntoResponse {
        match ModelComment::select_comment_by_post(post_id).await {
            Ok(comments) => (
                StatusCode::OK,
                Json(json!({
                    "status": true,
                    "data": comments
                })),
            )
                .into_response(),
            Err(err) => err.into_response(),
        }
    }

    pub async fn post_new_comment(
        Extension(claims): Extension<Claims>,
        Json(new_comment): Json<CommentRequest>,
    ) -> impl IntoResponse {
        match ModelComment::insert_comment(new_comment, claims.user_id).await {
            Ok(_) => (
                StatusCode::CREATED,
                Json(json!({
                    "status": true,
                    "message": "comentário criado com sucesso",
                })),
            )
                .into_response(),
            Err(err) => println!("erwsdf {:?}", err).into_response(),
        }
    }
}
