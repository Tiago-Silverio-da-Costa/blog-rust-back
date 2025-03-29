use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use serde_json::json;

use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

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
}

#[derive(Deserialize)]
pub struct EmailPayload {
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
        let query = "SELECT password from users WHERE email = ?";
        let params = vec![data.user.email.clone()];

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(rows) => {
                if let Some(row) = rows.get(0) {
                    let hashed_password: String = row.try_get("password").unwrap_or_default();

                    if verify(&data.user.password, &hashed_password).unwrap_or(false) {
                        let auth: HelperMiddlewareToken = HelperMiddlewareToken::new();
                        return auth.create_token(data).await;
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
        let query = "SELECT id, email FROM users WHERE email = ?";
        let params = vec![email];

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(rows) => {
                if let Some(row) = rows.get(0) {
                    let user = UserCode {
                        id: row.try_get("id").unwrap_or(0),
                        email: row.try_get("email").unwrap_or_default(),
                        code: row.try_get("code").ok(),
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
        code: &str,
    ) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
        let query = "UPDATE users SET code = ? WHERE id = ?";
        let params = vec![code.to_string(), user_id.to_string()];

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(_) => Ok(()),
            Err(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": false, "message": "Erro ao atualizar código" })),
            )),
        }
    }
}
