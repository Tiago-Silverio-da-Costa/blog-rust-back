use crate::{
    helpers::middleware::token::HelperMiddlewareToken,
    mvc::controllers::user::controller_user::ControllerUser,
};
use axum::{
    body::Body,
    http::{Method, Request},
    middleware::from_fn,
    middleware::Next,
    response::Response,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use std::env;
use tower_http::cors::{Any, CorsLayer};

async fn auth_middleware(req: Request<Body>, next: Next) -> Response {
    let auth: HelperMiddlewareToken = HelperMiddlewareToken::new();
    auth.verify_token(req, next).await
}

pub fn create_routes() -> Router {
    dotenv().ok();
    let base_url: String = env::var("BASE_URL").expect("BASE_URL não configurada");

    let cors = CorsLayer::new()
        .allow_origin(base_url.parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    let public_routes = Router::new()
        .route("/register", post(ControllerUser::register_user))
        .route("/login", post(ControllerUser::login))
        .route(
            "/fg/send/email",
            post(ControllerUser::fg_send_code_to_email),
        )
        .route("/fg/check/code", post(ControllerUser::fg_check_code));

    let protected_routes = Router::new()
        .route(
            "/fg/update/password",
            post(ControllerUser::fg_update_user_password).layer(from_fn(auth_middleware)),
        )
        .route(
            "/session",
            get(ControllerUser::get_me).layer(from_fn(auth_middleware)),
        );

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors)
}
