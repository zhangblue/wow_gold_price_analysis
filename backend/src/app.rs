use crate::{api::gold_prices::get_gold_prices, repository::gold_prices::GoldPriceReader};
use axum::{
    http::{HeaderValue, Method},
    routing::get,
    Router,
};
use std::{path::PathBuf, sync::Arc};
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

pub struct AppState<R> {
    repository: Arc<R>,
}

impl<R> AppState<R> {
    pub fn repository(&self) -> &R {
        &self.repository
    }
}

impl<R> Clone for AppState<R> {
    fn clone(&self) -> Self {
        Self {
            repository: Arc::clone(&self.repository),
        }
    }
}

pub fn build_app<R>(repository: R, dist_dir: PathBuf, development_origin: HeaderValue) -> Router
where
    R: GoldPriceReader,
{
    let index_file = dist_dir.join("index.html");
    let static_files = ServeDir::new(dist_dir).not_found_service(ServeFile::new(index_file));
    let cors = CorsLayer::new()
        .allow_origin(development_origin)
        .allow_methods([Method::GET]);

    Router::new()
        .route("/api/gold-prices", get(get_gold_prices::<R>))
        .fallback_service(static_files)
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(AppState {
            repository: Arc::new(repository),
        })
}
