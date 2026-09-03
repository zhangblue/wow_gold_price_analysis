import { render, screen } from '@testing-library/react';
import { PriceTrendChart } from './PriceTrendChart';

test('labels the chart and renders a left price axis', () => {
  render(<PriceTrendChart data={[{ date: '2026-08-01', price: 0.0124 }]} />);

  expect(screen.getByRole('img', { name: '金币日价格趋势图' })).toBeInTheDocument();
  expect(screen.getByTestId('price-y-axis-left')).toBeInTheDocument();
});
