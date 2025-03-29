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
            Ok(_) => {
                // Retorna sucesso com o usuário criado
                (
                    StatusCode::CREATED,
                    Json(json!({
                        "status": true,
                        "message": "Usuário criado com sucesso",
                    })),
                )
                    .into_response()
            }
            Err(_e) => {
                // Retorna erro genérico
                HelpersResponse::error("Erro ao inserir usuário").into_response()
            }
        }
    }

    pub async fn verify_email_already_exists(
        email: &str,
    ) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
        let query = "SELECT COUNT(*) as count FROM users WHERE email = ?";
        let params = vec![email];

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(rows) => {
                // Verifica se há alguma linha retornada
                if let Some(row) = rows.get(0) {
                    // Extrai o valor do campo "count"
                    let count: i64 = row.try_get("count").unwrap_or(0);

                    // Verifica se o email já existe
                    if count > 0 {
                        // Email já cadastrado
                        Err((
                            StatusCode::BAD_REQUEST,
                            Json(json!({
                                "status": false,
                                "message": "Email já cadastrado no sistema"
                            })),
                        ))
                    } else {
                        // Email não existe
                        Ok(())
                    }
                } else {
                    // Caso não haja nenhuma linha, considere como erro
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "status": false,
                            "message": "Erro inesperado na consulta"
                        })),
                    ))
                }
            }
            Err(_e) => {
                // Erro ao executar a consulta
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "status": false,
                        "message": "Erro ao verificar email"
                    })),
                ))
            }
        }
    }

    pub async fn fg_verify_email_already_exists(
        email: &str,
    ) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
        let query = "SELECT COUNT(*) as count FROM users WHERE email = ?";
        let params = vec![email];
        println!("params {:?}", params);

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(rows) => {
                if let Some(row) = rows.get(0) {
                    let count: i64 = row.try_get("count").unwrap_or(0);

                    // Se count for maior que 0, o e-mail existe, e retornamos Ok
                    if count > 0 {
                        Ok(print!("achou email"))
                    } else {
                        // Caso não exista, retorna um erro
                        Err((
                            StatusCode::BAD_REQUEST,
                            Json(json!({
                                "status": false,
                                "message": "Email não cadastrado no sistema"
                            })),
                        ))
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
}
