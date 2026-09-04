import { afterEach, vi } from 'vitest';
import { getDailyGoldPrices, refreshDailyGoldPrices } from './goldPriceRepository';

afterEach(() => {
  vi.unstubAllGlobals();
});

test('requests the inclusive date range from the backend api', async () => {
  const fetchMock = vi.fn().mockResolvedValue(
    new Response(JSON.stringify({ data: [{ date: '2026-08-02', price: 0.0119 }] }), { status: 200 }),
  );
  vi.stubGlobal('fetch', fetchMock);

  await expect(getDailyGoldPrices('2026-08-02', '2026-08-02')).resolves.toEqual([
    { date: '2026-08-02', price: 0.0119 },
  ]);
  expect(fetchMock).toHaveBeenCalledWith('/api/gold-prices?start_date=2026-08-02&end_date=2026-08-02');
});

test('throws when the backend responds with an error', async () => {
  vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response('{"error":"bad request"}', { status: 400 })));

  await expect(getDailyGoldPrices('2026-08-02', '2026-08-02')).rejects.toThrow();
});

test('posts to the daily summary endpoint', async () => {
  const fetchMock = vi.fn().mockResolvedValue(
    new Response(JSON.stringify({ summary_count: 31, aggregated_at: '2026-09-04T10:30:00+08:00' }), { status: 200 }),
  );
  vi.stubGlobal('fetch', fetchMock);

  await expect(refreshDailyGoldPrices()).resolves.toEqual({
    summaryCount: 31,
    aggregatedAt: '2026-09-04T10:30:00+08:00',
  });
  expect(fetchMock).toHaveBeenCalledWith('/api/gold-prices/summary', { method: 'POST' });
});
