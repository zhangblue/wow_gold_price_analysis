import { getPriceSummary, validateDateRange } from './priceMetrics';

test('rejects a reversed date range', () => {
  expect(validateDateRange('2026-08-31', '2026-08-01')).toBe('结束日期不能早于开始日期');
});

test('calculates latest, high, low, and day-over-day change', () => {
  expect(getPriceSummary([
    { date: '2026-08-01', price: 0.0120 },
    { date: '2026-08-02', price: 0.0117 },
    { date: '2026-08-03', price: 0.0124 },
  ])).toEqual({ latest: 0.0124, high: 0.0124, low: 0.0117, change: 0.0007, changePercent: 5.98 });
});
