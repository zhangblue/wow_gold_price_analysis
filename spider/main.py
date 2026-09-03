"""Command-line entry point for the DD373 ratio crawler."""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime
from pathlib import Path
from tempfile import NamedTemporaryFile
from typing import Callable, Optional
from zoneinfo import ZoneInfo

from spider.fetcher import fetch_url
from spider.parser import parse_result_html


DEFAULT_URL = "https://www.dd373.com/s-aj0khw-0-1bcwm5-8rg681-0-0-tf85vg-0-0-0-0-0-1-0-0-1.html"
DEFAULT_OUTPUT = Path("spider/output/results.json")
SHANGHAI = ZoneInfo("Asia/Shanghai")


def crawl(
    url: str,
    fetch_html: Callable[[str], str] = fetch_url,
    captured_at: Optional[datetime] = None,
) -> list[dict[str, object]]:
    """Fetch a page and return its first ten parsed product ratio records."""
    records = parse_result_html(fetch_html(url), limit=10)
    timestamp = (captured_at or datetime.now(SHANGHAI)).astimezone(SHANGHAI).isoformat()
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
    return parser


def main() -> int:
    args = build_argument_parser().parse_args()
    try:
        records = crawl(args.url)
        if len(records) != 10:
            raise RuntimeError(f"仅解析到 {len(records)} 条有效比例记录，期望 10 条")
        write_records(records, args.output)
    except Exception as error:  # Keep CLI failures actionable without a traceback.
        print(f"抓取失败：{error}", file=sys.stderr)
        return 1

    print(f"已写入 {len(records)} 条记录到 {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
