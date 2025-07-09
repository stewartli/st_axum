use serde::Deserialize;
use tower_http::services::ServeDir;
use axum::{
    extract::Query, handler::HandlerWithoutStateExt, http::StatusCode, response::{Html, IntoResponse}, routing::{any_service, get, get_service, MethodRouter}, Router 
};

use crate::config;

#[derive(Debug, Deserialize)]
struct MyJob{
    name: String, 
}

async fn handle_home(Query(job): Query<MyJob>) -> impl IntoResponse{
    // /?name=mas
    let res = format!("<h1>this is home page for {}</h1>", job.name);
    Html(res)
}

pub fn route_app() -> Router{
    Router::new()
        .route("/", get(handle_home))
}

// method 1
pub fn route_static() -> Router{
    // /src/main.rs
    Router::new().
        nest_service("/", get_service(ServeDir::new("./")))
}

// method 2
pub fn route_static1() -> MethodRouter{
    async fn handle_404() -> (StatusCode, &'static str){
        (StatusCode::NOT_FOUND, "web folder not found")
    }
    any_service(ServeDir::new(&config().WEB_FOLDER).not_found_service(handle_404.into_service()))
}




