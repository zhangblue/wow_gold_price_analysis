use axum::{
    body::{to_bytes, Body},
    http::{HeaderValue, Request, StatusCode},
    Router,
};
use chrono::NaiveDate;
use gold_price_backend::{
    app::build_app,
    repository::gold_prices::{DailyGoldPrice, GoldPriceReader, RepositoryError},
};
use std::{future::Future, path::PathBuf};
use tower::ServiceExt;

#[derive(Clone)]
struct FakeRepository {
    prices: Vec<DailyGoldPrice>,
    fails: bool,
}

impl GoldPriceReader for FakeRepository {
    fn daily_medians(
        &self,
        _start: NaiveDate,
        _end: NaiveDate,
    ) -> impl Future<Output = Result<Vec<DailyGoldPrice>, RepositoryError>> + Send {
        let result = if self.fails {
            Err(RepositoryError::Query)
        } else {
            Ok(self.prices.clone())
        };

        async move { result }
    }
}

fn test_app(result: Result<Vec<DailyGoldPrice>, RepositoryError>) -> Router {
    let (prices, fails) = match result {
        Ok(prices) => (prices, false),
        Err(_) => (vec![], true),
    };

    build_app(
        FakeRepository { prices, fails },
        PathBuf::from("missing-test-dist"),
        HeaderValue::from_static("http://localhost:5173"),
    )
}

#[tokio::test]
async fn rejects_a_reversed_date_range() {
    let response = test_app(Ok(vec![]))
        .oneshot(
            Request::builder()
                .uri("/api/gold-prices?start_date=2026-08-31&end_date=2026-08-01")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response_body(response).await,
        r#"{"error":"结束日期不能早于开始日期"}"#
    );
}

#[tokio::test]
async fn rejects_a_malformed_date() {
    let response = test_app(Ok(vec![]))
        .oneshot(
            Request::builder()
                .uri("/api/gold-prices?start_date=08-01-2026&end_date=2026-08-31")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(response_body(response).await, r#"{"error":"请求参数无效"}"#);
}

#[tokio::test]
async fn rejects_a_missing_date() {
    let response = test_app(Ok(vec![]))
        .oneshot(
            Request::builder()
                .uri("/api/gold-prices?start_date=2026-08-01")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(response_body(response).await, r#"{"error":"请求参数无效"}"#);
}

#[tokio::test]
async fn returns_an_empty_data_array_when_no_prices_match() {
    let response = test_app(Ok(vec![]))
        .oneshot(
            Request::builder()
                .uri("/api/gold-prices?start_date=2026-08-01&end_date=2026-08-31")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response_body(response).await, r#"{"data":[]}"#);
}

#[tokio::test]
async fn returns_daily_prices_as_json() {
    let response = test_app(Ok(vec![DailyGoldPrice {
        date: NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        price: 0.0121,
    }]))
    .oneshot(
        Request::builder()
            .uri("/api/gold-prices?start_date=2026-08-01&end_date=2026-08-31")
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response_body(response).await,
        r#"{"data":[{"date":"2026-08-01","price":0.0121}]}"#
    );
}

#[tokio::test]
async fn hides_repository_failures() {
    let response = test_app(Err(RepositoryError::Query))
        .oneshot(
            Request::builder()
                .uri("/api/gold-prices?start_date=2026-08-01&end_date=2026-08-31")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response_body(response).await,
        r#"{"error":"服务器内部错误"}"#
    );
}

async fn response_body(response: axum::response::Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}
