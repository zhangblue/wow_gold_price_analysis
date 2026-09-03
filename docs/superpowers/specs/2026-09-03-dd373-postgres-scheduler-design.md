# DD373 爬虫 PostgreSQL 持久化与可配置调度设计

## 目标

将每次成功爬取的 10 条 DD373 比例记录持久化到已存在的 PostgreSQL 数据库，并支持一次性运行或按可配置的分钟间隔循环运行。

## 范围

- 保持现有公开页面读取与 XPath 对应的 `p[2]` 解析规则。
- 每轮抓取成功时写入一个抓取批次及其前 10 条记录。
- 用命令行参数控制调度；不引入操作系统级 cron 配置。
- 表结构由部署者预先创建；爬虫不执行 DDL 或迁移。
- 不记录数据库连接字符串、密码或其他凭据到代码、日志或文档。

## 数据模型

### `crawl_runs`

一行表示一次抓取尝试。

| 列 | 类型 | 约束与含义 |
| --- | --- | --- |
| `id` | `bigint generated always as identity` | 主键 |
| `source_url` | `text` | 非空，实际抓取的列表页 URL |
| `started_at` | `timestamptz` | 非空，抓取开始时间 |
| `finished_at` | `timestamptz` | 成功或失败时写入 |
| `status` | `text` | 非空，仅 `success` 或 `failed` |
| `record_count` | `smallint` | 非空，默认 0；成功必须为 10，失败为 0 |
| `error_message` | `text` | 失败原因；成功为 `NULL` |

约束：`status = 'success'` 时 `record_count = 10` 且 `error_message IS NULL`；`status = 'failed'` 时 `record_count = 0` 且 `error_message IS NOT NULL`。

### `gold_price_records`

一行表示一次成功抓取中一个排名位置的比例记录。

| 列 | 类型 | 约束与含义 |
| --- | --- | --- |
| `id` | `bigint generated always as identity` | 主键 |
| `crawl_run_id` | `bigint` | 非空，外键引用 `crawl_runs(id)`，删除批次时级联删除 |
| `rank` | `smallint` | 非空，范围 1–10，页面显示顺序 |
| `ratio` | `numeric(12,8)` | 非空，来自 `1金=<数值>元` 等号后的数值 |
| `raw_text` | `text` | 非空，原始节点文本，例如 `1金=0.0124元` |
| `fetched_at` | `timestamptz` | 非空，本轮 10 条记录共用的采集时间 |

约束：`unique (crawl_run_id, rank)`，并创建 `fetched_at` 索引以支持按时间查询历史价格。

## 运行与调度

新增 `--interval-minutes` 参数，类型为非负数字，默认 `0`。

- `0`：执行一轮抓取与入库后退出。
- 正数：启动后立即执行一轮；随后等待指定分钟数并再次执行，直至进程被停止。
- 负数：参数校验失败，打印错误并以非零状态退出。

每轮独立：一次抓取失败会记录到已存在的 `crawl_runs`，并在循环模式下继续等待下一轮；不会让常驻进程退出。成功轮在同一个数据库事务中写入批次状态和 10 条明细。数据库连接、表缺失或结构不匹配时，程序明确失败；由于无可用表，这类错误无法额外记录失败批次。

## 配置与安全

数据库连接仅从环境变量 `DATABASE_URL` 读取。缺失、无法连接、表缺失或结构不匹配时，程序明确报错，不尝试使用默认数据库、执行 DDL 或输出凭据。

数据库驱动将作为工程依赖显式声明。代码使用参数化 SQL，不拼接比例文本或 URL 到 SQL 字符串。

## 验收与测试

- 部署前已按本文档预先创建两张表和索引；爬虫运行不会创建或修改数据库对象。
- 成功一轮产生 1 条 `crawl_runs(status='success')` 与 10 条关联明细。
- 解析少于 10 条、网络错误或写库失败产生失败批次且没有部分明细。
- 参数 `0` 只调用一轮；正数进入循环；负数被拒绝。
- 数据库存储的 `ratio`、`raw_text`、排名与现有解析输出一致。
