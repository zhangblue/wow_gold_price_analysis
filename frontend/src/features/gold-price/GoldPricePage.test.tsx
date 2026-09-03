import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';
import App from '../../App';
import * as goldPriceRepository from './goldPriceRepository';

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
