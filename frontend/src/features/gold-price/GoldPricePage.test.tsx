import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, vi } from 'vitest';
import App from '../../App';
import * as goldPriceRepository from './goldPriceRepository';

beforeEach(() => {
  vi.stubGlobal(
    'fetch',
    vi.fn((request: RequestInfo | URL) => {
      const url = new URL(request.toString(), 'http://localhost');
      const data = url.searchParams.get('start_date') === '2026-09-01'
        ? []
        : [{ date: '2026-08-31', price: 0.0142 }];
      return Promise.resolve(new Response(JSON.stringify({ data }), { status: 200 }));
    }),
  );
});

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

async function enterDateRange(
  user: ReturnType<typeof userEvent.setup>,
  startDateValue: string,
  endDateValue: string,
) {
  const startDate = screen.getByLabelText('开始日期');
  const endDate = screen.getByLabelText('结束日期');
  await user.clear(startDate);
  await user.type(startDate, startDateValue);
  await user.clear(endDate);
  await user.type(endDate, endDateValue);
}

test('shows a validation message instead of loading reversed dates', async () => {
  const user = userEvent.setup();
  render(<App />);

  await enterDateRange(user, '2026-08-31', '2026-08-01');
  await user.click(screen.getByRole('button', { name: '查询价格' }));

  expect(await screen.findByText('结束日期不能早于开始日期')).toBeInTheDocument();
});

test('shows the latest price heading before a query', () => {
  render(<App />);

  expect(screen.getByRole('heading', { name: '最新价格' })).toBeInTheDocument();
});

test('shows the calculated summary after a query returns prices', async () => {
  const user = userEvent.setup();
  render(<App />);

  await user.click(screen.getByRole('button', { name: '查询价格' }));

  expect(await screen.findByText('最新价格：0.0142')).toBeInTheDocument();
  expect(screen.getByRole('img', { name: '金币日价格趋势图' })).toBeInTheDocument();
});

test('clears a successful summary when a later query has invalid dates', async () => {
  const user = userEvent.setup();
  render(<App />);

  await user.click(screen.getByRole('button', { name: '查询价格' }));
  expect(await screen.findByText('最新价格：0.0142')).toBeInTheDocument();

  await enterDateRange(user, '2026-08-31', '2026-08-01');
  await user.click(screen.getByRole('button', { name: '查询价格' }));

  expect(await screen.findByText('结束日期不能早于开始日期')).toBeInTheDocument();
  expect(screen.queryByText('最新价格：0.0142')).not.toBeInTheDocument();
});

test('clears a successful summary when a later query omits a date', async () => {
  const user = userEvent.setup();
  render(<App />);

  await user.click(screen.getByRole('button', { name: '查询价格' }));
  expect(await screen.findByText('最新价格：0.0142')).toBeInTheDocument();

  await user.clear(screen.getByLabelText('开始日期'));
  await user.click(screen.getByRole('button', { name: '查询价格' }));

  expect(await screen.findByText('请选择开始日期和结束日期')).toBeInTheDocument();
  expect(screen.queryByText('最新价格：0.0142')).not.toBeInTheDocument();
});

test('shows an empty-state message when a query returns no prices', async () => {
  const user = userEvent.setup();
  render(<App />);

  await enterDateRange(user, '2026-09-01', '2026-09-02');
  await user.click(screen.getByRole('button', { name: '查询价格' }));

  expect(await screen.findByText('该时间范围暂无价格数据')).toBeInTheDocument();
});

test('shows a retry message when the repository query fails', async () => {
  const user = userEvent.setup();
  vi.spyOn(goldPriceRepository, 'getDailyGoldPrices').mockRejectedValueOnce(new Error('network unavailable'));
  render(<App />);

  await user.click(screen.getByRole('button', { name: '查询价格' }));

  expect(await screen.findByText('加载价格数据失败，请重试')).toBeInTheDocument();
});

test('reloads the current dates after a successful summary', async () => {
  const user = userEvent.setup();
  const fetchMock = vi.fn()
    .mockResolvedValueOnce(new Response(JSON.stringify({ summary_count: 2, aggregated_at: '2026-09-04T10:30:00+08:00' })))
    .mockResolvedValueOnce(new Response(JSON.stringify({ data: [{ date: '2026-08-01', price: 0.0121 }] })));
  vi.stubGlobal('fetch', fetchMock);
  render(<App />);

  await user.click(screen.getByRole('button', { name: '汇总数据' }));

  expect(await screen.findByText('汇总完成，共处理 2 天数据')).toBeInTheDocument();
  expect(await screen.findByText('最新价格：0.0121')).toBeInTheDocument();
  expect(fetchMock).toHaveBeenNthCalledWith(2, '/api/gold-prices?start_date=2026-08-01&end_date=2026-08-31');
});

test('disables both actions while a summary is in progress', async () => {
  const user = userEvent.setup();
  let resolveSummary: (value: Response) => void;
  vi.stubGlobal('fetch', vi.fn().mockImplementation(() => new Promise<Response>((resolve) => {
    resolveSummary = resolve;
  })));
  render(<App />);

  await user.click(screen.getByRole('button', { name: '汇总数据' }));

  expect(screen.getByRole('button', { name: '汇总中…' })).toBeDisabled();
  expect(screen.getByRole('button', { name: '查询价格' })).toBeDisabled();
  resolveSummary!(new Response(JSON.stringify({ summary_count: 1, aggregated_at: '2026-09-04T10:30:00+08:00' })));
});

test('keeps the existing chart when a summary fails', async () => {
  const user = userEvent.setup();
  render(<App />);
  await user.click(screen.getByRole('button', { name: '查询价格' }));
  expect(await screen.findByRole('img', { name: '金币日价格趋势图' })).toBeInTheDocument();

  vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response('failure', { status: 500 })));
  await user.click(screen.getByRole('button', { name: '汇总数据' }));

  expect(await screen.findByText('汇总数据失败，请重试')).toBeInTheDocument();
  expect(screen.getByRole('img', { name: '金币日价格趋势图' })).toBeInTheDocument();
});
