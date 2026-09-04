use chrono::NaiveDate;
use gold_price_backend::repository::gold_prices::{DailyGoldPrice, GoldPriceRepository};

#[test]
fn builds_a_bounded_daily_median_query() {
    let query = GoldPriceRepository::daily_median_statement(
        NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 8, 31).unwrap(),
    );
    let debug = format!("{query:?}");

    assert!(debug.contains("percentile_cont(0.5)"));
    assert!(debug.contains("Asia/Shanghai"));
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
