use chrono::NaiveDate;
use gold_price_backend::repository::gold_prices::{DailyGoldPrice, GoldPriceRepository};

#[test]
fn reads_date_range_from_daily_summary_table() {
    let start = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 8, 31).unwrap();
    let debug = format!(
        "{:?}",
        GoldPriceRepository::daily_median_statement(start, end)
    );

    assert!(debug.contains("daily_gold_price_summaries"));
    assert!(debug.contains("summary_date >= $1"));
    assert!(debug.contains("summary_date <= $2"));
    assert!(debug.contains("median_ratio::double precision AS price"));
}

#[test]
fn refresh_query_names_summary_date_before_ordering_it() {
    let debug = format!(
        "{:?}",
        GoldPriceRepository::refresh_daily_summaries_statement()
    );

    assert!(debug.contains("::date AS summary_date"), "{debug}");
    assert!(debug.contains("ORDER BY summary_date"), "{debug}");
}

#[test]
fn serializable_daily_price_keeps_its_date_and_price() {
    let item = DailyGoldPrice {
        date: NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        price: 0.0121,
    };
    assert_eq!(item.date.to_string(), "2026-08-01");
    assert_eq!(item.price, 0.0121);
}
