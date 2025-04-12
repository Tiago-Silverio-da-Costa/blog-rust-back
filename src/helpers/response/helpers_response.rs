use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use serde_json::json;
pub struct HelpersResponse;

impl HelpersResponse {
    pub fn success<T: Serialize>(message: &str, results: T) -> Response {
        (
            StatusCode::OK,
            Json(json!({
                "code": "SUCCESS",
                "type": "success",
                "message": message,
                "results": results,
            })),
        )
            .into_response()
    }

    pub fn error(message: &str) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": "ERROR",
                "type": "error",
                "message": message
            })),
        )
            .into_response()
    }

    // Função para resposta de erro com detalhes adicionais
    pub fn error_with_detail<E: std::fmt::Display>(message: &str, err: E) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": "ERROR",
                "type": "error",
                "message": message,
                "error": err.to_string()
            })),
        )
            .into_response()
    }
}
