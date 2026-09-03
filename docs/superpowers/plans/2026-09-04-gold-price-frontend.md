# 金币价格趋势查询页实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 在 `frontend` 中交付一个无导航的 React 单页，按日查询并用左侧纵轴折线图展示金币价格。

**架构：** 使用 Vite 创建独立的 React + TypeScript 应用。页面组件只管理 UI 状态；`goldPriceRepository` 是唯一的数据读取边界，初期从本地样例数组筛选，未来替换为 HTTP 请求。摘要计算和日期校验为纯函数，图表只接收已排序的日价格数据。

**技术栈：** Vite、React、TypeScript、Recharts、Vitest、React Testing Library、jsdom。

---

## 文件结构

- `frontend/package.json`：前端脚本及运行、构建、测试依赖。
- `frontend/src/main.tsx`：挂载 React 根组件。
- `frontend/src/App.tsx`：导出唯一页面入口。
- `frontend/src/features/gold-price/types.ts`：`DailyGoldPrice` 和查询状态类型。
- `frontend/src/features/gold-price/sampleData.ts`：按日期升序的本地样例数据。
- `frontend/src/features/gold-price/goldPriceRepository.ts`：`getDailyGoldPrices` 数据边界。
- `frontend/src/features/gold-price/priceMetrics.ts`：日期校验、闭区间筛选、价格摘要纯函数。
- `frontend/src/features/gold-price/GoldPricePage.tsx`：日期状态、请求状态及页面组合。
- `frontend/src/features/gold-price/DateRangeFilter.tsx`：日期输入和提交事件。
- `frontend/src/features/gold-price/PriceSummary.tsx`：最新、最高、最低价格展示。
- `frontend/src/features/gold-price/PriceTrendChart.tsx`：Recharts 折线图、左侧 Y 轴与 tooltip。
- `frontend/src/styles.css`：无菜单暖色数据页和响应式样式。
- `frontend/src/test/setup.ts`：Testing Library 的 DOM 断言初始化。
- `frontend/src/features/gold-price/*.test.ts(x)`：纯函数、数据边界与页面交互测试。
- `frontend/vite.config.ts`：React 插件及 Vitest 的 jsdom 配置。

### 任务 1：建立可测试的 React 前端骨架

**文件：**

- 创建：`frontend/package.json`
- 创建：`frontend/index.html`
- 创建：`frontend/tsconfig.json`
- 创建：`frontend/vite.config.ts`
- 创建：`frontend/src/main.tsx`
- 创建：`frontend/src/App.tsx`
- 创建：`frontend/src/styles.css`
- 创建：`frontend/src/test/setup.ts`
- 创建：`frontend/src/App.test.tsx`

- [ ] **步骤 1：编写页面入口的失败测试**

```tsx
import { render, screen } from '@testing-library/react';
import App from './App';

test('renders the gold price heading', () => {
  render(<App />);
  expect(screen.getByRole('heading', { name: '金币价格走势' })).toBeInTheDocument();
});
```

- [ ] **步骤 2：运行测试并确认失败**

运行：`cd frontend && npm test -- --run src/App.test.tsx`

预期：失败，提示 `package.json` 或 `App` 尚不存在。

- [ ] **步骤 3：创建 Vite、React、Vitest 配置及最小入口**

`package.json` 至少提供如下脚本和依赖：

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc -b && vite build",
    "test": "vitest"
  },
  "dependencies": { "react": "^19", "react-dom": "^19", "recharts": "^3" },
  "devDependencies": {
    "@testing-library/jest-dom": "^6",
    "@testing-library/react": "^16",
    "@types/react": "^19",
    "@types/react-dom": "^19",
    "@vitejs/plugin-react": "^6",
    "jsdom": "^26",
    "typescript": "^5",
    "vite": "^8",
    "vitest": "^4"
  }
}
```

`App.tsx` 先提供可验证入口：

```tsx
export default function App() {
  return <main><h1>金币价格走势</h1></main>;
}
```

在 `vite.config.ts` 中使用 React 插件，并将 `test.environment` 设为 `jsdom`、`test.setupFiles` 设为 `./src/test/setup.ts`；在 setup 文件中导入 `@testing-library/jest-dom/vitest`。

- [ ] **步骤 4：安装依赖并确认入口测试通过**

运行：`cd frontend && npm install && npm test -- --run src/App.test.tsx`

预期：PASS，显示 1 个通过测试。

- [ ] **步骤 5：提交骨架**

```bash
git add frontend
git commit -m "feat(frontend): scaffold gold price application"
```

### 任务 2：实现日价格模型、样例数据和数据访问边界

**文件：**

- 创建：`frontend/src/features/gold-price/types.ts`
- 创建：`frontend/src/features/gold-price/sampleData.ts`
- 创建：`frontend/src/features/gold-price/goldPriceRepository.ts`
- 创建：`frontend/src/features/gold-price/goldPriceRepository.test.ts`

- [ ] **步骤 1：编写数据边界的失败测试**

```ts
import { getDailyGoldPrices } from './goldPriceRepository';

