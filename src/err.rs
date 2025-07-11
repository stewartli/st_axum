use axum::{
    http::StatusCode, 
    response::IntoResponse, Json, 
};

pub type MyResult<T> = core::result::Result<T, MyError>;

#[derive(Debug, thiserror::Error)]
pub enum MyError{
    #[error("fail to login")]
    LoginFail, 
    #[error("fail to config {0}")]
    ConfigEnvMissing(#[from] std::env::VarError), 
}

impl IntoResponse for MyError{
    fn into_response(self) -> axum::response::Response {
        match self{
            MyError::LoginFail => {
                (StatusCode::INTERNAL_SERVER_ERROR, "LOGIN_FAIL").into_response()
            }
            MyError::ConfigEnvMissing(x) => {
                (StatusCode::BAD_REQUEST, format!("ENV_FAIL: {}", x.to_string())).into_response()
            }
        }
    }
}



