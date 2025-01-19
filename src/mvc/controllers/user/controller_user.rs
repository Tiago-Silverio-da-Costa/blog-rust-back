use crate::{
    helpers::middleware::token::HelperMiddlewareToken,
    mvc::models::user::model_user::{ModelUser, LoginRequest, UserRequestRegister},
};
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::Value;

pub struct ControllerUser;

impl ControllerUser {
    pub async fn login(Json(data): Json<LoginRequest>) -> impl IntoResponse {
        let auth: HelperMiddlewareToken = HelperMiddlewareToken::new();

        auth.create_token(Json(data)).await
    }

    pub async fn register_user(
        data: Json<UserRequestRegister>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        ModelUser::verify_email_already_exists(&data.user.email).await?;

        Ok(ModelUser::insert_user(data).await)
    }
}