use axum::{
    http::StatusCode, 
    response::IntoResponse, 
};

pub type MyResult<T> = core::result::Result<T, MyError>;

#[derive(Debug, thiserror::Error)]
pub enum MyError{
    #[error("fail to login")]
    LoginFail, 
}

impl IntoResponse for MyError{
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "LOGIN_FAIL").into_response()
    }
}



