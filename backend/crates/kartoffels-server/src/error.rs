use anyhow::Error;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub type AppResult<T, E = AppError> = Result<T, E>;

#[derive(Clone, Debug)]
pub enum AppError {
    WorldNotFound,
    Other(StatusCode, String),
}

impl AppError {
    pub const MAP_HTTP_400: fn(Error) -> Self =
        |err| AppError::Other(StatusCode::BAD_REQUEST, format!("{:?}", err));

    pub const MAP_HTTP_500: fn(Error) -> Self = |err| {
        AppError::Other(StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err))
    };
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::WorldNotFound => {
                let status = StatusCode::NOT_FOUND;
                let body = "world not found".into();

                (status, body)
            }

            AppError::Other(status, body) => (status, body),
        };

        (status, body).into_response()
    }
}
