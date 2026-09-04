# 金币价格日汇总实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 增加手动汇总原始价格的按钮与事务化后端接口，让价格趋势查询从每日汇总表读取。

**架构：** 操作者手工创建汇总表；SeaORM 在一个事务内清空并重建它；Axum 暴露汇总 POST 接口，GET 改读汇总表；React 点击汇总后重新查询当前日期范围。

**技术栈：** PostgreSQL、SeaORM、Axum、Tokio、React、TypeScript、Vitest。

---

## 文件结构

- `backend/sql/create_daily_gold_price_summaries.sql`：手动执行的汇总表 SQL。
- `backend/src/repository/gold_prices.rs`：汇总表查询与事务刷新。
- `backend/src/api/gold_prices.rs`：汇总 HTTP 处理器和响应类型。
- `backend/src/app.rs`：汇总路由和 POST CORS。
- `backend/tests/*.rs`：SQL 与 HTTP 行为测试。
- `frontend/src/features/gold-price/*.ts(x)`：汇总 API 客户端、按钮和交互测试。

### 任务 1：建立汇总表并切换价格查询

**文件：**

- 创建：`backend/sql/create_daily_gold_price_summaries.sql`
- 修改：`backend/src/repository/gold_prices.rs`
- 修改：`backend/tests/repository_gold_prices.rs`

- [ ] **步骤 1：编写失败的汇总表查询测试**

```rust
#[test]
fn reads_date_range_from_daily_summary_table() {
    let start = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 8, 31).unwrap();
    let debug = format!("{:?}", GoldPriceRepository::daily_median_statement(start, end));

    assert!(debug.contains("daily_gold_price_summaries"));
    assert!(debug.contains("summary_date >= $1"));
    assert!(debug.contains("summary_date <= $2"));
    assert!(debug.contains("median_ratio::double precision AS price"));
}
```

- [ ] **步骤 2：运行测试并确认失败**

运行：`cargo test --manifest-path backend/Cargo.toml --test repository_gold_prices reads_date_range_from_daily_summary_table`

预期：FAIL，当前 SQL 仍聚合 `gold_price_records`。

- [ ] **步骤 3：添加建表 SQL 并改为查询汇总表**

创建 `backend/sql/create_daily_gold_price_summaries.sql`：

```sql
CREATE TABLE IF NOT EXISTS daily_gold_price_summaries (
    summary_date date PRIMARY KEY,
    median_ratio numeric(20, 10) NOT NULL,
    source_record_count integer NOT NULL CHECK (source_record_count > 0),
    aggregated_at timestamptz NOT NULL DEFAULT now()
);
```

将 `DAILY_MEDIAN_SQL` 替换为：

```sql
SELECT summary_date AS date, median_ratio::double precision AS price
FROM daily_gold_price_summaries
WHERE summary_date >= $1 AND summary_date <= $2
ORDER BY summary_date ASC
```

保留 `Statement::from_sql_and_values(..., [start.into(), end.into()])`，日期不得插入 SQL 字符串。

- [ ] **步骤 4：运行仓储测试并确认通过**

运行：`cargo test --manifest-path backend/Cargo.toml --test repository_gold_prices`

预期：PASS，验证汇总表、闭区间和绑定值占位符。

- [ ] **步骤 5：提交查询切换**

```bash
git add backend/sql/create_daily_gold_price_summaries.sql backend/src/repository/gold_prices.rs backend/tests/repository_gold_prices.rs
git commit -m "feat(backend): read daily price summaries"
```

### 任务 2：实现事务化汇总刷新接口

**文件：**

- 修改：`backend/src/repository/gold_prices.rs`
- 修改：`backend/src/api/gold_prices.rs`
- 修改：`backend/src/app.rs`
- 修改：`backend/tests/api_gold_prices.rs`

- [ ] **步骤 1：编写失败的刷新接口测试**

```rust
#[tokio::test]
async fn refreshes_daily_summaries() {
    let response = test_app_with_summary(Ok(SummaryRefresh {
        summary_count: 31,
        aggregated_at: "2026-09-04T10:30:00+08:00".to_owned(),
    }))
    .oneshot(Request::post("/api/gold-prices/summary").body(Body::empty()).unwrap())
    .await
    .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response_body(response).await, r#"{"summary_count":31,"aggregated_at":"2026-09-04T10:30:00+08:00"}"#);
}
```

- [ ] **步骤 2：运行测试并确认失败**

运行：`cargo test --manifest-path backend/Cargo.toml --test api_gold_prices refreshes_daily_summaries`

预期：FAIL，POST 路由和刷新方法尚不存在。

- [ ] **步骤 3：实现仓储事务与 POST 路由**

扩展 `GoldPriceReader`：

```rust
fn refresh_daily_summaries(
    &self,
) -> impl Future<Output = Result<SummaryRefresh, RepositoryError>> + Send;
```

`GoldPriceRepository` 使用 `self.db.begin()`，在同一事务中依次执行：

```sql
TRUNCATE TABLE daily_gold_price_summaries;
```

```sql
INSERT INTO daily_gold_price_summaries (summary_date, median_ratio, source_record_count, aggregated_at)
SELECT (fetched_at AT TIME ZONE 'Asia/Shanghai')::date,
       percentile_cont(0.5) WITHIN GROUP (ORDER BY ratio::double precision)::numeric(20, 10),
       COUNT(*)::integer, now()
FROM gold_price_records
GROUP BY (fetched_at AT TIME ZONE 'Asia/Shanghai')::date
ORDER BY summary_date
RETURNING aggregated_at;
```

