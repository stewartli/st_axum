use axum::Router;

mod route;
mod err;
mod web;

pub use self::err::{MyError, MyResult};
use route::{route_app, route_static};

#[tokio::main]
pub async fn run(){
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8090").await.unwrap();

    let app = Router::new()
        .merge(route_app())
        .merge(web::login::route_login())
        .fallback_service(route_static());

    axum::serve(listener, app).await.unwrap();
}

