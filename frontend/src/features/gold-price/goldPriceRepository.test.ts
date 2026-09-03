import { getDailyGoldPrices } from './goldPriceRepository';

test('returns ascending records inside the inclusive date range', async () => {
  await expect(getDailyGoldPrices('2026-08-02', '2026-08-03')).resolves.toEqual([
    { date: '2026-08-02', price: 0.0119 },
    { date: '2026-08-03', price: 0.0117 },
  ]);
});
