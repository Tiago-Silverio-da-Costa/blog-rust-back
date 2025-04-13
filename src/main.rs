mod server;

pub mod helpers {
    pub mod db {
        pub mod helpers_mysql;
    }
    pub mod middleware {
        pub mod token;
    }
    pub mod response {
        pub mod helpers_response;
    }
}

pub mod mvc {
    pub mod models {
        pub mod user {
            pub mod model_user;
        }

        pub mod post {
            pub mod model_post;
        }

        pub mod comment {
            pub mod model_comment;
        }
    }

    pub mod controllers {
        pub mod user {
            pub mod controller_user;
        }

        pub mod post {
            pub mod controller_post;
        }

        pub mod comment {
            pub mod controller_comment;
        }
    }

    pub mod routes {
        pub mod user {
            pub mod route_user;
        }

        pub mod post {
            pub mod route_post;
        }

        pub mod comment {
            pub mod route_comment;
        }
    }

    pub mod services {
        pub mod user {
            pub mod email {
                pub mod services_user_email;
            }
        }
    }
}

use crate::helpers::db::helpers_mysql::HelperMySql;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let base_url_back: String = env::var("BASE_URL_BACK").expect("BASE_URL não configurada");

    let app: axum::Router = server::create_app().await;
    let listener: tokio::net::TcpListener =
        tokio::net::TcpListener::bind(base_url_back).await.unwrap();

    match HelperMySql::init().await {
        Ok(_helper) => {
            println!("Conexão estabelecida com sucesso!")
        }
        Err(e) => {
            eprintln!("Erro ao conectar ao banco: {}", e)
        }
    };

    axum::serve(listener, app).await.unwrap();
    println!("passou aqui");
}
