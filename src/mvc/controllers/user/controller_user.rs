use crate::mvc::models::user::model_user::{
    EmailPayload, LoginRequest, ModelUser, UserRequestRegister,
};
use crate::mvc::services::user::email::services_user_email::ServicesUserEmail;
use axum::{http::StatusCode, response::IntoResponse, Json};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::Value;

pub struct ControllerUser;

fn generate_code() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .map(char::from)
        .collect()
}

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

    pub async fn fg_send_code_to_email(
        Json(payload): Json<EmailPayload>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        // verify_email_already_exists
        ModelUser::fg_verify_email_already_exists(&payload.email).await?;

        let code = generate_code();
        println!("this is the code {:?}", code);
        //send email service lettre
        Ok(ServicesUserEmail::send_code(&payload.email, &code).await)
    }
}
