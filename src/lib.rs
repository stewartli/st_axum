use axum::{
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

pub use self::err::{MyError, MyResult};
use route::{route_app, route_static};

#[tokio::main]
pub async fn run(){
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8090").await.unwrap();
    // shared state
    let mydb = model::MyDB::new().await.unwrap();
    // middleware
    let route_api = model::route_ticket(mydb)
        .route_layer(middleware::from_fn(map_auth));
    // main route
    let app = Router::new()
        .merge(route_app())
        .merge(web::login::route_login())
        .nest("/api", route_api)
        .layer(middleware::map_response(map_resp))
        .fallback_service(route_static());
    // start server
    println!("server listening on {}", listener.local_addr().unwrap());
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


