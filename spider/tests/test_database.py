import os
import unittest
from datetime import datetime, timezone

from spider.database import DatabaseConfigurationError, DatabaseRepository


class Cursor:
    def __init__(self): self.executed = []; self.many = []
    def __enter__(self): return self
    def __exit__(self, *args): return False
    def execute(self, sql, params): self.executed.append((sql, params))
    def executemany(self, sql, params): self.many.append((sql, list(params)))
    def fetchone(self): return (42,)

class Connection:
    def __init__(self): self.cursor_instance = Cursor()
    def __enter__(self): return self
    def __exit__(self, *args): return False
    def transaction(self): return self
    def cursor(self): return self.cursor_instance

class DatabaseRepositoryTests(unittest.TestCase):
    def test_saves_one_run_and_ten_ranked_records_atomically(self):
        connection = Connection()
        repository = DatabaseRepository("postgresql://example", connector=lambda _: connection)
        timestamp = datetime(2026, 9, 3, tzinfo=timezone.utc)
        records = [{"rank": i, "ratio": 0.0124, "raw_text": "1金=0.0124元", "fetched_at": timestamp.isoformat()} for i in range(1, 11)]

        repository.save_success("https://example.test", timestamp, records)

        self.assertIn("INSERT INTO crawl_runs", connection.cursor_instance.executed[0][0])
        self.assertEqual(connection.cursor_instance.executed[0][1][-3:-1], ("success", 10))
        self.assertEqual(len(connection.cursor_instance.many[0][1]), 10)
        self.assertEqual(connection.cursor_instance.many[0][1][0][1], 1)

    def test_requires_database_url_from_environment(self):
        original = os.environ.pop("DATABASE_URL", None)
        try:
            with self.assertRaises(DatabaseConfigurationError):
                DatabaseRepository.from_environment()
        finally:
            if original is not None: os.environ["DATABASE_URL"] = original
