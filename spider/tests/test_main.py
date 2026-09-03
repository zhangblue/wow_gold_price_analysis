import os
import unittest
from datetime import datetime, timezone, timedelta
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from spider.main import crawl, interval_minutes, load_repository_environment, run_scheduled


FIXTURE = """
<main>
  <div class="goods-list-item"><div class="kucun"><div>
    <p>1元=80.3859金</p><p>1金=0.0124元</p>
  </div></div></div>
  <div class="goods-list-item"><div class="kucun"><div>
    <p>1元=79.9金</p><p>1金=0.0125元</p>
  </div></div></div>
</main>
"""


class CrawlTests(unittest.TestCase):
    def test_adds_the_same_supplied_timestamp_to_every_record(self) -> None:
        captured_at = datetime(2026, 9, 3, 14, 30, 15, tzinfo=timezone(timedelta(hours=8)))

        records = crawl(
            "https://example.test/list",
            fetch_html=lambda _: FIXTURE,
            captured_at=captured_at,
        )
        self.assertEqual(records, [{"rank": 1, "ratio": 0.0124, "raw_text": "1金=0.0124元", "fetched_at": "2026-09-03T14:30:15+08:00"}, {"rank": 2, "ratio": 0.0125, "raw_text": "1金=0.0125元", "fetched_at": "2026-09-03T14:30:15+08:00"}])


class SchedulerTests(unittest.TestCase):
    def test_zero_runs_once_without_sleeping(self) -> None:
        calls = []
        run_scheduled(0, run_once_fn=lambda: calls.append("run"), sleep_fn=lambda _: self.fail("slept"))
        self.assertEqual(calls, ["run"])

    def test_positive_interval_sleeps_in_seconds_between_runs(self) -> None:
        calls = []
        def stop_after_first(seconds):
            self.assertEqual(seconds, 150.0)
            raise KeyboardInterrupt
        with self.assertRaises(KeyboardInterrupt):
            run_scheduled(2.5, run_once_fn=lambda: calls.append("run"), sleep_fn=stop_after_first)
        self.assertEqual(calls, ["run"])

    def test_negative_interval_is_rejected(self) -> None:
        with self.assertRaises(Exception):
            interval_minutes("-1")


class EnvironmentLoadingTests(unittest.TestCase):
    def test_loads_database_url_from_repository_dotenv_when_not_in_environment(self) -> None:
        with TemporaryDirectory() as temporary_directory:
            dotenv_path = Path(temporary_directory) / ".env"
            dotenv_path.write_text("# local configuration\nDATABASE_URL=postgresql://dotenv.example/test\n", encoding="utf-8")

            with patch.dict("os.environ", {}, clear=True):
                load_repository_environment(dotenv_path)
                actual = os.environ["DATABASE_URL"]

            self.assertEqual(actual, "postgresql://dotenv.example/test")

    def test_preserves_database_url_already_set_in_environment(self) -> None:
        with TemporaryDirectory() as temporary_directory:
            dotenv_path = Path(temporary_directory) / ".env"
            dotenv_path.write_text("DATABASE_URL=postgresql://dotenv.example/test\n", encoding="utf-8")

            with patch.dict("os.environ", {"DATABASE_URL": "postgresql://environment.example/test"}, clear=True):
                load_repository_environment(dotenv_path)
                actual = os.environ["DATABASE_URL"]

            self.assertEqual(actual, "postgresql://environment.example/test")



if __name__ == "__main__":
    unittest.main()
