import { getPriceSummary } from './priceMetrics';
import type { DailyGoldPrice } from './types';

type PriceSummaryProps = {
  prices: DailyGoldPrice[];
};

export function PriceSummary({ prices }: PriceSummaryProps) {
  const summary = getPriceSummary(prices);

  return (
    <section aria-label="价格摘要">
      <p>最新价格：{summary.latest}</p>
      <p>最高价格：{summary.high}</p>
      <p>最低价格：{summary.low}</p>
      <p>涨跌：{summary.change} ({summary.changePercent}%)</p>
    </section>
  );
}
