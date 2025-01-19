use crate::mvc::models::post::model_post::ModelPost;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

pub struct ControllerPost;

impl ControllerPost {
    pub async fn get_all_posts() -> impl IntoResponse {
        match ModelPost::select_post().await {
            Ok(posts) => {
                // Retornar os dados como JSON
                (
                    StatusCode::OK,
                    Json(json!({ "status": true, "data": posts })),
                )
            }
            Err(err) => {
                // Retornar erro genérico
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "status": false,
                        "message": format!("Erro ao buscar posts: {}", err)
                    })),
                )
            }
        }
    }
}
