use serde::{Deserialize, Serialize};
use serde_json::{json};

use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use sqlx::{mysql::MySqlRow, Row};

use crate::{
    helpers::db::helpers_mysql::HelperMySql, helpers::response::helpers_response::HelpersResponse,
};

pub struct ModelUser;

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
    pub async fn insert_user(data: Json<UserRequest>) -> impl IntoResponse {
        let query: String = format!("INSERT INTO users (name, email, password) VALUES (?, ?,  ?)");

        match HelperMySql::execute_query(query).await {
            Ok(_) => (StatusCode::CREATED, data).into_response(),
            Err(_e) => {
                return HelpersResponse::error("Erro ao inserir usuário").into_response();
            }
        }
    }

    pub async fn verify_email_already_exists(
        email: &str,
    ) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
        let query = "SELECT COUNT(*) as count FROM users WHERE email = ?";

        let params = vec![email];

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(row) => {
                // Extrair o valor do campo "count"
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
}
