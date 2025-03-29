use crate::mvc::models::user::model_user::{LoginRequest, ModelUser, UserRequestRegister};
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::Value;

pub struct ControllerUser;

impl ControllerUser {
    pub async fn login(
        Json(data): Json<LoginRequest>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        Ok(ModelUser::auth_user(&data).await)
    }

    pub async fn register_user(
        data: Json<UserRequestRegister>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        ModelUser::verify_email_already_exists(&data.user.email).await?;

        Ok(ModelUser::insert_user(data).await)
    }
}
