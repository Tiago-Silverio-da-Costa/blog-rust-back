mod server;

pub mod helpers {
    pub mod db {
        pub mod helpers_mysql;
    }
    pub mod middleware {
        pub mod token;
    }
    pub mod response {
        pub mod helpers_response;
    }
}

pub mod mvc {
    pub mod models {
        pub mod user {
            pub mod model_user;
        }
    }

    pub mod controllers {
        pub mod user {
            pub mod controller_user;
        }
    }

    pub mod routes {
        pub mod user {
            pub mod route_user;
        }
    }
}

use crate::helpers::db::helpers_mysql::HelperMySql;

#[tokio::main]
async fn main() {
    let app: axum::Router = server::create_app().await;
    let listener: tokio::net::TcpListener =
        tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Servidor rodando em http://0.0.0.0:3000");

    match HelperMySql::init().await {
        Ok(_helper) => {
            println!("Conexão estabelecida com sucesso!")
        }
        Err(e) => {
            eprintln!("Erro ao conectar ao banco: {}", e)
        }
    };

    axum::serve(listener, app).await.unwrap();
}

use axum::{
    extract::State,
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::FromRow;
use sqlx::Row;
use std::sync::Arc;
use tokio::net::TcpListener;


#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Post {
    id: Option<i32>,
    author_id: i32,
    title: String,
    description: String,
    publication_date: NaiveDateTime,
    category_id: i32,
    post_image_url: Option<String>,
    content: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PostWithRelations {
    id: Option<i32>,
    title: String,
    description: String,
    publication_date: NaiveDateTime,
    category_name: String,
    author_name: String,
    post_image_url: Option<String>,
    content: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    id: Option<i32>,
    content: String,
    user_id: i32,
    post_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Author {
    id: i32,
    name: String,
    bio: Option<String>,
    photo_url: Option<String>,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    name: String,
    email: String,
    password: String,
}

pub struct LoginRequest {
    email: String,
    password: String,
}

pub struct AppState {
    pub pool: MySqlPool,
}


pub async fn health_check_handle() -> impl IntoResponse {
    const MESSAGE: &str = "API Services";

    let json_response = serde_json::json!({
        "status": "ok",
        "message": MESSAGE
    });

    Json(json_response)
}

pub async fn get_all_posts(State(state): axum::extract::State<Arc<AppState>>) -> impl IntoResponse {
    let posts = sqlx::query_as::<_, Post>("SELECT * FROM posts")
        .fetch_all(&state.pool)
        .await
        .unwrap();

    Json(posts)
}

pub async fn get_post_by_id(
    State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> impl IntoResponse {
    let result: Result<Option<PostWithRelations>, sqlx::Error> =
        sqlx::query_as::<_, PostWithRelations>(
            "
        SELECT 
            posts.*,
            categories.name AS category_name,
            users.name AS author_name
       FROM 
            posts
    LEFT JOIN 
            categories ON posts.category_id = categories.id
    LEFT JOIN 
            users ON posts.author_id = users.id
    WHERE 
            posts.id = ?",
        )
        .bind(id)
        .fetch_optional(&state.pool)
        .await;

    match result {
        Ok(Some(post)) => Json(post).into_response(),
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Post not found" })),
        )
            .into_response(),
        Err(err) => {
            eprintln!("Database query failed: {:?}", err);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal Server Error" })),
            )
                .into_response()
        }
    }
}

