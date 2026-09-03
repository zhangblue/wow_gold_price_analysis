import { useState } from 'react';
import { getDailyGoldPrices } from './goldPriceRepository';
import { validateDateRange } from './priceMetrics';
import { DateRangeFilter } from './DateRangeFilter';
import { PriceSummary } from './PriceSummary';
import type { DailyGoldPrice } from './types';

type LoadState = 'idle' | 'loading' | 'success' | 'empty' | 'error';

export function GoldPricePage() {
  const [startDate, setStartDate] = useState('2026-08-01');
  const [endDate, setEndDate] = useState('2026-08-31');
  const [loadState, setLoadState] = useState<LoadState>('idle');
  const [prices, setPrices] = useState<DailyGoldPrice[]>([]);
  const [validationMessage, setValidationMessage] = useState<string | null>(null);

  async function queryPrices() {
    const validationError = validateDateRange(startDate, endDate);
    if (validationError) {
      setValidationMessage(validationError);
      return;
    }

    setValidationMessage(null);
    setLoadState('loading');

    try {
      const result = await getDailyGoldPrices(startDate, endDate);
      setPrices(result);
      setLoadState(result.length > 0 ? 'success' : 'empty');
    } catch {
      setLoadState('error');
    }
  }

  return (
    <main>
      <h1>金币价格走势</h1>
      <DateRangeFilter
        startDate={startDate}
        endDate={endDate}
        isLoading={loadState === 'loading'}
        onStartDateChange={setStartDate}
        onEndDateChange={setEndDate}
        onSubmit={queryPrices}
      />
      {validationMessage && <p role="alert">{validationMessage}</p>}
      <section aria-label="最新价格">
        <h2>最新价格</h2>
        {loadState === 'loading' && <p>正在加载价格数据…</p>}
        {loadState === 'empty' && <p>该时间范围暂无价格数据</p>}
        {loadState === 'error' && <p role="alert">加载价格数据失败，请重试</p>}
        {loadState === 'success' && prices.length > 0 && <PriceSummary prices={prices} />}
      </section>
    </main>
  );
}
