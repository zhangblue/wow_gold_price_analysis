# 项目约定

## 金币价格数据

- `gold_price_records` 是爬虫写入的原始数据表，后端不得修改其中的数据。
- `daily_gold_price_summaries` 是由页面“汇总数据”按钮刷新的一日一条汇总表；操作者必须在运行服务前手动执行 `backend/sql/create_daily_gold_price_summaries.sql`。
- 汇总按 `Asia/Shanghai` 自然日分组，对同日 `ratio` 取中位数；刷新必须在单个 PostgreSQL 事务中执行清空和插入。
- 日价格查询必须从 `daily_gold_price_summaries` 读取，不得在查询请求中重新聚合原始表。

## 发布与配置

- 生产数据库地址仅从发布包同级的 `config/.env` 读取。
- 使用 `python3 tools/build_release.py` 构建发布包；该脚本会保留已有 `release/config/.env`，但替换 `release/dist/`。
