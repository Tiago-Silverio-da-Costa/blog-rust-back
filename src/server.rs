use axum::Router;

use crate::mvc::routes::user::route_user;

pub async fn create_app() -> Router {
    Router::new()
        .nest("/user", route_user::create_routes()
    )
}