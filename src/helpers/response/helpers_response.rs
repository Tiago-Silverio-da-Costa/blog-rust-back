use axum::{http::StatusCode, response::{IntoResponse,Json, Response}};
use serde_json::{json, Value};

pub struct HelpersResponse;

impl HelpersResponse {
    pub fn success(message: &str, results: Value) -> Response {
        (
            StatusCode::OK,
            Json(json!({
                "code": "SUCCESS",
                "type": "success",
                "message": message,
                "results": results,
            }))
        ).into_response()
    }

    pub fn error(message: &str) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "code": "ERROR",    
                "tyoe": "error",
                "message": message
            }))
        ).into_response()
    }
}