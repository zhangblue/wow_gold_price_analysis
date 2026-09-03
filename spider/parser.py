"""Parse the requested DD373 ratio node from list-page HTML."""

from __future__ import annotations

import re
from html.parser import HTMLParser
from typing import List, Optional, Tuple


RatioRecord = Tuple[float, str]
RATIO_PATTERN = re.compile(r"^1(?:元|金)\s*=\s*(?P<ratio>\d+(?:\.\d+)?)\s*(?:金|元)$")
TARGET_PATTERN = re.compile(r"^1金\s*=\s*(?P<ratio>\d+(?:\.\d+)?)\s*元$")


def _normalise_text(text: str) -> str:
    return " ".join(text.split())


def parse_ratio_text(text: str) -> Optional[float]:
    """Return the number after ``=`` from a complete unit-ratio string."""
    match = RATIO_PATTERN.fullmatch(_normalise_text(text))
    return float(match.group("ratio")) if match else None


class _ResultCardParser(HTMLParser):
    """Collect ``.kucun``'s second paragraph from each product-card div."""

    def __init__(self, limit: int) -> None:
        super().__init__(convert_charrefs=True)
        self.limit = limit
        self.records: List[RatioRecord] = []
        self._card_depth = 0
        self._kucun_depth: Optional[int] = None
        self._paragraph_index = 0
        self._paragraph_parts: Optional[List[str]] = None
        self._target_text: Optional[str] = None

    def handle_starttag(self, tag: str, attrs: list[tuple[str, Optional[str]]]) -> None:
        classes = set((dict(attrs).get("class") or "").split())
        if self._card_depth == 0:
            if tag == "div" and "goods-list-item" in classes:
                self._card_depth = 1
                self._kucun_depth = None
                self._paragraph_index = 0
                self._target_text = None
            return

        self._card_depth += 1
        if tag == "div" and "kucun" in classes:
            self._kucun_depth = self._card_depth
            self._paragraph_index = 0
        elif tag == "p" and self._kucun_depth is not None:
            self._paragraph_index += 1
            if self._paragraph_index == 2:
                self._paragraph_parts = []

    def handle_endtag(self, tag: str) -> None:
        if self._card_depth == 0:
            return
        if tag == "p" and self._paragraph_parts is not None:
            self._target_text = _normalise_text("".join(self._paragraph_parts))
            self._paragraph_parts = None
        if self._kucun_depth == self._card_depth:
            self._kucun_depth = None
        self._card_depth -= 1
        if self._card_depth == 0:
            self._finish_card()

    def handle_data(self, data: str) -> None:
        if self._paragraph_parts is not None:
            self._paragraph_parts.append(data)

    def close(self) -> None:
        super().close()
        if self._card_depth:
            self._finish_card()
            self._card_depth = 0

    def _finish_card(self) -> None:
        if self._target_text is None or len(self.records) >= self.limit:
            return
        match = TARGET_PATTERN.fullmatch(self._target_text)
        if match:
            self.records.append((float(match.group("ratio")), self._target_text))


def parse_result_html(html: str, limit: int = 10) -> List[RatioRecord]:
    """Return the first ``limit`` values at the requested second-``p`` path."""
    parser = _ResultCardParser(limit=limit)
    parser.feed(html)
    parser.close()
    return parser.records
