import unittest
from datetime import datetime, timezone, timedelta

from spider.main import crawl


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

        self.assertEqual(
            records,
            [
                {
                    "rank": 1,
                    "ratio": 0.0124,
                    "raw_text": "1金=0.0124元",
                    "fetched_at": "2026-09-03T14:30:15+08:00",
                },
                {
                    "rank": 2,
                    "ratio": 0.0125,
                    "raw_text": "1金=0.0125元",
                    "fetched_at": "2026-09-03T14:30:15+08:00",
                },
            ],
        )


if __name__ == "__main__":
    unittest.main()
