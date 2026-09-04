import { useState } from 'react';
import { getDailyGoldPrices, refreshDailyGoldPrices } from './goldPriceRepository';
import { validateDateRange } from './priceMetrics';
import { DateRangeFilter } from './DateRangeFilter';
import { PriceSummary } from './PriceSummary';
import { PriceTrendChart } from './PriceTrendChart';
import type { DailyGoldPrice } from './types';

type LoadState = 'idle' | 'loading' | 'success' | 'empty' | 'error';

function formatDate(date: Date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

function getCurrentMonthRange(now = new Date()) {
  const firstDay = new Date(now.getFullYear(), now.getMonth(), 1);
  const lastDay = new Date(now.getFullYear(), now.getMonth() + 1, 0);
  return {
    startDate: formatDate(firstDay),
    endDate: formatDate(lastDay),
  };
}

export function GoldPricePage() {
  const [{ startDate: initialStartDate, endDate: initialEndDate }] = useState(getCurrentMonthRange);
  const [startDate, setStartDate] = useState(initialStartDate);
  const [endDate, setEndDate] = useState(initialEndDate);
  const [loadState, setLoadState] = useState<LoadState>('idle');
  const [prices, setPrices] = useState<DailyGoldPrice[]>([]);
  const [validationMessage, setValidationMessage] = useState<string | null>(null);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [refreshMessage, setRefreshMessage] = useState<string | null>(null);

  async function queryPrices() {
    const validationError = validateDateRange(startDate, endDate);
    if (validationError) {
      setPrices([]);
      setLoadState('idle');
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

  async function refreshPrices() {
    setIsRefreshing(true);
    setRefreshMessage(null);

    try {
      const { summaryCount } = await refreshDailyGoldPrices();
      setRefreshMessage(`汇总完成，共处理 ${summaryCount} 天数据`);
      await queryPrices();
    } catch {
      setRefreshMessage('汇总数据失败，请重试');
    } finally {
      setIsRefreshing(false);
    }
  }

  return (
    <main className="gold-price-page">
      <h1>金币价格走势</h1>
      <DateRangeFilter
        startDate={startDate}
        endDate={endDate}
        isLoading={loadState === 'loading'}
        isRefreshing={isRefreshing}
        onStartDateChange={setStartDate}
        onEndDateChange={setEndDate}
        onSubmit={queryPrices}
        onRefresh={refreshPrices}
      />
      {validationMessage && <p role="alert">{validationMessage}</p>}
      {refreshMessage && <p role={refreshMessage === '汇总数据失败，请重试' ? 'alert' : undefined}>{refreshMessage}</p>}
      <section aria-label="最新价格" className="gold-price-page__results">
        <h2>最新价格</h2>
        {loadState === 'loading' && <p>正在加载价格数据…</p>}
        {loadState === 'empty' && <p>该时间范围暂无价格数据</p>}
        {loadState === 'error' && <p role="alert">加载价格数据失败，请重试</p>}
        {loadState === 'success' && prices.length > 0 && (
          <>
            <PriceSummary prices={prices} />
            <section aria-label="价格趋势" className="gold-price-page__trend">
              <h2>价格趋势</h2>
              <PriceTrendChart data={prices} />
            </section>
          </>
        )}
      </section>
    </main>
  );
}
