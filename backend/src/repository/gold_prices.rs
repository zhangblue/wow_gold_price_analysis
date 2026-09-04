use chrono::{DateTime, FixedOffset, NaiveDate};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, FromQueryResult, Statement, TransactionTrait,
};
use serde::Serialize;
use std::{fmt, future::Future};

const DAILY_MEDIAN_SQL: &str = r#"
SELECT summary_date AS date, median_ratio::double precision AS price
FROM daily_gold_price_summaries
WHERE summary_date >= $1 AND summary_date <= $2
ORDER BY summary_date ASC
"#;

const TRUNCATE_DAILY_SUMMARIES_SQL: &str = "TRUNCATE TABLE daily_gold_price_summaries;";

const REFRESH_DAILY_SUMMARIES_SQL: &str = r#"
INSERT INTO daily_gold_price_summaries (summary_date, median_ratio, source_record_count, aggregated_at)
SELECT (fetched_at AT TIME ZONE 'Asia/Shanghai')::date,
       percentile_cont(0.5) WITHIN GROUP (ORDER BY ratio::double precision)::numeric(20, 10),
       COUNT(*)::integer, now()
FROM gold_price_records
GROUP BY (fetched_at AT TIME ZONE 'Asia/Shanghai')::date
ORDER BY summary_date
RETURNING aggregated_at;
"#;

const CURRENT_TIMESTAMP_SQL: &str = "SELECT now() AS aggregated_at;";

#[derive(Debug, Clone, PartialEq, Serialize, FromQueryResult)]
pub struct DailyGoldPrice {
    pub date: NaiveDate,
    pub price: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SummaryRefresh {
    pub summary_count: usize,
    pub aggregated_at: String,
}

pub struct GoldPriceRepository {
    db: DatabaseConnection,
}

#[derive(Debug, Clone, Copy)]
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

    fn refresh_daily_summaries(
        &self,
    ) -> impl Future<Output = Result<SummaryRefresh, RepositoryError>> + Send;
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

    pub async fn refresh_daily_summaries(&self) -> Result<SummaryRefresh, RepositoryError> {
        let transaction = self.db.begin().await.map_err(|_| RepositoryError::Query)?;

        transaction
            .execute(Statement::from_string(
                DbBackend::Postgres,
                TRUNCATE_DAILY_SUMMARIES_SQL,
            ))
            .await
            .map_err(|_| RepositoryError::Query)?;

        let rows = transaction
            .query_all(Statement::from_string(
                DbBackend::Postgres,
                REFRESH_DAILY_SUMMARIES_SQL,
            ))
            .await
            .map_err(|_| RepositoryError::Query)?;

        let aggregated_at = match rows.first() {
            Some(row) => row
                .try_get::<DateTime<FixedOffset>>("", "aggregated_at")
                .map_err(|_| RepositoryError::Query)?,
            None => transaction
                .query_one(Statement::from_string(
                    DbBackend::Postgres,
                    CURRENT_TIMESTAMP_SQL,
                ))
                .await
                .map_err(|_| RepositoryError::Query)?
                .ok_or(RepositoryError::Query)?
                .try_get::<DateTime<FixedOffset>>("", "aggregated_at")
                .map_err(|_| RepositoryError::Query)?,
        };

        let refresh = SummaryRefresh {
            summary_count: rows.len(),
            aggregated_at: aggregated_at.to_rfc3339(),
        };

        transaction
            .commit()
            .await
            .map_err(|_| RepositoryError::Query)?;

        Ok(refresh)
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

    fn refresh_daily_summaries(
        &self,
    ) -> impl Future<Output = Result<SummaryRefresh, RepositoryError>> + Send {
        GoldPriceRepository::refresh_daily_summaries(self)
    }
}
