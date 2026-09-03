"""Read-only HTTP fetching for the DD373 list page."""

from __future__ import annotations

import gzip
from urllib.request import Request, urlopen


USER_AGENT = "dd373-ratio-spider/1.0 (+local data collection)"


def decode_response_body(body: bytes, content_encoding: str | None, charset: str) -> str:
    """Decode an HTTP response body, including standard gzip compression."""
    if content_encoding and "gzip" in content_encoding.lower():
        body = gzip.decompress(body)
    return body.decode(charset, errors="replace")


def fetch_url(url: str, timeout: float = 15.0) -> str:
    """Download one public HTML page and return decoded text."""
    request = Request(
        url,
        headers={
            "User-Agent": USER_AGENT,
            "Accept": "text/html,application/xhtml+xml",
            "Accept-Encoding": "gzip",
        },
    )
    with urlopen(request, timeout=timeout) as response:
        charset = response.headers.get_content_charset() or "utf-8"
        return decode_response_body(
            response.read(),
            content_encoding=response.headers.get("Content-Encoding"),
            charset=charset,
        )
