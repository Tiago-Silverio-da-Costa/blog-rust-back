use axum::Router;

use crate::mvc::routes::user::route_user;
use crate::mvc::routes::post::route_post;
use crate::mvc::routes::comment::route_comment;


pub async fn create_app() -> Router {
    Router::new()
        .nest("/user", route_user::create_routes())
        .nest("/post", route_post::create_routes())
        .nest("/comments", route_comment::create_routes())
}