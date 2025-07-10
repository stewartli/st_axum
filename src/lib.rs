#![allow(unused_imports)]

use tracing::Span;
use tracing_subscriber::EnvFilter;
use tower::{Layer , Service, ServiceBuilder};
use tower_http::trace::TraceLayer;
use axum::{
    body::Body, 
    extract::{FromRequestParts, Request}, 
    http::{Method, Response, Uri}, 
    middleware::{self, Next}, 
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
        // tower Service trait
        .layer(MyHelloLayer::new())
        // fallback service
        .fallback_service(route_static())
        // fallback handler [not matched route]
        .fallback(route_static1());
        // .layer(TraceLayer::new_for_http().make_span_with(|_| tracing::info_span!("http-request")).on_request(trace_req))

    axum::serve(listener, app).await.unwrap();
}

// 01. middleware
// map_res: https://docs.rs/axum/latest/axum/middleware/fn.map_response.html
async fn map_resp(mut res: axum::response::Response) -> axum::response::Response{
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
) -> MyResult<axum::response::Response>{
    // request methods
    let target = req.headers().get("x-foo").unwrap().to_str().unwrap();
    println!("{}", target);
    // use extractor
    println!("id is {} by calling {}", user?.id, method.as_str());
    // response methods
    Ok(next.run(req).await)
}

// 02. extractor 
// Extension: https://docs.rs/axum/latest/axum/struct.Extension.html
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

// 04 Service trait 
// https://docs.rs/tower/latest/tower/trait.Service.html
#[derive(Clone)]
struct MyHello<S>{
    inner: S, 
}

impl<S> MyHello<S>{
    fn new(inner: S) -> Self{
        Self{inner}
    }
}

impl<S, B> tower::Service<Request<B>> for MyHello<S>
where 
    S: Service<Request<B>> + Send + Clone + 'static,
    S::Future: Send, 
    // 1> constrait your response
    // S::Response: Debug + Display, 
{
    // 2> specify response or other type [http vs axum response]
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        tracing::info!("call service trait {}", req.method());
        self.inner.call(req)
    }
} 

#[derive(Clone)]
struct MyHelloLayer;

impl MyHelloLayer{
    fn new() -> Self{
        Self{}
    }
}

impl<S> Layer<S> for MyHelloLayer{
    type Service = MyHello<S>;
    fn layer(&self, inner: S) -> Self::Service {
        MyHello::new(inner)
    }
}

