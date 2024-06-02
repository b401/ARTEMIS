use askama_axum::Template;
use axum::http::StatusCode;

#[derive(Template,Debug)]
#[template(path = "error.html")]
pub struct ErrorHandler {
    pub code: StatusCode,
    pub msg: String
}

pub async fn code_404() -> ErrorHandler {
    ErrorHandler {
        code: StatusCode::NOT_FOUND,
        msg: "no route for uri".to_string(),
    }
}

pub async fn code_generic(code: u16) -> ErrorHandler {
    ErrorHandler {
        code: StatusCode::from_u16(code).unwrap_or(StatusCode::IM_A_TEAPOT),
        msg: "generic error".to_string(),
    }
}

// impl IntoResponse for ErrorHandler {
//     fn into_response(self) -> Response {
//         // Convert your ErrorHandler to a response
//         // This is a basic example; adjust according to your error handling logic
//         let error_message = format!("Error: {:?}", self);
//         (axum::http::StatusCode::INTERNAL_SERVER_ERROR, error_message).into_response()
//     }
// }}

