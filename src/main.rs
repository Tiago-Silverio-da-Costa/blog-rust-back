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
use tower_http::cors::{Any, CorsLayer};

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

pub struct AppState {
    pub pool: MySqlPool,
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
        .route("/api/register", post(register))
        // .route("/api/sendcodetoemail", post(send_code_to_email))
        // .route("/api/checkcode", post(check_code))
        // .route("/api/newpassword", post(create_new_password))
        .with_state(Arc::new(AppState { pool: pool.clone() }))
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

pub async fn email_already_exists(
    email: &str,
    pool: &MySqlPool,
) -> Result<(), (StatusCode, Json<Value>)> {
    println!("passou12");
    let query = "SELECT COUNT(*) as count FROM users WHERE email = ?";
    println!("Executando consulta para verificar email...");
    match sqlx::query(query).bind(email).fetch_one(pool).await {
        Ok(row) => {
            let count: i64 = row.get("count");
            if count > 0 {
                println!("Email já existe.");
                Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "status": false,
                        "message": "Erro ao verificar email"
                    })),
                ))
            } else {
                Ok(())
            }
        }
        Err(err) => {
            println!("Erro ao executar consulta: {:?}", err);
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

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let pool = &state.pool;

    println!(
        "Recebido: Nome: {}, Email: {}, Senha: {}",
        payload.name, payload.email, payload.password
    );

    if let Err(err) = email_already_exists(&payload.email, pool).await {
        println!("Erro ao verificar email: {:?}", err);
        return Err(err.0); // Retorna o StatusCode do erro
    }

    let hashed_password = match bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST) {
        Ok(hash) => {
            println!("Senha criptografada com sucesso.");
            hash
        }
        Err(err) => {
            println!("Erro ao criptografar senha: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    println!("Recebido: hashed_password: {}", hashed_password);


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
