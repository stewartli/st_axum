#![allow(unused_imports)]

use tracing::Span;
use tracing_subscriber::EnvFilter;
use tower_http::trace::TraceLayer;
use axum::{
    body::Body, 
    extract::{FromRequestParts, Request}, 
    http::{Method, Uri}, 
    middleware::{self, Next}, 
    response::Response, 
    RequestPartsExt, 
    Router,   
};

mod route;
mod err;
mod web;
mod model;
mod config;

pub use self::err::{MyError, MyResult};
pub use config::config;
use route::{route_app, route_html, route_static, route_static1};

#[tokio::main]
pub async fn run(){
    // tracing
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // shared state
    let mydb = model::MyDB::new().await.unwrap();

    // middleware
    let route_api = model::route_ticket(mydb)
        .route_layer(middleware::from_fn(map_auth));

    // start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8090").await.unwrap();
    tracing::info!("server listening on {}", listener.local_addr().unwrap());

    let app = Router::new()
        .merge(route_app())
        .merge(route_html())
        .merge(web::login::route_login())
        .nest("/api", route_api)
        .layer(middleware::map_response(map_resp))
        .fallback_service(route_static())
        .fallback_service(route_static1());
        // .layer(TraceLayer::new_for_http().make_span_with(|_| tracing::info_span!("http-request")).on_request(trace_req))

    axum::serve(listener, app).await.unwrap();
}

// 01. middleware
// map_res: https://docs.rs/axum/latest/axum/middleware/fn.map_response.html
async fn map_resp(mut res: Response) -> Response{
    res.headers_mut().insert("x-foo", "foo".parse().unwrap());
    res
}

// without B: https://docs.rs/axum/latest/axum/?search=Request#middleware
// from_fn: https://docs.rs/axum/latest/axum/middleware/fn.from_fn.html
async fn map_auth(
    user: MyResult<User>, 
    method: Method, 
    req: Request, 
    next: Next, 
) -> MyResult<Response>{
    // request methods
    let target = req.headers().get("x-foo").unwrap().to_str().unwrap();
    println!("{}", target);
    // use extractor
    println!("id is {} by calling {}", user?.id, method.as_str());
    Ok(next.run(req).await)
}

// 02. extractor 
#[derive(Debug)]
struct User{
    id: i32, 
}

impl<S: Send + Sync> FromRequestParts<S> for User{
    type Rejection = MyError;
    async  fn from_request_parts(parts: &mut axum::http::request::Parts, _state: &S,) -> MyResult<Self>{
        let res = parts.extract::<Uri>().await.unwrap();
        let target = res.to_string().len() as i32;
        Ok(User { id: target })
    }
}

// 03. trace layer
// lifetime issue: https://docs.rs/tower-http/latest/tower_http/trace/index.html
#[allow(unused)]
fn trace_req(req: &Request<Body>, _: &Span){
    tracing::info!("request: method {} path {}", req.method(), req.uri().path());
}

