use axum::{
    body::Body,
    http::{Method, Request},
    middleware::from_fn,
    middleware::Next,
    response::Response,
    routing::post,
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    controllers::user::controller_user::ControllerUser,
    helpers::middleware::token::HelperMiddlewareToken,
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

    let public_routes = Router::new().route("/register", post(ControllerUser::register_user));

    let protected_routes = Router::new()
        .route("/login", post(ControllerUser::login))
        .layer(from_fn(auth_middleware));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors)
}
