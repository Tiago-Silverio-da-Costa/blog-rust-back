use crate::{
    helpers::middleware::token::HelperMiddlewareToken,
    mvc::models::user::model_user::{LoginRequest, ModelUser, UserRequestRegister},
};
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::Value;

pub struct ControllerUser;

impl ControllerUser {
    pub async fn login(Json(data): Json<LoginRequest>) -> impl IntoResponse {
        let auth: HelperMiddlewareToken = HelperMiddlewareToken::new();
        
        match ModelUser::validate_user(data.user.email, data.user.password).await {
            Ok(user_data) => auth.create_token(&user_data).await,
            Err(api_error) => api_error.into_response(),
        }
    }

    pub async fn register_user(
        data: Json<UserRequestRegister>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        ModelUser::verify_email_already_exists(&data.user.email).await?;

        Ok(ModelUser::insert_user(data).await)
    }
}
