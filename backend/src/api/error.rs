use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: &'static str,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
}

impl ApiError {
    pub fn invalid_query() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: "请求参数无效",
        }
    }

    pub fn reversed_date_range() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: "结束日期不能早于开始日期",
        }
    }

    pub fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "服务器内部错误",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
    }
}
