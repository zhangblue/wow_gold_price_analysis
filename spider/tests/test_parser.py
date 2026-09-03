import unittest

from spider.parser import parse_ratio_text, parse_result_html


def product_card(index: int, ratio: str) -> str:
    return f"""
    <div class=\"goods-list-item\">
      <div class=\"kucun\"><div>
        <p>1元={ratio}金</p>
        <p>1金=0.0124元</p>
      </div></div>
    </div>
    """


def xpath_target_card(index: int, first_paragraph: str, target_paragraph: str) -> str:
    return f"""
    <div class=\"goods-list-item\">
      <div class=\"border-box\">
        <div class=\"kucun\">
          <div>
            <p>{first_paragraph}</p>
            <p>{target_paragraph}</p>
          </div>
        </div>
      </div>
    </div>
    """


class RatioParserTests(unittest.TestCase):
    def test_extracts_number_after_equals_sign(self) -> None:
        self.assertEqual(parse_ratio_text("1元=80.3859金"), 80.3859)

    def test_ignores_non_ratio_text(self) -> None:
        self.assertIsNone(parse_ratio_text("库存：100000金"))

    def test_returns_first_ten_product_ratios_in_page_order(self) -> None:
        html = "<main>" + "".join(
            product_card(index, f"80.{index}") for index in range(1, 12)
        ) + "</main>"

        records = parse_result_html(html)

        self.assertEqual(len(records), 10)
        self.assertEqual(records[0], (0.0124, "1金=0.0124元"))
        self.assertEqual(records[-1], (0.0124, "1金=0.0124元"))

    def test_skips_product_cards_without_a_valid_ratio(self) -> None:
        html = xpath_target_card(1, "1元=80.1金", "1金=0.0124元") + xpath_target_card(
            2, "1元=79.9金", "格式错误"
        )

        self.assertEqual(parse_result_html(html), [(0.0124, "1金=0.0124元")])

    def test_extracts_the_second_kucun_paragraph_from_each_product_card(self) -> None:
        html = "<div class=\"good-list-box\">" + "".join(
            xpath_target_card(index, f"1元={80 + index / 10}金", "1金=0.0124元")
            for index in range(1, 12)
        ) + "</div>"

        records = parse_result_html(html)

        self.assertEqual(len(records), 10)
        self.assertEqual(records[0], (0.0124, "1金=0.0124元"))
        self.assertEqual(records[-1], (0.0124, "1金=0.0124元"))


if __name__ == "__main__":
    unittest.main()
