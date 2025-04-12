use crate::mvc::models::post::model_post::{
    CreateAuthor, CreateCategory, EditPost, ModelPost, PostRequest,
};
use axum::{extract::Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde_json::{json, Value};

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

    pub async fn get_all_categories() -> impl IntoResponse {
        match ModelPost::get_all_categories().await {
            Ok(categories) => (
                StatusCode::OK,
                Json(json!({ "status": true, "data": categories})),
            ),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": false,
                    "message": format!("Erro ao buscar categorias {}", err)
                })),
            ),
        }
    }

    pub async fn get_all_authors() -> impl IntoResponse {
        match ModelPost::get_all_authors().await {
            Ok(authors) => (
                StatusCode::OK,
                Json(json!({ "status": true, "data": authors })),
            ),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": false,
                    "message": format!("Erro ao buscar autores: {}", err)
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

    pub async fn create_author(
        Json(create_author): Json<CreateAuthor>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        Ok(ModelPost::create_author(create_author).await)
    }

    pub async fn create_category(
        Json(create_category): Json<CreateCategory>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        Ok(ModelPost::create_category(create_category).await)
    }

    pub async fn create_post(
        Json(create_post): Json<PostRequest>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        let existing_slugs = ModelPost::get_all_slugs().await.unwrap_or(vec![]);
        let slug = generate_slug(&create_post.post.title, existing_slugs);
        Ok(ModelPost::create_post(&slug, create_post).await)
    }

    pub async fn edit_post(
        Json(edit_post): Json<EditPost>,
    ) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
        Ok(ModelPost::edit_post(edit_post).await)
    }
}
