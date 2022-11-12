use askama_axum::Template;
use axum::http::StatusCode;

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorHandler {
    pub code: StatusCode,
}

pub async fn code_404() -> ErrorHandler {
    ErrorHandler {
        code: StatusCode::NOT_FOUND,
    }
}

pub async fn code_generic(code: u16) -> ErrorHandler {
    ErrorHandler {
        code: StatusCode::from_u16(code).unwrap_or(StatusCode::IM_A_TEAPOT),
    }
}
