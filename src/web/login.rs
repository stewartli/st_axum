use axum::{routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{MyResult, MyError};

#[derive(Debug, Deserialize)]
struct LoginPayload{
    user: String, 
    pass: String,
}

async fn handle_login(Json(login): Json<LoginPayload>) -> MyResult<Json<Value>>{
    if login.user != "hello" || login.pass != "world"{
        return Err(MyError::LoginFail);
    }
    let res = json!({
        "auth": {"success": true}
    });
    Ok(Json(res))
}    

pub fn route_login() -> Router{
    Router::new().route("/login", post(handle_login))
}



