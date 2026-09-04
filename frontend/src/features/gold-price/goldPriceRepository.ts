import type { DailyGoldPrice } from './types';

export async function getDailyGoldPrices(
  startDate: string,
  endDate: string,
): Promise<DailyGoldPrice[]> {
  const params = new URLSearchParams({
    start_date: startDate,
    end_date: endDate,
  });
  const response = await fetch(`/api/gold-prices?${params.toString()}`);

  if (!response.ok) {
    throw new Error(`Failed to load gold prices: ${response.status}`);
  }

  const body: { data: DailyGoldPrice[] } = await response.json();
  return body.data;
}
