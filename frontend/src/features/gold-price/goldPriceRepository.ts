import { sampleDailyGoldPrices } from './sampleData';
import type { DailyGoldPrice } from './types';

export async function getDailyGoldPrices(
  startDate: string,
  endDate: string,
): Promise<DailyGoldPrice[]> {
  return sampleDailyGoldPrices.filter(
    (item) => item.date >= startDate && item.date <= endDate,
  );
}
