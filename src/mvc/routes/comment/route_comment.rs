use axum::{
    body::Body,
    http::{Method, Request},
    middleware::from_fn,
    middleware::Next,
    response::Response,
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    helpers::middleware::token::HelperMiddlewareToken,
    mvc::controllers::comment::controller_comment::ControllerComment,
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
        .route("/{id}", get(ControllerComment::get_all_comments_by_post)) // GET para buscar comentários de um post
        .route("/", post(ControllerComment::post_new_comment));

    let protected_routes = Router::new()
        .route(
            "/comments/protected",
            post(|| async { "Endpoint protegido" }),
        ) // Exemplo de endpoint protegido
        .layer(from_fn(auth_middleware));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors)
}
