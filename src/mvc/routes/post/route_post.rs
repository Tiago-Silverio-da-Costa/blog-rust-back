use axum::{
    body::Body,
    http::{Method, Request},
    middleware::from_fn,
    middleware::Next,
    response::Response,
    routing::{get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    helpers::middleware::token::HelperMiddlewareToken,
    mvc::controllers::post::controller_post::ControllerPost,
};

async fn auth_middleware(req: Request<Body>, next: Next) -> Response {
    let auth: HelperMiddlewareToken = HelperMiddlewareToken::new();
    auth.verify_token(req, next).await
}

pub fn create_routes() -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:3000"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    let public_routes = Router::new()
        .route("/", get(ControllerPost::get_all_posts))
        .route("/{id}", get(ControllerPost::get_post_by_id))
        .route("/slug/{slug}", get(ControllerPost::get_post_by_slug));

    let protected_routes = Router::new()
        .route(
            "/create",
            post(ControllerPost::create_post).layer(from_fn(auth_middleware)),
        )
        .route(
            "/author",
            get(ControllerPost::get_all_authors).layer(from_fn(auth_middleware)),
        )
        .route(
            "/category",
            get(ControllerPost::get_all_categories).layer(from_fn(auth_middleware)),
        )
        .route(
            "/create/author",
            post(ControllerPost::create_author).layer(from_fn(auth_middleware)),
        )
        .route(
            "/create/category",
            post(ControllerPost::create_category).layer(from_fn(auth_middleware)),
        )
        .route(
            "/edit",
            put(ControllerPost::edit_post).layer(from_fn(auth_middleware)),
        )
        .route(
            "/remove",
            put(ControllerPost::delete_post).layer(from_fn(auth_middleware)),
        );

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors)
}
