use crate::mvc::models::user::model_user::{
    EmailPayload, LoginRequest, ModelUser, UserRequestRegister,
};
use crate::mvc::services::user::email::services_user_email::ServicesUserEmail;
use axum::{http::StatusCode, response::IntoResponse, Json};
use rand::distributions::Uniform;
use rand::seq::SliceRandom;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde_json::Value;

pub struct ControllerUser;

fn generate_code() -> String {
    let mut rng = thread_rng();
    let mut code: Vec<char> = vec![];

    for _ in 0..2 {
        let num = rng.sample(Uniform::new_inclusive(0, 9));
        code.push(std::char::from_digit(num, 10).unwrap());
    }

    while code.len() < 5 {
        let letter = rng.sample(Alphanumeric) as char;
        if letter.is_ascii_alphabetic() {
            code.push(letter.to_ascii_uppercase());
        }
    }

    code.shuffle(&mut rng);

    code.into_iter().collect()
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
        let user = ModelUser::fg_verify_email_already_exists(&payload.email).await?;

        let code = generate_code();

        ModelUser::update_user_code(user.id, &code).await?;

        Ok(ServicesUserEmail::send_code(&payload.email, &code).await)
    }
}