test('returns ascending records inside the inclusive date range', async () => {
  await expect(getDailyGoldPrices('2026-08-02', '2026-08-03')).resolves.toEqual([
    { date: '2026-08-02', price: 0.0119 },
    { date: '2026-08-03', price: 0.0117 },
  ]);
});
```

- [ ] **步骤 2：运行测试并确认失败**

运行：`cd frontend && npm test -- --run src/features/gold-price/goldPriceRepository.test.ts`

预期：失败，提示模块不存在。

- [ ] **步骤 3：定义模型与样例实现**

```ts
export type DailyGoldPrice = { date: string; price: number };

export async function getDailyGoldPrices(startDate: string, endDate: string) {
  return sampleDailyGoldPrices.filter(
    (item) => item.date >= startDate && item.date <= endDate,
  );
}
```

在 `sampleData.ts` 提供至少 31 条连续的 `2026-08-01` 至 `2026-08-31` 样例记录，并包含测试中指定的 2 条数据。数组按 `date` 升序保存。

- [ ] **步骤 4：运行数据边界测试并确认通过**

运行：`cd frontend && npm test -- --run src/features/gold-price/goldPriceRepository.test.ts`

预期：PASS，返回 2 条闭区间记录。

- [ ] **步骤 5：提交数据访问边界**

```bash
git add frontend/src/features/gold-price/types.ts frontend/src/features/gold-price/sampleData.ts frontend/src/features/gold-price/goldPriceRepository.ts frontend/src/features/gold-price/goldPriceRepository.test.ts
git commit -m "feat(frontend): add daily gold price repository"
```

### 任务 3：实现校验和价格摘要纯函数

**文件：**

- 创建：`frontend/src/features/gold-price/priceMetrics.ts`
- 创建：`frontend/src/features/gold-price/priceMetrics.test.ts`

- [ ] **步骤 1：编写失败测试**

```ts
import { getPriceSummary, validateDateRange } from './priceMetrics';

test('rejects a reversed date range', () => {
  expect(validateDateRange('2026-08-31', '2026-08-01')).toBe('结束日期不能早于开始日期');
});

test('calculates latest, high, low, and day-over-day change', () => {
  expect(getPriceSummary([
    { date: '2026-08-01', price: 0.0120 },
    { date: '2026-08-02', price: 0.0117 },
    { date: '2026-08-03', price: 0.0124 },
  ])).toEqual({ latest: 0.0124, high: 0.0124, low: 0.0117, change: 0.0007, changePercent: 5.98 });
});
```

- [ ] **步骤 2：运行测试并确认失败**

运行：`cd frontend && npm test -- --run src/features/gold-price/priceMetrics.test.ts`

预期：失败，提示函数未导出。

- [ ] **步骤 3：实现无副作用的计算函数**

```ts
export function validateDateRange(startDate: string, endDate: string): string | null {
  if (!startDate || !endDate) return '请选择开始日期和结束日期';
  return startDate > endDate ? '结束日期不能早于开始日期' : null;
}
```

`getPriceSummary` 对已按日期升序且非空的数组计算最后一项、`Math.max`、`Math.min` 和相邻两项的变化率；使用 `Number(value.toFixed(2))` 生成百分比。空数组不调用该函数。

- [ ] **步骤 4：运行纯函数测试并确认通过**

运行：`cd frontend && npm test -- --run src/features/gold-price/priceMetrics.test.ts`

预期：PASS，日期倒置和四项摘要均匹配预期。

- [ ] **步骤 5：提交计算逻辑**

```bash
git add frontend/src/features/gold-price/priceMetrics.ts frontend/src/features/gold-price/priceMetrics.test.ts
git commit -m "feat(frontend): calculate gold price metrics"
```

### 任务 4：实现查询、状态反馈和摘要组件

**文件：**

- 创建：`frontend/src/features/gold-price/DateRangeFilter.tsx`
- 创建：`frontend/src/features/gold-price/PriceSummary.tsx`
- 创建：`frontend/src/features/gold-price/GoldPricePage.tsx`
- 创建：`frontend/src/features/gold-price/GoldPricePage.test.tsx`
- 修改：`frontend/src/App.tsx`

- [ ] **步骤 1：编写失败的页面交互测试**

```tsx
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import App from '../../App';

