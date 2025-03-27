use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use serde_json::json;

use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use sqlx::Row;

use crate::{
    helpers::db::helpers_mysql::HelperMySql, helpers::response::helpers_response::HelpersResponse,
};

pub struct ModelUser;

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
    pub async fn insert_user(data: Json<UserRequestRegister>) -> impl IntoResponse {
        let hashed_password = match hash(&data.user.password, DEFAULT_COST) {
            Ok(hp) => hp,
            Err(_) => {
                return HelpersResponse::error("Erro ao gerar o hash da senha").into_response();
            }
        };

        let query = "INSERT INTO users (name, email, password) VALUES (?, ?, ?)";
        let params = vec![
            data.user.name.clone(),
            data.user.email.clone(),
            hashed_password,
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

        println!("teste emailasda {}", email);

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

    pub async fn validate_user(email: String, password: String) -> Result<User, ApiError> {
        let query = "SELECT * FROM users WHERE email = ?";
        let params = vec![email.clone()];

        println!("test emaikl {}, passwrod {}", email, password);

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(rows) => {
                if let Some(row) = rows.get(0) {
                    let hashed_password: String = row.try_get("password").unwrap_or_default();

                    if verify(password, &hashed_password).unwrap_or(false) {
                        let user = User {
                            id: row.try_get("id").unwrap_or_default(),
                            name: row.try_get("name").unwrap_or_default(),
                            bio: row.try_get("bio").unwrap_or_default(),
                            photo_url: row.try_get("photo_url").unwrap_or_default(),
                            role: row.try_get("role").unwrap_or_default(),
                            email: row.try_get("email").unwrap_or_default(),
                            password: row.try_get("password").unwrap_or_default(),
                        };
                        return Ok(user);
                    } else {
                        Err(ApiError {
                            status_code: StatusCode::UNAUTHORIZED,
                            message: "Email ou senha inválidos".to_string(),
                        })
                    }
                } else {
                    Err(ApiError {
                        status_code: StatusCode::UNAUTHORIZED,
                        message: "Email ou senha inválidos".to_string(),
                    })
                }
            }
            Err(_) => Err(ApiError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "Erro ao verificar credenciais".to_string(),
            }),
        }
    }
}
