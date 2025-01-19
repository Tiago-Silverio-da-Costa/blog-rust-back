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
    }

    pub mod controllers {
        pub mod user {
            pub mod controller_user;
        }
    }

    pub mod routes {
        pub mod user {
            pub mod route_user;
        }
    }
}

use crate::helpers::db::helpers_mysql::HelperMySql;

#[tokio::main]
async fn main() {

    let app: axum::Router = server::create_app().await;
    let listener: tokio::net::TcpListener =
        tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Servidor rodando em http://127.0.0.1:8080");

    match HelperMySql::init().await {
        Ok(_helper) => {
            println!("Conexão estabelecida com sucesso!")
        }
        Err(e) => {
            eprintln!("Erro ao conectar ao banco: {}", e)
        }
    };

    axum::serve(listener, app).await.unwrap();
}
