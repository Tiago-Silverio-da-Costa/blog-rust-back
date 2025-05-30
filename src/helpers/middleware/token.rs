use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use sqlx::Row;

use crate::helpers::{db::helpers_mysql::HelperMySql, response::helpers_response::HelpersResponse};
use crate::mvc::models::user::model_user::LoginRequest;
use chrono::{Duration, Utc};
use dotenv::dotenv;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub user_id: i32,
    pub exp: usize,
    pub iat: usize,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaimsFG {
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

    pub async fn create_token(&self, user: &LoginRequest, user_id: i32) -> Response {
        let query = "SELECT role FROM users WHERE email = ?";
        let params = vec![user.user.email.clone()];
        let role = match HelperMySql::execute_query_with_params(query, params).await {
            Ok(rows) => rows
                .get(0)
                .and_then(|row| row.try_get("role").ok())
                .unwrap_or("user".to_string()),
            Err(_) => "user".to_string(),
        };

        let now = Utc::now();
        let exp = (now + Duration::hours(24)).timestamp() as usize;
        let iat = now.timestamp() as usize;
        let claims = Claims {
            sub: user.user.email.clone(),
            role,
            user_id,
            exp,
            iat,
        };

        match encode(&Header::default(), &claims, &self.encoding_key) {
            Ok(token) => {
                let results = json!({
                    "token": token,
                    "user_id": user_id,
                });
                HelpersResponse::success("Login bem-sucedido", results)
            }
            Err(_) => (HelpersResponse::error("Erro ao gerar token")).into_response(),
        }
    }

    pub async fn create_token_fg(
        &self,
        email: String,
    ) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
        let now = Utc::now();
        let exp = (now + Duration::minutes(5)).timestamp() as usize;
        let iat = now.timestamp() as usize;

        let claims = ClaimsFG {
            sub: email.clone(),
            exp,
            iat,
        };

        match encode(&Header::default(), &claims, &self.encoding_key) {
            Ok(token) => Ok(token),
            Err(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": false, "message": "Erro ao gerar token" })),
            )),
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
                        if req.uri().path().starts_with("/post")
                            && token_data.claims.role != "admin"
                        {
                            return (
                                StatusCode::FORBIDDEN,
                                Json(json!({ "message": "Acesso negado: apenas administradores"})),
                            )
                                .into_response();
                        }
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
