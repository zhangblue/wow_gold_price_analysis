use crate::{
    api::error::ApiError,
    app::AppState,
    repository::gold_prices::{DailyGoldPrice, GoldPriceReader, SummaryRefresh},
};
use axum::{
    extract::{rejection::QueryRejection, Query, State},
    http::StatusCode,
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Deserialize)]
pub(crate) struct GoldPricesQuery {
    start_date: NaiveDate,
    end_date: NaiveDate,
}

#[derive(Serialize)]
pub(crate) struct GoldPricesResponse {
    data: Vec<GoldPriceResponse>,
}

#[derive(Serialize)]
struct GoldPriceResponse {
    date: String,
    price: f64,
}

#[derive(Serialize)]
pub(crate) struct SummaryRefreshError {
    error: &'static str,
}

pub(crate) async fn get_gold_prices<R>(
    State(state): State<AppState<R>>,
    query: Result<Query<GoldPricesQuery>, QueryRejection>,
) -> Result<Json<GoldPricesResponse>, ApiError>
where
    R: GoldPriceReader,
{
    let Query(query) = query.map_err(|_| ApiError::invalid_query())?;

    if query.start_date > query.end_date {
        return Err(ApiError::reversed_date_range());
    }

    let prices = state
        .repository()
        .daily_medians(query.start_date, query.end_date)
        .await
        .map_err(|repository_error| {
            error!(error = %repository_error, "failed to query daily gold prices");
            ApiError::internal()
        })?;

    Ok(Json(GoldPricesResponse {
        data: prices.into_iter().map(GoldPriceResponse::from).collect(),
    }))
}

pub(crate) async fn refresh_gold_price_summaries<R>(
    State(state): State<AppState<R>>,
) -> Result<Json<SummaryRefresh>, (StatusCode, Json<SummaryRefreshError>)>
where
    R: GoldPriceReader,
{
    let refresh =
        state
            .repository()
            .refresh_daily_summaries()
            .await
            .map_err(|repository_error| {
                error!(error = %repository_error, "failed to refresh daily gold price summaries");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(SummaryRefreshError {
                        error: "汇总数据失败，请重试",
                    }),
                )
            })?;

    Ok(Json(refresh))
}

impl From<DailyGoldPrice> for GoldPriceResponse {
    fn from(price: DailyGoldPrice) -> Self {
        Self {
            date: price.date.format("%Y-%m-%d").to_string(),
            price: price.price,
        }
    }
}
