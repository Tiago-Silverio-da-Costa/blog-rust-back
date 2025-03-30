use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;

use crate::helpers::{
    db::helpers_mysql::HelperMySql, middleware::token::HelperMiddlewareToken,
    response::helpers_response::HelpersResponse,
};

pub struct ModelUser;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserCode {
    pub id: i64,
    pub email: String,
    pub code: Option<String>,
    pub code_expiration: Option<chrono::DateTime<Utc>>,
}

#[derive(serde::Deserialize)]
pub struct UpdatePasswordPayload {
    // pub token: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct UserPassword {
    pub email: String,
    pub password: String,
}
#[derive(Deserialize)]
pub struct EmailPayload {
    pub email: String,
}

#[derive(Deserialize)]
pub struct CodeEmailPayload {
    pub code: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub user: UserRequestLoginSchema,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UserRequestLoginSchema {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRequestRegister {
    pub user: UserRequestRegisterSchema,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UserRequestRegisterSchema {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRequest {
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub bio: String,
    pub photo_url: String,
    pub role: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug)]
pub struct ApiError {
    status_code: StatusCode,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status_code,
            Json(json!({
                "status": false,
                "message": self.message
            })),
        )
            .into_response()
    }
}

impl ModelUser {
    pub async fn auth_user(data: &LoginRequest) -> impl IntoResponse {
        let query = "SELECT * from users WHERE email = ?";
        let params = vec![data.user.email.clone()];

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(rows) => {
                // if rows.is_empty() {
                //     return (HelpersResponse::error("Usuário não encontrado")).into_response();
                // }
                // for (i, row) in rows.iter().enumerate() {
                //     println!("Row {}:", i);
                //     for column in row.columns() {
                //         let column_name = column.name();
                //         let value: Option<&str> = row.try_get(column_name).ok();
                //         println!("  {}: {:?}", column_name, value);
                //     }
                // }
                if let Some(row) = rows.get(0) {
                    let hashed_password: String = row.try_get("password").unwrap_or_default();

                    if verify(&data.user.password, &hashed_password).unwrap_or(false) {
                        let auth: HelperMiddlewareToken = HelperMiddlewareToken::new();
                        let user_id: i32 = row.try_get("id").unwrap_or_default();
                        return auth.create_token(data, user_id).await;
                    } else {
                        (HelpersResponse::error("Credenciais inválidas")).into_response()
                    }
                } else {
                    (HelpersResponse::error("Usuário não encontrado")).into_response()
                }
            }

            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": false,
                    "message": "Erro ao buscar usuário"
                })),
            )
                .into_response(),
        }
    }

    pub async fn insert_user(data: Json<UserRequestRegister>) -> impl IntoResponse {
        let hashed_password = match hash(&data.user.password, DEFAULT_COST) {
            Ok(hp) => hp,
            Err(_) => return (HelpersResponse::error("Erro ao processar a senha")).into_response(),
        };

        let query = "INSERT INTO users (name, email, password) VALUES (?, ?, ?)";
        let params = vec![
            data.user.name.clone(),
            data.user.email.clone(),
            hashed_password.to_string(),
        ];

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(_) => (
                StatusCode::CREATED,
                Json(json!({
                    "status": true,
                    "message": "Usuário criado com sucesso",
                })),
            )
                .into_response(),
            Err(_e) => HelpersResponse::error("Erro ao inserir usuário").into_response(),
        }
    }

    pub async fn verify_email_already_exists(
        email: &str,
    ) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
        let query = "SELECT COUNT(*) as count FROM users WHERE email = ?";
        let params = vec![email];

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(rows) => {
                if let Some(row) = rows.get(0) {
                    let count: i64 = row.try_get("count").unwrap_or(0);

                    if count > 0 {
                        Err((
                            StatusCode::BAD_REQUEST,
                            Json(json!({
                                "status": false,
                                "message": "Email já cadastrado no sistema"
                            })),
                        ))
                    } else {
                        Ok(())
                    }
                } else {
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "status": false,
                            "message": "Erro inesperado na consulta"
                        })),
                    ))
                }
            }
            Err(_e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": false,
                    "message": "Erro ao verificar email"
                })),
            )),
        }
    }

    pub async fn fg_verify_email_already_exists(
        email: &str,
    ) -> Result<UserCode, (StatusCode, Json<serde_json::Value>)> {
        let query = "SELECT id, email, code, code_expiration FROM users WHERE email = ?";
        let params = vec![email];

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(rows) => {
                if let Some(row) = rows.get(0) {
                    let user = UserCode {
                        id: row.try_get("id").unwrap_or(0),
                        email: row.try_get("email").unwrap_or_default(),
                        code: row.try_get("code").ok(),
                        code_expiration: row.try_get("code_expiration").ok(),
                    };
                    Ok(user)
                } else {
                    Err((
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "status": false, "message": "Email não cadastrado" })),
                    ))
                }
            }
            Err(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": false, "message": "Erro ao verificar email" })),
            )),
        }
    }

    pub async fn update_user_code(
        user_id: i64,
        hashed_code: &str,
        expiration: &DateTime<Utc>,
    ) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
        let sao_paulo_offset = Duration::hours(-3);
        let expiration_sao_paulo = *expiration + sao_paulo_offset;
        let formatted_expiration = expiration_sao_paulo.format("%Y-%m-%d %H:%M:%S").to_string();

        let query = "UPDATE users SET code = ?, code_expiration = ? WHERE id = ?";
        let params = vec![
            hashed_code.to_string(),
            formatted_expiration,
            user_id.to_string(),
        ];

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(_) => Ok(()),
            Err(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": false, "message": "Erro ao atualizar código" })),
            )),
        }
    }

    pub async fn get_user_by_email(
        email: &str,
    ) -> Result<UserCode, (StatusCode, Json<serde_json::Value>)> {
        let query = "SELECT id, email, code, code_expiration FROM users WHERE email = ?";
        let params = vec![email.to_string()];

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(rows) => {
                if let Some(row) = rows.get(0) {
                    let user = UserCode {
                        id: row.try_get("id").unwrap_or(0),
                        email: row.try_get("email").unwrap_or_default(),
                        code: row.try_get("code").ok(),
                        code_expiration: row.try_get("code_expiration").ok(),
                    };
                    Ok(user)
                } else {
                    Err((
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "status": false, "message": "Email não cadastrado" })),
                    ))
                }
            }
            Err(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": false, "message": "Erro ao verificar email" })),
            )),
        }
    }

    pub async fn fg_update_user_password(data: Json<UserPassword>) -> impl IntoResponse {
        let hashed_password = match hash(&data.password, DEFAULT_COST) {
            Ok(hp) => hp,
            Err(_) => return (HelpersResponse::error("Erro ao processar a senha")).into_response(),
        };

        let query = "UPDATE users SET password = ? WHERE email = ?";
        let params = vec![hashed_password.to_string(), data.email.to_string()];

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(_) => (
                StatusCode::CREATED,
                Json(json!({
                    "status": true,
                    "message": "Senha atualiza com sucesso",
                })),
            )
                .into_response(),
            Err(_e) => HelpersResponse::error("Erro ao atualizar senha!").into_response(),
        }
    }

    pub async fn clear_code(user_id: i64) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
        let query = "UPDATE users SET code = NULL, code_expiration = NULL WHERE id = ?";
        let params = vec![user_id.to_string()];

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(_) => Ok(()),
            Err(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": false, "message": "Erro ao limpar código" })),
            )),
        }
    }
}
