#![allow(unused_imports)]

use serde::Deserialize;
use validator::Validate;
use askama::Template;
use tower_http::services::ServeDir;

use axum::{
    extract::Query, 
    handler::HandlerWithoutStateExt, 
    http::StatusCode, 
    response::{Html, IntoResponse, Redirect, Response}, 
    routing::{any_service, get, get_service, MethodRouter}, 
    Form, Router,    
};

use crate::config;

// 01. query param
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
    Router::new().route("/", get(handle_home))
}

// 02. serve static
// method 1
pub fn route_static() -> Router{
    // /src/main.rs
    Router::new().
        // nest_service("/", get_service(ServeDir::new("./")))
        nest_service("/", ServeDir::new("asset"))
}

// method 2
pub fn route_static1() -> MethodRouter{
    async fn handle_404() -> (StatusCode, &'static str){
        (StatusCode::NOT_FOUND, "web folder not found")
    }
    any_service(ServeDir::new(&config().WEB_FOLDER).not_found_service(handle_404.into_service()))
}

// 03. html templates
// https://askama.readthedocs.io/en/latest/template_syntax.html?highlight=call%20runction#call
#[derive(askama::Template)]
#[template(path = "page/home.html")]
struct HomeTemplate{}

#[derive(askama::Template)]
#[template(path = "page/signup.html")]
struct SignUpTemplate{
    // only for UI 
    inner: usize, 
    // askama HtmlSafe trait does not suport struct with const param
    // change: fn(usize) -> usize, 
}

fn change(x: &usize) -> usize{
    *x + 10
}

async fn handle_html_home() -> impl IntoResponse{
    let res = HomeTemplate{}.render().unwrap();
    Html(res)
}

async fn handle_html_sign() -> impl IntoResponse{
    let res = SignUpTemplate{inner: 0}.render().unwrap();
    Html(res)
}

#[derive(Debug, Deserialize, Validate)]
struct UserForm{
    email: String,
    #[validate(length(min = 10, message = "incorrect password"))]
    pass: String, 
}

async fn handle_form(Form(user_form): Form<UserForm>) -> Response {
    tracing::info!("email: {}, pass: {}", user_form.email, user_form.pass);
    let res = match user_form.validate(){
        Ok(_) => Redirect::to("/").into_response(), 
        Err(x) => {
            let inner = x.to_string().len();
            let target = SignUpTemplate{inner}.render().unwrap();
            (StatusCode::BAD_REQUEST, Html(target)).into_response()
        }
    };
    res
}

pub fn route_html() -> Router{
    Router::new()
        .route(
            "/temp", 
            get(handle_html_home)
                .get(handle_html_sign)
                .post(handle_form)
        )
}

