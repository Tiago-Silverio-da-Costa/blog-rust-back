use serde_json::{json, value};
use serder::{Deserialize, Serialize};

use axum::{
    extract::Json,
    http::StatusCode,
    resonse::{IntoResponse, Response},
};


use crate::{

    helpers::db::helper_mysql::HelperMysql,
    helpers::response::helpers_response::HelpersResponse
};

pub struct ModelUser;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRequest {
    pub user: User
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub bio: String,
    pub photo_url: String,
    pub role: String,
    pub email: String,
    pub password: String
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
            }))
        ).into_response()
    }
}

impl ModelUser{
    pub async fn insert_user(data: Json<UserRequest>) -> impl IntoResponse {
        let query = "INSERT INTO users (name, email, password) VALUES (?, ?,  ?)";
        match sqlx::query(query)
            .bind(&payload.name)
            .bind(&payload.email)
            .bind(&hashed_password)
            .execute(pool)
            .await
        {
            Ok(_) => {
                println!("Usuário inserido com sucesso.");
                Ok((
                    StatusCode::CREATED,
                    Json(serde_json::json!({"message": "User registered successfully"})),
                ))
            }
            Err(err) => {
                println!("Erro ao inserir usuário: {:?}", err);
                if let sqlx::Error::Database(db_err) = &err {
                    if db_err.constraint().unwrap_or("") == "users_email_unique" {
                        println!("Conflito: email já registrado.");
                        return Err(StatusCode::CONFLICT);
                    }
                }
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}