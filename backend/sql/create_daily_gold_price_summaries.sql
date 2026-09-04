CREATE TABLE IF NOT EXISTS daily_gold_price_summaries (
    summary_date date PRIMARY KEY,
    median_ratio numeric(20, 10) NOT NULL,
    source_record_count integer NOT NULL CHECK (source_record_count > 0),
    aggregated_at timestamptz NOT NULL DEFAULT now()
);
