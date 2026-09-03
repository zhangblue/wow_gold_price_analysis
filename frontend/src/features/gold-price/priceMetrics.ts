import type { DailyGoldPrice } from './types';

export type PriceSummary = {
  latest: number;
  high: number;
  low: number;
  change: number;
  changePercent: number;
};

export function validateDateRange(startDate: string, endDate: string): string | null {
  if (!startDate || !endDate) return '请选择开始日期和结束日期';
  return startDate > endDate ? '结束日期不能早于开始日期' : null;
}

export function getPriceSummary(prices: DailyGoldPrice[]): PriceSummary {
  const latest = prices[prices.length - 1].price;
  const high = Math.max(...prices.map((item) => item.price));
  const low = Math.min(...prices.map((item) => item.price));
  const previous = prices.length > 1 ? prices[prices.length - 2].price : latest;
  const change = latest - previous;
  const changePercent = previous === 0
    ? 0
    : Number(((change / previous) * 100).toFixed(2));

  return { latest, high, low, change, changePercent };
}