仅在插入和 `commit` 成功后返回 `SummaryRefresh`；错误让事务回滚。API 成功 JSON 为 `{"summary_count":31,"aggregated_at":"RFC3339 时间"}`。失败记录服务端日志，返回 `500 {"error":"汇总数据失败，请重试"}`。在 `app.rs` 注册 `post(refresh_gold_price_summaries::<R>)`，并将 CORS 改为允许 `GET` 和 `POST`。

- [ ] **步骤 4：运行 API 测试并确认通过**

运行：`cargo test --manifest-path backend/Cargo.toml --test api_gold_prices`

预期：PASS，POST 成功、POST 失败与既有 GET 测试都通过，且不连接真实数据库。

- [ ] **步骤 5：提交汇总接口**

```bash
git add backend/src/repository/gold_prices.rs backend/src/api/gold_prices.rs backend/src/app.rs backend/tests/api_gold_prices.rs
git commit -m "feat(backend): refresh daily price summaries"
```

### 任务 3：接入前端汇总按钮和自动刷新

**文件：**

- 修改：`frontend/src/features/gold-price/goldPriceRepository.ts`
- 修改：`frontend/src/features/gold-price/goldPriceRepository.test.ts`
- 修改：`frontend/src/features/gold-price/DateRangeFilter.tsx`
- 修改：`frontend/src/features/gold-price/GoldPricePage.tsx`
- 修改：`frontend/src/features/gold-price/GoldPricePage.test.tsx`

- [ ] **步骤 1：编写失败的前端汇总测试**

```ts
test('posts to daily summary endpoint', async () => {
  const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({
    summary_count: 31, aggregated_at: '2026-09-04T10:30:00+08:00',
  })));
  vi.stubGlobal('fetch', fetchMock);

  await expect(refreshDailyGoldPrices()).resolves.toEqual({
    summaryCount: 31, aggregatedAt: '2026-09-04T10:30:00+08:00',
  });
  expect(fetchMock).toHaveBeenCalledWith('/api/gold-prices/summary', { method: 'POST' });
});
```

```tsx
test('reloads current dates after successful summary', async () => {
  const user = userEvent.setup();
  vi.stubGlobal('fetch', vi.fn()
    .mockResolvedValueOnce(new Response(JSON.stringify({ summary_count: 2, aggregated_at: '2026-09-04T10:30:00+08:00' })))
    .mockResolvedValueOnce(new Response(JSON.stringify({ data: [{ date: '2026-08-01', price: 0.0121 }] }))));
  render(<App />);
  await user.click(screen.getByRole('button', { name: '汇总数据' }));
  expect(await screen.findByText('汇总完成，共处理 2 天数据')).toBeInTheDocument();
});
```

- [ ] **步骤 2：运行测试并确认失败**

运行：`cd frontend && npm test -- --run src/features/gold-price/goldPriceRepository.test.ts src/features/gold-price/GoldPricePage.test.tsx`

预期：FAIL，因为汇总客户端和按钮尚不存在。

- [ ] **步骤 3：实现客户端、按钮和页面状态**

增加：

```ts
export async function refreshDailyGoldPrices(): Promise<{ summaryCount: number; aggregatedAt: string }> {
  const response = await fetch('/api/gold-prices/summary', { method: 'POST' });
  if (!response.ok) throw new Error(`Failed to refresh gold prices: ${response.status}`);
  const body: { summary_count: number; aggregated_at: string } = await response.json();
  return { summaryCount: body.summary_count, aggregatedAt: body.aggregated_at };
}
```

`DateRangeFilter` 接收 `isRefreshing` 和 `onRefresh`，新增：

```tsx
<button type="button" disabled={isLoading || isRefreshing} onClick={onRefresh}>
  {isRefreshing ? '汇总中…' : '汇总数据'}
</button>
```

`GoldPricePage` 成功时显示“汇总完成，共处理 N 天数据”，再调用现有查询；失败时保留 `prices` 与图表，仅显示“汇总数据失败，请重试”。汇总期间两个操作按钮禁用。

- [ ] **步骤 4：运行前端测试并确认通过**

运行：`cd frontend && npm test -- --run src/features/gold-price/goldPriceRepository.test.ts src/features/gold-price/GoldPricePage.test.tsx`

预期：PASS，覆盖 POST、禁用、成功重查、失败保留已有图表。

- [ ] **步骤 5：提交前端交互**

```bash
git add frontend/src/features/gold-price/goldPriceRepository.ts frontend/src/features/gold-price/goldPriceRepository.test.ts frontend/src/features/gold-price/DateRangeFilter.tsx frontend/src/features/gold-price/GoldPricePage.tsx frontend/src/features/gold-price/GoldPricePage.test.tsx
git commit -m "feat(frontend): refresh daily price summaries"
```

### 任务 4：完整验证

**文件：**

- 修改：仅修复前述测试明确暴露的问题。

- [ ] **步骤 1：运行 Rust 验证**

运行：`cargo fmt --manifest-path backend/Cargo.toml -- --check && cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings && cargo test --manifest-path backend/Cargo.toml`

预期：全部通过。

- [ ] **步骤 2：运行前端与发布验证**

运行：`cd frontend && npm test -- --run && npm run build`

预期：全部前端测试和生产构建通过。

- [ ] **步骤 3：构建发布包并检查差异**

运行：`PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tests/test_build_release.py -v && python3 tools/build_release.py && git diff --check`

预期：发布测试通过，发布包构建成功，差异无空白错误。

- [ ] **步骤 4：提交验证后的收尾改动**

```bash
git add -A
git commit -m "test: verify daily price summary refresh"
```
