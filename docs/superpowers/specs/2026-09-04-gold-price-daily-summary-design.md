# 金币价格日汇总设计

## 目标

将 `gold_price_records` 中的原始价格按 `Asia/Shanghai` 自然日汇总为每日中位数。用户通过页面“汇总数据”按钮手动刷新汇总表；趋势图和日期查询随后只读取汇总表，不再在查询请求中聚合原始记录。

## 范围

- 新增由操作者手动创建的 `daily_gold_price_summaries` 表。
- 新增触发汇总刷新的 Rust API。
- 将现有日价格查询切换到汇总表。
- 在前端增加汇总按钮、进行状态、成功后重查与失败提示。

不在本次范围内：自动定时汇总、爬虫改动、历史汇总版本、用户鉴权和数据库迁移自动执行。

## 手动建表 SQL

操作者必须在程序运行前手动执行以下 SQL：

```sql
CREATE TABLE IF NOT EXISTS daily_gold_price_summaries (
    summary_date date PRIMARY KEY,
    median_ratio numeric(20, 10) NOT NULL,
    source_record_count integer NOT NULL CHECK (source_record_count > 0),
    aggregated_at timestamptz NOT NULL DEFAULT now()
);
```

`summary_date` 作为主键，保证一个上海自然日只有一条记录。`median_ratio` 使用 `numeric(20, 10)`，保留偶数条 `ratio` 数据取中位数时可能出现的额外小数位。`source_record_count` 用于确认当天参与汇总的原始记录数量；`aggregated_at` 记录最后一次刷新时间。

## 汇总刷新

新增 `POST /api/gold-prices/summary`。处理器调用仓储层，在单个 PostgreSQL 事务中执行清空和插入；任何一步失败都回滚，不会使查询接口读到空表或半成品。

```sql
TRUNCATE TABLE daily_gold_price_summaries;

INSERT INTO daily_gold_price_summaries (
    summary_date,
    median_ratio,
    source_record_count,
    aggregated_at
)
SELECT
    (fetched_at AT TIME ZONE 'Asia/Shanghai')::date,
    percentile_cont(0.5) WITHIN GROUP (ORDER BY ratio::double precision)::numeric(20, 10),
    COUNT(*)::integer,
    now()
FROM gold_price_records
GROUP BY (fetched_at AT TIME ZONE 'Asia/Shanghai')::date
ORDER BY summary_date;
```

成功时返回 `200`：

```json
{
  "summary_count": 31,
  "aggregated_at": "2026-09-04T10:30:00+08:00"
}
```

数据库错误时记录不含连接信息的服务端日志，返回 `500`：

```json
{
  "error": "汇总数据失败，请重试"
}
```

## 日价格查询

现有 `GET /api/gold-prices?start_date=YYYY-MM-DD&end_date=YYYY-MM-DD` 保留 API 契约，改为从 `daily_gold_price_summaries` 读取：

```sql
SELECT
    summary_date AS date,
    median_ratio::double precision AS price
FROM daily_gold_price_summaries
WHERE summary_date >= $1 AND summary_date <= $2
ORDER BY summary_date ASC;
```

日期范围是闭区间。汇总表尚无数据或范围内没有数据时返回 `200` 与 `{"data":[]}`。

## 页面交互

日期筛选表单中新增“汇总数据”按钮。用户点击后调用 `POST /api/gold-prices/summary`：

- 汇总进行中，查询与汇总按钮都禁用，汇总按钮文案为“汇总中…”。
- 汇总成功后，调用现有日期查询逻辑刷新摘要和折线图。
- 汇总失败时，保留当前摘要和图表，显示“汇总数据失败，请重试”。
- 日期校验只适用于查询操作；汇总全量原始数据，不依赖当前筛选日期。

## 文件边界

- `backend/src/repository/gold_prices.rs`：区分汇总刷新和汇总表查询的 SQL、结果模型与事务。
- `backend/src/api/gold_prices.rs`：保留日价格查询处理器，新增汇总处理器与响应 DTO。
- `backend/src/app.rs`：注册 `POST /api/gold-prices/summary`。
- `backend/tests/`：覆盖查询来源、汇总 SQL、事务失败和 API 响应。
- `frontend/src/features/gold-price/DateRangeFilter.tsx`：增加汇总操作入口与禁用状态。
- `frontend/src/features/gold-price/GoldPricePage.tsx`：管理汇总状态、错误和刷新。
- `frontend/src/features/gold-price/goldPriceRepository.ts`：封装汇总 API 调用。
- `frontend/src/features/gold-price/*.test.tsx`：覆盖按钮和刷新交互。

## 测试策略

- 仓储测试验证查询只使用汇总表，并验证刷新 SQL 使用事务、清空、上海时区中位数和记录数。
- API 测试覆盖汇总成功、仓储失败与既有日期校验。
- 前端测试覆盖汇总中禁用按钮、汇总成功后按当前范围重新加载，以及失败后保留现有结果。
