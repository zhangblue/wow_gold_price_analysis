import gzip
import unittest

from spider.fetcher import decode_response_body


class FetcherTests(unittest.TestCase):
    def test_decodes_gzip_encoded_html(self) -> None:
        body = gzip.compress("<html>1元=80.3859金</html>".encode("utf-8"))

        self.assertEqual(
            decode_response_body(body, content_encoding="gzip", charset="utf-8"),
            "<html>1元=80.3859金</html>",
        )


if __name__ == "__main__":
    unittest.main()
