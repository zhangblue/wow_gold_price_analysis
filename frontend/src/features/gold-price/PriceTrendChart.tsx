import {
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts';
import type { DailyGoldPrice } from './types';

type PriceTrendChartProps = {
  data: DailyGoldPrice[];
};

export function PriceTrendChart({ data }: PriceTrendChartProps) {
  if (data.length === 0) {
    return null;
  }

  return (
    <div
      aria-label="金币日价格趋势图"
      className="price-trend-chart"
      role="img"
    >
      <div className="price-trend-chart__left-axis" data-testid="price-y-axis-left">
        <ResponsiveContainer height={280} width="100%">
          <LineChart data={data} margin={{ top: 12, right: 12, bottom: 0, left: 8 }}>
            <CartesianGrid stroke="#eee8de" vertical={false} />
            <XAxis
              dataKey="date"
              tickFormatter={(date: string) => date.slice(5).replace('-', '/')}
            />
            <YAxis
              dataKey="price"
              orientation="left"
              tickFormatter={(price: number) => price.toFixed(4)}
              width={54}
            />
            <Tooltip
              formatter={(price) => [`¥ ${Number(price).toFixed(4)} / 金`, '价格']}
            />
            <Line
              activeDot={{ r: 4 }}
              dataKey="price"
              dot={false}
              stroke="#b88431"
              strokeWidth={2.5}
              type="monotone"
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
