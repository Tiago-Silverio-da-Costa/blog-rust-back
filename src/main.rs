use axum::{
    extract::State,
    http::Method,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use chrono::{NaiveDate, NaiveDateTime};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::FromRow;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Post {
    id: Option<i32>,
    author_id: i32,
    title: String,
    description: String,
    publication_date: chrono::NaiveDateTime,
    category_id: i32,
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

pub struct AppState {
    db: MySqlPool,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    println!("Starting Rest API Service");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must set");
    let pool = match MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            println!("Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("Failed to connect to the database: {:?}", err);
            std::process::exit(1)
        }
    };

    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:3000"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/healthcheck", get(health_check_handle))
        .route("/api/posts", get(get_all_posts))
        .route("/api/posts/{id}", get(get_post_by_id))
        // .route("/api/login", post(auth_login))
        // .route("/api/register", post(register))
        // .route("/api/sendcodetoemail", post(send_code_to_email))
        // .route("/api/checkcode", post(check_code))
        // .route("/api/newpassword", post(create_new_password))
        .with_state(Arc::new(AppState { db: pool.clone() }))
        .layer(cors);

    println!("Server started succussfully at http://127.0.0.1:8080/api/healthcheck");

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
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
        .fetch_all(&state.db)
        .await
        .unwrap();

    Json(posts)
}

pub async fn get_post_by_id(
    State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
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
