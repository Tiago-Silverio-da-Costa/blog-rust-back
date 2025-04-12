use crate::helpers::db::helpers_mysql::HelperMySql;
use crate::helpers::middleware::token::{Claims, HelperMiddlewareToken};
use crate::helpers::response::helpers_response::HelpersResponse;
use crate::mvc::models::user::model_user::{
    CodeEmailPayload, EmailPayload, LoginRequest, ModelUser, UpdatePasswordPayload,
    UserRequestRegister,
};
use crate::mvc::services::user::email::services_user_email::ServicesUserEmail;
use axum::extract::Extension;
use axum::{http::StatusCode, response::IntoResponse, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use rand::distributions::Uniform;
use rand::seq::SliceRandom;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde_json::{json, Value};

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
    pub async fn get_me(
        Extension(claims): Extension<Claims>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        Ok(ModelUser::session_user(claims).await)
    }

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

        let hashed_code = hash(&code, DEFAULT_COST).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": false, "message": "Erro ao gerar hash do código" })),
            )
        })?;

        let expiration = Utc::now() + Duration::minutes(15);

        ModelUser::update_user_code(user.id, &hashed_code, &expiration).await?;

        Ok(ServicesUserEmail::send_code(&payload.email, &code).await)
    }

    pub async fn fg_check_code(
        Json(payload): Json<CodeEmailPayload>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let user = ModelUser::get_user_by_email(&payload.email).await?;

        if let (Some(stored_code), Some(expiration)) = (user.code, user.code_expiration) {
            let sao_paulo_offset = Duration::hours(-3);
            let now_sao_paulo = Utc::now() + sao_paulo_offset;

            if now_sao_paulo > expiration {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "status": false, "message": "Código expirado" })),
                ));
            }
            if verify(&payload.code, &stored_code).unwrap_or(false) {
                ModelUser::clear_code(user.id).await?;

                let token_result = HelperMiddlewareToken::new()
                    .create_token_fg(user.email)
                    .await;

                match token_result {
                    Ok(token) => {
                        Ok(HelpersResponse::success("Código válido", &token).into_response())
                    }
                    Err(err) => Err(err),
                }
            } else {
                Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "status": false, "message": "Código inválido" })),
                ))
            }
        } else {
            Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "status": false, "message": "Nenhum código encontrado" })),
            ))
        }
    }

    pub async fn fg_update_user_password(
        Extension(claims): Extension<Claims>,
        Json(payload): Json<UpdatePasswordPayload>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let user_email = claims.sub;
        let hashed_password = hash(&payload.password, DEFAULT_COST).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": false, "message": "Erro ao gerar hash da senha" })),
            )
        })?;

        let query = "UPDATE users SET password = ? WHERE email = ?";
        let params = vec![hashed_password, user_email];

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(_) => Ok(Json(
                json!({ "status": true, "message": "Senha atualizada com sucesso" }),
            )),
            Err(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": false, "message": "Erro ao atualizar senha" })),
            )),
        }
    }
}
