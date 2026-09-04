use crate::{
    api::error::ApiError,
    app::AppState,
    repository::gold_prices::{DailyGoldPrice, GoldPriceReader},
};
use axum::{
    extract::{rejection::QueryRejection, Query, State},
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

impl From<DailyGoldPrice> for GoldPriceResponse {
    fn from(price: DailyGoldPrice) -> Self {
        Self {
            date: price.date.format("%Y-%m-%d").to_string(),
            price: price.price,
        }
    }
}
