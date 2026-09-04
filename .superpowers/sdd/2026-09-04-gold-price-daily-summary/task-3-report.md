# 任务 3：前端汇总按钮和自动刷新

## 已实现

- 增加 `refreshDailyGoldPrices`，向 `/api/gold-prices/summary` 发送 POST，并将后端响应映射为前端命名。
- 日期筛选区增加“汇总数据”按钮；汇总或查询进行时，两个操作均禁用。
- 汇总成功后展示处理天数并按当前日期范围重新查询价格。
- 汇总失败时展示重试提示，已有价格摘要和趋势图保持不变。

## TDD 证据

先添加仓储 POST 契约、成功重查、进行中禁用、失败保留图表的测试。初次可执行测试运行因缺少客户端实现及按钮而失败；实现后通过以下验证：

```text
npm test -- --run src/features/gold-price/goldPriceRepository.test.ts src/features/gold-price/GoldPricePage.test.tsx
Test Files  2 passed (2)
Tests  13 passed (13)

npm run build
✓ built in 328ms
```
