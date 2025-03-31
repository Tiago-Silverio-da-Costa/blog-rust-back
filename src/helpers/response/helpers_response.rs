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
}
