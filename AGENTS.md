# 项目约定

## 金币价格数据

- `gold_price_records` 是爬虫写入的原始数据表，后端不得修改其中的数据。
- `daily_gold_price_summaries` 是由页面“汇总数据”按钮刷新的一日一条汇总表；操作者必须在运行服务前手动执行 `backend/sql/create_daily_gold_price_summaries.sql`。
- 汇总按 `Asia/Shanghai` 自然日分组，对同日 `ratio` 取中位数；刷新必须在单个 PostgreSQL 事务中执行清空和插入。
- 日价格查询必须从 `daily_gold_price_summaries` 读取，不得在查询请求中重新聚合原始表。
- 前端的「汇总数据」操作调用 `POST /api/gold-prices/summary`；它成功后才重新查询当前日期区间。
- 前端日期条件初始值必须由浏览器当前月份计算：开始日为本月第 1 天，结束日为本月最后 1 天；不得写死年月。
- 爬虫写入的 `started_at`、`finished_at` 和 `fetched_at` 必须是 `Asia/Shanghai` 时区、精确到秒的时间戳。

## 测试与文档

- 修改前端日期或查询行为时，更新 `frontend/src/features/gold-price/GoldPricePage.test.tsx` 的回归覆盖，并运行 `cd frontend && npm test -- --run` 与 `npm run build`。
- 修改爬虫持久化逻辑时，运行 `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s spider/tests -v`。
- 根目录 `readme.md` 应与 API、数据库初始化和发布行为保持同步。

## 发布与配置

- 生产数据库地址仅从发布包同级的 `config/.env` 读取。
- 使用 `python3 tools/build_release.py` 构建发布包；该脚本会保留已有 `release/config/.env`，但替换 `release/dist/`。
