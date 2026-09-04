use chrono::NaiveDate;
use sea_orm::{DatabaseConnection, DbBackend, FromQueryResult, Statement};
use serde::Serialize;
use std::{fmt, future::Future};

const DAILY_MEDIAN_SQL: &str = r#"
SELECT
  (fetched_at AT TIME ZONE 'Asia/Shanghai')::date AS date,
  percentile_cont(0.5) WITHIN GROUP (ORDER BY ratio::double precision) AS price
FROM gold_price_records
WHERE (fetched_at AT TIME ZONE 'Asia/Shanghai')::date >= $1
  AND (fetched_at AT TIME ZONE 'Asia/Shanghai')::date <= $2
GROUP BY (fetched_at AT TIME ZONE 'Asia/Shanghai')::date
ORDER BY date ASC
"#;

#[derive(Debug, Clone, PartialEq, Serialize, FromQueryResult)]
pub struct DailyGoldPrice {
    pub date: NaiveDate,
    pub price: f64,
}

pub struct GoldPriceRepository {
    db: DatabaseConnection,
}

#[derive(Debug)]
pub enum RepositoryError {
    Query,
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Query => formatter.write_str("database query failed"),
        }
    }
}

impl std::error::Error for RepositoryError {}

pub trait GoldPriceReader: Send + Sync + 'static {
    fn daily_medians(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> impl Future<Output = Result<Vec<DailyGoldPrice>, RepositoryError>> + Send;
}

impl GoldPriceRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn daily_median_statement(start: NaiveDate, end: NaiveDate) -> Statement {
        Statement::from_sql_and_values(
            DbBackend::Postgres,
            DAILY_MEDIAN_SQL,
            [start.into(), end.into()],
        )
    }

    pub async fn daily_medians(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<DailyGoldPrice>, RepositoryError> {
        DailyGoldPrice::find_by_statement(Self::daily_median_statement(start, end))
            .all(&self.db)
            .await
            .map_err(|_| RepositoryError::Query)
    }
}

impl GoldPriceReader for GoldPriceRepository {
    fn daily_medians(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> impl Future<Output = Result<Vec<DailyGoldPrice>, RepositoryError>> + Send {
        GoldPriceRepository::daily_medians(self, start, end)
    }
}
