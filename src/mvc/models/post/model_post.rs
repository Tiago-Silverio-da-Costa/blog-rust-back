use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use sqlx::{prelude::FromRow, Row};

use crate::helpers::{db::helpers_mysql::HelperMySql, response::helpers_response::HelpersResponse};

pub struct ModelPost;

#[derive(Debug, Serialize, Deserialize)]
pub struct PostRequestModel {
    pub post: PostRequestItem,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PostRequest {
    pub post: PostRequestItem,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostRequestItem {
    pub author_id: i32,
    pub category_id: i32,
    pub title: String,
    pub description: String,
    pub post_image_url: Option<String>,
    pub content: String,
    pub slug: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub author_id: i32,
    pub category_id: i32,
    pub title: String,
    pub description: String,
    pub publication_date: NaiveDateTime,
    pub post_image_url: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuthor {
    pub author: CreateAuthorItem,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuthorItem {
    pub name: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct Author {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct CategoryReq {
    pub id: i32,
    pub name: String,
}

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

impl ModelPost {
    pub async fn select_post() -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let query = "    SELECT 
                p.id AS post_id, p.author_id, a.id AS author_id, a.name AS author_name,
                p.category_id, c.id AS category_id, c.name AS category_name,
                p.*
            FROM 
                    posts p
                LEFT JOIN authors a ON p.author_id = a.id
                LEFT JOIN categories c ON p.category_id = c.id;
          "; // Ajuste para a sua tabela
        match HelperMySql::execute_select(query).await {
            Ok(rows) => {
                // Convertendo linhas para JSON
                let posts: Vec<serde_json::Value> = rows
                    .into_iter()
                    .map(|row| {
                        json!({
                              "id": row.try_get::<i32, _>("id").unwrap_or_default(),
                               "author_id": row.try_get::<i32, _>("author_id").unwrap_or_default(),
                               "author_name": row.try_get::<String, _>("author_name").unwrap_or_default(),
                               "category_id": row.try_get::<i32, _>("category_id").unwrap_or_default(),
                               "category_name": row.try_get::<String, _>("category_name").unwrap_or_default(),
                               "title": row.try_get::<String, _>("title").unwrap_or_default(),
                               "description": row.try_get::<String, _>("description").unwrap_or_default(),
                               "publication_date": row.try_get::<NaiveDateTime, _>("publication_date").unwrap_or_default(),
                               "post_image_url": row.try_get::<Option<String>, _>("post_image_url").unwrap_or(None),
                               "content": row.try_get::<String, _>("content").unwrap_or_default(),
                               "created_at": row.try_get::<DateTime<Utc>, _>("created_at").unwrap_or_default(),
                               "updated_at": row.try_get::<DateTime<Utc>, _>("updated_at").unwrap_or_default(),
                               "slug": row.try_get::<String, _>("slug").unwrap_or_default(),

                        })
                    })
                    .collect();
                Ok(posts)
            }
            Err(err) => Err(err),
        }
    }

    pub async fn select_post_by_id(post_id: i32) -> Result<serde_json::Value, ApiError> {
        let query = r#"
            SELECT 
                p.id AS post_id, p.author_id, a.id AS author_id, a.name AS author_name,
                p.category_id, c.id AS category_id, c.name AS category_name,
                p.title, p.description, p.publication_date, p.post_image_url, 
                p.content, p.created_at, p.updated_at
            FROM 
                posts p
            LEFT JOIN authors a ON p.author_id = a.id
            LEFT JOIN categories c ON p.category_id = c.id;
        "#;

        // Executa a consulta ao banco de dados com o ID como parâmetro
        let params: Vec<i32> = vec![post_id.into()];
        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(rows) => {
                // Verifica se houve retorno
                if rows.is_empty() {
                    return Err(ApiError {
                        status_code: StatusCode::NOT_FOUND,
                        message: "Post não encontrado".to_string(),
                    });
                }

                // Mapeia os dados do primeiro resultado para JSON
                let row = &rows[0];
                let post = json!({
                    "id": row.try_get::<i32, _>("id").unwrap_or_default(),
                    "author_id": row.try_get::<i32, _>("author_id").unwrap_or_default(),
                    "author_name": row.try_get::<String, _>("author_name").unwrap_or_default(),
                    "category_id": row.try_get::<i32, _>("category_id").unwrap_or_default(),
                    "category_name": row.try_get::<String, _>("category_name").unwrap_or_default(),
                    "title": row.try_get::<String, _>("title").unwrap_or_default(),
                    "description": row.try_get::<String, _>("description").unwrap_or_default(),
                    "publication_date": row.try_get::<NaiveDateTime, _>("publication_date").unwrap_or_default().format("%Y-%m-%d").to_string(),
                    "post_image_url": row.try_get::<Option<String>, _>("post_image_url").unwrap_or(None),
                    "content": row.try_get::<String, _>("content").unwrap_or_default(),
                    "created_at": row.try_get::<DateTime<Utc>, _>("created_at").unwrap_or_default(),
                    "updated_at": row.try_get::<DateTime<Utc>, _>("updated_at").unwrap_or_default(),
                });

                Ok(post)
            }
            Err(err) => Err(ApiError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("Erro ao buscar o post: {}", err),
            }),
        }
    }

    pub async fn get_all_slugs() -> Result<Vec<String>, sqlx::Error> {
        let query = "SELECT slug FROM  posts";
        let rows = HelperMySql::execute_select(query).await?;
        let slugs: Vec<String> = rows.into_iter().map(|row| row.get("slug")).collect();
        Ok(slugs)
    }

    pub async fn get_all_categories() -> Result<Vec<CategoryReq>, sqlx::Error> {
        let query = "SELECT * from categories";
        let rows = HelperMySql::execute_select(query).await?;
        let categories: Result<Vec<CategoryReq>, sqlx::Error> = rows
            .into_iter()
            .map(|row| CategoryReq::from_row(&row))
            .collect();
        categories
    }

    pub async fn get_all_authors() -> Result<Vec<Author>, sqlx::Error> {
        let query = "SELECT * from authors";
        let rows = HelperMySql::execute_select(query).await?;
        let authors: Result<Vec<Author>, sqlx::Error> =
            rows.into_iter().map(|row| Author::from_row(&row)).collect();
        authors
    }

    pub async fn create_author(create_author: CreateAuthor) -> impl IntoResponse {
        let query = r#"
            INSERT INTO authors (name) 
            VALUES (?)
        "#;

        let params = vec![create_author.author.name.to_string()];

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(_) => HelpersResponse::success("Autor criado!", create_author).into_response(),
            Err(_) => HelpersResponse::error("Erro ao criar autor").into_response(),
        }
    }

    pub async fn create_post(slug: &str, post_request: PostRequest) -> impl IntoResponse {
        let query = r#"
        INSERT INTO posts (author_id, category_id, title, description, post_image_url, content, slug)
        VALUES (?, ?, ?, ?, ?, ?, ?)
    "#;

        let params = vec![
            post_request.post.author_id.to_string(),
            post_request.post.category_id.to_string(),
            post_request.post.title,
            post_request.post.description,
            post_request.post.post_image_url.unwrap_or_default(),
            post_request.post.content,
            slug.to_string(),
        ];

        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(_) => HelpersResponse::success("Post criado!", slug).into_response(),
            Err(_e) => {
                HelpersResponse::error_with_detail("Erro ao criar post!", _e).into_response()
            }
        }
    }

    pub async fn select_post_by_slug(slug: String) -> Result<serde_json::Value, ApiError> {
        let query = r#"
        SELECT 
            p.id AS post_id, p.author_id, a.id AS author_id, a.name AS author_name,
            p.category_id, c.id AS category_id, c.name AS category_name,
            p.title, p.description, p.publication_date, p.post_image_url, 
            p.content, p.created_at, p.updated_at, p.slug
        FROM 
            posts p
        LEFT JOIN authors a ON p.author_id = a.id
        LEFT JOIN categories c ON p.category_id = c.id
        WHERE p.slug = ?
        "#;

        let params: Vec<String> = vec![slug];
        match HelperMySql::execute_query_with_params(query, params).await {
            Ok(rows) => {
                if rows.is_empty() {
                    return Err(ApiError {
                        status_code: StatusCode::NOT_FOUND,
                        message: "Post não encontrado".to_string(),
                    });
                }

                let row = &rows[0];
                let post = json!({
                    "id": row.try_get::<i32, _>("post_id").unwrap_or_default(),
                    "author_id": row.try_get::<i32, _>("author_id").unwrap_or_default(),
                    "author_name": row.try_get::<String, _>("author_name").unwrap_or_default(),
                    "category_id": row.try_get::<i32, _>("category_id").unwrap_or_default(),
                    "category_name": row.try_get::<String, _>("category_name").unwrap_or_default(),
                    "title": row.try_get::<String, _>("title").unwrap_or_default(),
                    "description": row.try_get::<String, _>("description").unwrap_or_default(),
                    "publication_date": row.try_get::<NaiveDateTime, _>("publication_date").unwrap_or_default(),
                    "post_image_url": row.try_get::<Option<String>, _>("post_image_url").unwrap_or(None),
                    "content": row.try_get::<String, _>("content").unwrap_or_default(),
                    "created_at": row.try_get::<DateTime<Utc>, _>("created_at").unwrap_or_default(),
                    "updated_at": row.try_get::<DateTime<Utc>, _>("updated_at").unwrap_or_default(),
                    "slug": row.try_get::<String, _>("slug").unwrap_or_default(),
                });

                Ok(post)
            }
            Err(err) => Err(ApiError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("Erro ao buscar o post: {}", err),
            }),
        }
    }
}
