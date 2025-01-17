use axum:: {
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{Response, IntoResponse},
    Json
};

use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, Header, DecodingKey, EncodingKey, Validation};
use serde_json::json;
use chrono::{Duration, Utc};
use crate::mvc::models::user::user_model::UserRequest;
use crate::helpers::response::helpers_response::HelpersResponse;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: Usize,
}

pub struct HelperMiddlewareToken {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl HelperMiddlewareToken {

    
    pub fn new() -> Self {
        dotenv().ok();
        let token_sign_secret = std::env::var("TOKEN_SIGN_SECRET").expect("TOKEN_SIGN_SECRET must set");
        let secret: &[u8; 17] = token_sign_secret;

        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }
   
}