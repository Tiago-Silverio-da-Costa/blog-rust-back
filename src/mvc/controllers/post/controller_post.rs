use crate::{
    helpers::db::helpers_mysql::HelperMySql,
    mvc::models::post::model_post::{ModelPost, PostRequest},
};
use axum::{extract::Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde_json::json;
use sqlx::Row;

pub struct ControllerPost;

fn generate_slug(title: &str, existing_slugs: Vec<String>) -> String {
    let base_slug = title
        .to_lowercase()
        .replace(" ", "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    let mut slug = base_slug.clone();
    let mut counter = 1;

    while existing_slugs.contains(&slug) {
        slug = format!("{}-{}", base_slug, counter);
        counter += 1;
    }

    slug
}

impl ControllerPost {
    pub async fn get_all_posts() -> impl IntoResponse {
        match ModelPost::select_post().await {
            Ok(posts) => (
                StatusCode::OK,
                Json(json!({ "status": true, "data": posts })),
            ),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": false,
                    "message": format!("Erro ao buscar posts: {}", err)
                })),
            ),
        }
    }

    pub async fn get_post_by_slug(Path(slug): Path<String>) -> impl IntoResponse {
        match ModelPost::select_post_by_slug(slug).await {
            Ok(post) => (
                StatusCode::OK,
                Json(json!({
                     "status": true,
                     "data": post,
                })),
            )
                .into_response(),
            Err(err) => err.into_response(),
        }
    }

    pub async fn get_all_slugs() -> Result<Vec<String>, sqlx::Error> {
        let query = "SELECT slug FROM posts";
        let rows = HelperMySql::execute_select(query).await?;
        let slugs: Vec<String> = rows.into_iter().map(|row| row.get("slug")).collect();
        Ok(slugs)
    }

    pub async fn get_post_by_id(Path(post_id): Path<i32>) -> impl IntoResponse {
        match ModelPost::select_post_by_id(post_id).await {
            Ok(post) => (
                StatusCode::OK,
                Json(json!({
                    "status": true,
                    "data": post,
                })),
            )
                .into_response(),
            Err(err) => err.into_response(),
        }
    }

    pub async fn create_post(
        Json(post_request): Json<PostRequest>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
        let existing_slugs = ModelPost::get_all_slugs().await.unwrap_or(vec![]);
        let slug = generate_slug(&post_request.post.title, existing_slugs);

        Ok(ModelPost::create_post(&slug, post_request).await)
    }
}
