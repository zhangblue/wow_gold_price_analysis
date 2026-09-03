-- Run this file once against the target PostgreSQL database before starting the crawler.
-- The crawler only inserts into these tables; it never runs this file or any DDL.

CREATE TABLE IF NOT EXISTS crawl_runs (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    source_url text NOT NULL,
    started_at timestamptz NOT NULL,
    finished_at timestamptz,
    status text NOT NULL CHECK (status IN ('success', 'failed')),
    record_count smallint NOT NULL DEFAULT 0 CHECK (record_count BETWEEN 0 AND 10),
    error_message text,
    CHECK (
        (status = 'success' AND record_count = 10 AND error_message IS NULL)
        OR
        (status = 'failed' AND record_count = 0 AND error_message IS NOT NULL)
    )
);

CREATE TABLE IF NOT EXISTS gold_price_records (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    crawl_run_id bigint NOT NULL REFERENCES crawl_runs(id) ON DELETE CASCADE,
    rank smallint NOT NULL CHECK (rank BETWEEN 1 AND 10),
    ratio numeric(12, 8) NOT NULL,
    raw_text text NOT NULL,
    fetched_at timestamptz NOT NULL,
    UNIQUE (crawl_run_id, rank)
);

CREATE INDEX IF NOT EXISTS gold_price_records_fetched_at_idx
    ON gold_price_records (fetched_at);
