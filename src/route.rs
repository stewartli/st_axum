use serde::Deserialize;

use tower_http::services::ServeDir;
use axum::{
    extract::Query, 
    response::{Html, IntoResponse}, 
    routing::{get, get_service}, 
    Router, 
};

// 01. query
#[derive(Debug, Deserialize)]
struct MyJob{
    name: String, 
}

async fn handle_home(Query(job): Query<MyJob>) -> impl IntoResponse{
    // /?name=mas
    let res = format!("<h1>this is home page for {}</h1>", job.name);
    Html(res)
}


// 02. route
pub fn route_app() -> Router{
    let res = Router::new()
        .route("/", get(handle_home));
    res
}

pub fn route_static() -> Router{
    // /src/main.rs
    Router::new().
        nest_service("/", get_service(ServeDir::new("./")))
}


