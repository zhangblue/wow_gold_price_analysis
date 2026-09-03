import { render, screen } from '@testing-library/react';
import App from './App';

test('renders the gold price heading', () => {
  render(<App />);
  expect(screen.getByRole('heading', { name: '金币价格走势' })).toBeInTheDocument();
});