test('shows a validation message instead of loading reversed dates', async () => {
  const user = userEvent.setup();
  render(<App />);
  await user.type(screen.getByLabelText('开始日期'), '2026-08-31');
  await user.type(screen.getByLabelText('结束日期'), '2026-08-01');
  await user.click(screen.getByRole('button', { name: '查询价格' }));
  expect(await screen.findByText('结束日期不能早于开始日期')).toBeInTheDocument();
});
```

- [ ] **步骤 2：运行测试并确认失败**

运行：`cd frontend && npm test -- --run src/features/gold-price/GoldPricePage.test.tsx`

预期：失败，日期输入或按钮尚未渲染。

- [ ] **步骤 3：实现受控日期筛选和查询状态机**

`GoldPricePage` 使用如下状态和值：

```ts
type LoadState = 'idle' | 'loading' | 'success' | 'empty' | 'error';
const [startDate, setStartDate] = useState('2026-08-01');
const [endDate, setEndDate] = useState('2026-08-31');
```

提交时先调用 `validateDateRange`；合法时设置 `loading`，`await getDailyGoldPrices(startDate, endDate)`，再根据数组长度设为 `success` 或 `empty`。异常时设为 `error`。`DateRangeFilter` 的两个 `input type="date"` 分别提供「开始日期」和「结束日期」可访问标签，加载期间禁用按钮；`PriceSummary` 仅在成功且有数据时显示。

- [ ] **步骤 4：补充并运行状态覆盖测试**

新增断言：初始页面显示「最新价格」；查询无数据时显示「该时间范围暂无价格数据」；repository 拒绝时显示可重试文本。运行：`cd frontend && npm test -- --run src/features/gold-price/GoldPricePage.test.tsx`

预期：PASS，验证、成功、空数据和错误状态全部通过。

- [ ] **步骤 5：提交页面交互**

```bash
git add frontend/src/App.tsx frontend/src/features/gold-price/DateRangeFilter.tsx frontend/src/features/gold-price/PriceSummary.tsx frontend/src/features/gold-price/GoldPricePage.tsx frontend/src/features/gold-price/GoldPricePage.test.tsx
git commit -m "feat(frontend): add gold price date query page"
```

### 任务 5：实现左侧坐标的趋势图和视觉样式

**文件：**

- 创建：`frontend/src/features/gold-price/PriceTrendChart.tsx`
- 创建：`frontend/src/features/gold-price/PriceTrendChart.test.tsx`
- 修改：`frontend/src/features/gold-price/GoldPricePage.tsx`
- 修改：`frontend/src/styles.css`

- [ ] **步骤 1：编写图表和视觉结构的失败测试**

```tsx
import { render, screen } from '@testing-library/react';
import { PriceTrendChart } from './PriceTrendChart';

test('labels the chart and renders a left price axis', () => {
  render(<PriceTrendChart data={[{ date: '2026-08-01', price: 0.0124 }]} />);
  expect(screen.getByRole('img', { name: '金币日价格趋势图' })).toBeInTheDocument();
  expect(screen.getByTestId('price-y-axis-left')).toBeInTheDocument();
});
```

- [ ] **步骤 2：运行测试并确认失败**

运行：`cd frontend && npm test -- --run src/features/gold-price/PriceTrendChart.test.tsx`

预期：失败，提示 `PriceTrendChart` 模块不存在。

- [ ] **步骤 3：实现 Recharts 图表组件并接入页面**

```tsx
<ResponsiveContainer width="100%" height={280}>
  <LineChart data={data} margin={{ top: 12, right: 12, bottom: 0, left: 8 }}>
    <CartesianGrid vertical={false} stroke="#eee8de" />
    <XAxis dataKey="date" tickFormatter={(date) => date.slice(5).replace('-', '/')} />
    <YAxis dataKey="price" orientation="left" width={54} tickFormatter={(price) => price.toFixed(4)} />
    <Tooltip formatter={(price) => [`¥ ${Number(price).toFixed(4)} / 金`, '价格']} />
    <Line type="monotone" dataKey="price" stroke="#b88431" strokeWidth={2.5} dot={false} activeDot={{ r: 4 }} />
  </LineChart>
</ResponsiveContainer>
```

为可测试性，在左侧 `YAxis` 外包一个带 `data-testid="price-y-axis-left"` 的元素，图表容器使用 `role="img"` 和 `aria-label="金币日价格趋势图"`。在 CSS 中实现确认稿的浅暖灰背景、暖白卡片、金棕色线、左轴留白和最大宽度；使用媒体查询将摘要变为单列、筛选按钮换行，且不引入菜单。

- [ ] **步骤 4：运行图表与完整验证**

运行：`cd frontend && npm test -- --run && npm run build`

预期：全部测试 PASS，TypeScript 编译及 Vite 生产构建成功。

- [ ] **步骤 5：进行窄屏视觉验证并提交**

运行：`cd frontend && npm run dev -- --host 127.0.0.1`，在约 375 px 宽度确认日期条件、摘要和图表不横向溢出；再运行 `git diff --check`。

```bash
git add frontend/src/features/gold-price/PriceTrendChart.tsx frontend/src/features/gold-price/PriceTrendChart.test.tsx frontend/src/features/gold-price/GoldPricePage.tsx frontend/src/styles.css
git commit -m "feat(frontend): render left-axis gold price trend"
```

## 计划自检

- 规格的无菜单、日期精确到日、左侧纵轴、摘要、空／加载／失败状态、窄屏和 API 替换边界，分别由任务 1 至 5 覆盖。
- 每个新增函数、组件和测试文件均在文件结构及相应任务中定义；`DailyGoldPrice`、`getDailyGoldPrices`、`validateDateRange` 和 `getPriceSummary` 的名称全篇一致。
- 计划不涉及后端、数据库、爬虫、路由或未授权功能。
