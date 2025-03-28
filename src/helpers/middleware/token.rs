use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::helpers::response::helpers_response::HelpersResponse;
use crate::mvc::models::user::model_user::User;
use chrono::{Duration, Utc};
use dotenv::dotenv;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub struct HelperMiddlewareToken {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl HelperMiddlewareToken {
    pub fn new() -> Self {
        dotenv().ok();
        let token_sign_secret =
            std::env::var("TOKEN_SIGN_SECRET").expect("TOKEN_SIGN_SECRET must set");
        let secret = token_sign_secret.as_bytes();

        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }

    pub async fn create_token(&self, user: &User) -> Response {
        let now = Utc::now();
        let exp = (now + Duration::hours(24)).timestamp() as usize;
        let iat = now.timestamp() as usize;

        let claims = Claims {
            sub: user.email.clone(),
            exp,
            iat,
        };

        match encode(&Header::default(), &claims, &self.encoding_key) {
            Ok(_token) => {
                let response_body = json!({
                    "message": "Login bem-sucedido",
                    "token": _token,
                    "userId": user.id
                });
                HelpersResponse::success("Query executada", json!(response_body)).into_response()
            }
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Erro ao gerar token"
                })),
            )
                .into_response(),
        }
    }
    pub async fn verify_token(&self, mut req: Request<Body>, next: Next) -> Response {
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .map(|s| s.trim_start_matches("Bearer "));

        match auth_header {
            Some(token) => {
                let validation = Validation::default();
                match decode::<Claims>(token, &self.decoding_key, &validation) {
                    Ok(token_data) => {
                        req.extensions_mut().insert(token_data.claims);
                        next.run(req).await
                    }
                    Err(_) => (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({ "message": "Token inválido" })),
                    )
                        .into_response(),
                }
            }
            None => (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "message": "Token não fornecido" })),
            )
                .into_response(),
        }
    }
}
