# 任务 5 报告：左侧坐标趋势图与暖色视觉样式

## 交付内容

- 新增 `PriceTrendChart`，基于 Recharts 3 渲染日期 X 轴、左侧价格 Y 轴、悬浮提示和金棕色趋势线。
- 查询成功且有数据时显示图表；空结果保持现有“该时间范围暂无价格数据”状态而不渲染图表。
- 将页面复刻为无菜单的暖灰背景、暖白卡片和金棕强调色；窄屏时筛选器换行、摘要切为单列。

## TDD 记录

### 红

运行：

```bash
cd frontend && npm test -- --run src/features/gold-price/PriceTrendChart.test.tsx
```

结果：失败。`PriceTrendChart` 模块不存在，符合预期。

为页面接入添加可访问图表断言后，临时移除 `PriceTrendChart` 接入并运行：

```bash
cd frontend && npm test -- --run src/features/gold-price/GoldPricePage.test.tsx
```

结果：1 个失败，明确提示无法找到角色为 `img`、名称为“金币日价格趋势图”的元素。

### 绿

恢复图表接入后，运行：

```bash
cd frontend && npm test -- --run src/features/gold-price/PriceTrendChart.test.tsx
```

结果：1 个测试通过。

## 验证命令与结果

```bash
cd frontend && npm test -- --run
cd frontend && npm run build
git diff --check
```

- 全量 Vitest：5 个测试文件、13 个测试通过。
- 生产构建：TypeScript 编译及 Vite 构建成功。
- `git diff --check`：无空白错误。
- 窄屏浏览器验证（375 px）：`bodyScrollWidth` 为 375，与视口宽度相同；图表宽度与滚动宽度均为 295；检测到的菜单/导航元素数量为 0。

## 修改文件

- `frontend/src/features/gold-price/PriceTrendChart.tsx`
- `frontend/src/features/gold-price/PriceTrendChart.test.tsx`
- `frontend/src/features/gold-price/GoldPricePage.tsx`
- `frontend/src/features/gold-price/GoldPricePage.test.tsx`
- `frontend/src/styles.css`

## 自审

- 左侧 Y 轴设为 `orientation="left"`，保留 54px 轴宽。
- X 轴将 `YYYY-MM-DD` 格式化为 `MM/DD`；Tooltip 显示四位价格和“/ 金”。
- 图表只在成功且数据非空时接入；空、加载、失败和日期校验状态沿用已存在的条件渲染。
- 页面没有新增菜单或导航元素。

## 疑虑

- Vite 报告 Recharts 导致压缩后的 JS 入口包约 547 kB，超过默认 500 kB 的性能提示阈值；这不影响构建或本任务功能。若后续页面扩展，可考虑将图表按需加载。
