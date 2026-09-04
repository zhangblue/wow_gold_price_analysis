"""PostgreSQL persistence for crawler results; no schema changes are performed."""
import os
from datetime import datetime
from zoneinfo import ZoneInfo


SHANGHAI = ZoneInfo("Asia/Shanghai")


def normalize_storage_timestamp(value):
    """Return a timezone-aware Shanghai timestamp with second precision."""
    if isinstance(value, str):
        value = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if value.tzinfo is None:
        value = value.replace(tzinfo=SHANGHAI)
    else:
        value = value.astimezone(SHANGHAI)
    return value.replace(microsecond=0)


class DatabaseConfigurationError(RuntimeError):
    pass


class DatabaseRepository:
    def __init__(self, database_url, connector=None):
        self.database_url = database_url
        self.connector = connector or self._connect

    @classmethod
    def from_environment(cls):
        url = os.environ.get("DATABASE_URL")
        if not url:
            raise DatabaseConfigurationError("DATABASE_URL is required")
        return cls(url)

    def _connect(self, url):
        import psycopg
        return psycopg.connect(url)

    def save_success(self, source_url, started_at, records):
        if len(records) != 10:
            raise ValueError("exactly 10 records are required")
        started_at = normalize_storage_timestamp(started_at)
        finished_at = normalize_storage_timestamp(datetime.now(SHANGHAI))
        with self.connector(self.database_url) as connection:
            with connection.transaction():
                with connection.cursor() as cursor:
                    cursor.execute(
                        "INSERT INTO crawl_runs (source_url, started_at, finished_at, status, record_count, error_message) VALUES (%s, %s, %s, %s, %s, %s) RETURNING id",
                        (source_url, started_at, finished_at, "success", 10, None),
                    )
                    run_id = cursor.fetchone()[0]
                    cursor.executemany(
                        "INSERT INTO gold_price_records (crawl_run_id, rank, ratio, raw_text, fetched_at) VALUES (%s, %s, %s, %s, %s)",
                        [
                            (
                                run_id,
                                item["rank"],
                                item["ratio"],
                                item["raw_text"],
                                normalize_storage_timestamp(item["fetched_at"]),
                            )
                            for item in records
                        ],
                    )

    def save_failure(self, source_url, started_at, error_message):
        started_at = normalize_storage_timestamp(started_at)
        with self.connector(self.database_url) as connection:
            with connection.transaction():
                with connection.cursor() as cursor:
                    cursor.execute(
                        "INSERT INTO crawl_runs (source_url, started_at, finished_at, status, record_count, error_message) VALUES (%s, %s, %s, %s, %s, %s)",
                        (source_url, started_at, normalize_storage_timestamp(datetime.now(SHANGHAI)), "failed", 0, str(error_message)),
                    )
