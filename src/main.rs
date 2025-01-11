use axum::{ extract::State, response::IntoResponse, routing::{get, post}, Json, Router};
use dotenv::dotenv;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::FromRow;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use std::sync::Arc;
use chrono::{NaiveDate, NaiveDateTime};
use chrono::{DateTime, Utc};

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
    created_at:  DateTime<Utc>,
    updated_at: DateTime<Utc>
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
    photo_url: Option<String>
}

pub struct AppState {
    db: MySqlPool
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    println!("Starting Rest API Service");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must set");
    let pool = match MySqlPoolOptions::new().max_connections(10).connect(&database_url).await {
        Ok(pool) => {
            println!("Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("Failed to connect to the database: {:?}", err);
            std::process::exit(1)
        }
    };
    
    let app = Router::new()
        .route("/api/healthcheck", get(health_check_handle))
        .route("/api/posts", get(get_all_posts))
        .route("/api/posts", post(create_post))
        .route("/api/posts/{id}/comments", post(create_comment))
        .with_state(Arc::new(AppState {db: pool.clone() }));

    println!("Server started succussfully at http://127.0.0.1:8080/api/healthcheck");

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
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

async fn create_post(
    State(state): axum::extract::State<Arc<AppState>>,
    Json(new_post): Json<Post>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "INSERT INTO posts (title, description, content, post_image_url , author_id, category_id, publication_date, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        new_post.title,
        new_post.description,
        new_post.content,
        new_post.post_image_url,
        new_post.author_id,
        new_post.category_id,
        new_post.publication_date,
        new_post.created_at,
        new_post.updated_at,
    )
    .execute(&state.db)
    .await
    .unwrap();

    Json(serde_json::json!({ "status": "success", "affected_rows": result.rows_affected() }))

}

async fn create_comment(
    State(state): axum::extract::State<Arc<AppState>>,
    Json(new_comment): Json<Comment>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "INSERT INTO comments (content, user_id, post_id) VALUES (?, ?, ?)",
        new_comment.content,
        new_comment.user_id,
        new_comment.post_id
    )
    .execute(&state.db)
    .await
    .unwrap();

    Json(serde_json::json!({ "status": "success", "affected_rows": result.rows_affected() }))
}