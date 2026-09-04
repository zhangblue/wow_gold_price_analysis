"""Command-line entry point for the DD373 ratio crawler."""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
from datetime import datetime
from pathlib import Path
from tempfile import NamedTemporaryFile
from typing import Callable, Optional
from zoneinfo import ZoneInfo

from spider.fetcher import fetch_url
from spider.parser import parse_result_html
from spider.database import DatabaseRepository


DEFAULT_URL = "https://www.dd373.com/s-aj0khw-0-1bcwm5-8rg681-0-0-tf85vg-0-0-0-0-0-1-0-0-1.html"
DEFAULT_OUTPUT = Path("spider/output/results.json")
SHANGHAI = ZoneInfo("Asia/Shanghai")


def load_repository_environment(dotenv_path: Optional[Path] = None) -> None:
    """Load DATABASE_URL from the repository .env without overriding the environment."""
    path = dotenv_path or Path(__file__).resolve().parents[1] / ".env"
    if "DATABASE_URL" in os.environ or not path.is_file():
        return

    for line in path.read_text(encoding="utf-8").splitlines():
        key, separator, value = line.strip().partition("=")
        if key != "DATABASE_URL" or not separator:
            continue
        value = value.strip()
        if len(value) >= 2 and value[0] == value[-1] and value[0] in {"'", '"'}:
            value = value[1:-1]
        if value:
            os.environ["DATABASE_URL"] = value
        return


def interval_minutes(value: str) -> float:
    try:
        interval = float(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError("interval must be a number") from error
    if interval < 0:
        raise argparse.ArgumentTypeError("interval must be zero or greater")
    return interval


def run_scheduled(interval: float, run_once_fn, sleep_fn=time.sleep) -> None:
    while True:
        run_once_fn()
        if interval == 0:
            return
        sleep_fn(interval * 60)


def crawl(
    url: str,
    fetch_html: Callable[[str], str] = fetch_url,
    captured_at: Optional[datetime] = None,
) -> list[dict[str, object]]:
    """Fetch a page and return its first ten parsed product ratio records."""
    records = parse_result_html(fetch_html(url), limit=10)
    timestamp = (captured_at or datetime.now(SHANGHAI)).astimezone(SHANGHAI).replace(microsecond=0).isoformat()
    return [
        {
            "rank": rank,
            "ratio": ratio,
            "raw_text": raw_text,
            "fetched_at": timestamp,
        }
        for rank, (ratio, raw_text) in enumerate(records, start=1)
    ]


def write_records(records: list[dict[str, object]], output: Path) -> None:
    """Atomically write records as UTF-8 JSON."""
    output.parent.mkdir(parents=True, exist_ok=True)
    with NamedTemporaryFile("w", encoding="utf-8", dir=output.parent, delete=False) as temporary:
        json.dump(records, temporary, ensure_ascii=False, indent=2)
        temporary.write("\n")
        temporary_path = Path(temporary.name)
    temporary_path.replace(output)


def build_argument_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="抓取 DD373 列表页前 10 条金币比例")
    parser.add_argument("--url", default=DEFAULT_URL, help="DD373 商品列表页 URL")
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT, help="JSON 输出文件")
    parser.add_argument("--interval-minutes", type=interval_minutes, default=0, help="运行间隔分钟数；0 表示只运行一次")
    return parser


def main() -> int:
    args = build_argument_parser().parse_args()
    load_repository_environment()
    repository = DatabaseRepository.from_environment()
    def run_once():
        started_at = datetime.now(SHANGHAI)
        try:
            records = crawl(args.url)
            if len(records) != 10:
                raise RuntimeError(f"仅解析到 {len(records)} 条有效比例记录，期望 10 条")
            repository.save_success(args.url, started_at, records)
            write_records(records, args.output)
            print(f"已写入 {len(records)} 条记录到 {args.output}")
        except Exception as error:
            try:
                repository.save_failure(args.url, started_at, str(error))
            except Exception:
                pass
            print(f"抓取失败：{error}", file=sys.stderr)
    try:
        run_scheduled(args.interval_minutes, run_once)
    except KeyboardInterrupt:
        return 0
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
