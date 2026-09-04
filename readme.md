# 金币价格分析

本项目采集并保存金币价格数据，提供按日期查询的价格趋势页面。系统由爬虫、React 前端和 Rust 后端组成：爬虫将原始数据写入 PostgreSQL；后端按 `Asia/Shanghai` 自然日对同日 `ratio` 取中位数；前端展示价格摘要和趋势图。

## 目录说明

```text
spider/      Python 爬虫与数据库建表脚本
frontend/    React + Vite 前端
backend/     Rust + Axum + SeaORM 后端
tools/       发布构建脚本
release/     生成的可运行发布包（不纳入版本控制）
```

## 前置条件

- Python 3
- Node.js 与 npm
- Rust 工具链（Cargo）
- 可访问的 PostgreSQL 数据库

首次部署需依次执行下列脚本：

```bash
psql "$DATABASE_URL" -f spider/sql/create_tables.sql
psql "$DATABASE_URL" -f backend/sql/create_daily_gold_price_summaries.sql
```

爬虫只写入原始表 `gold_price_records`；后端页面的「汇总数据」按钮会刷新 `daily_gold_price_summaries`，按 `Asia/Shanghai` 自然日计算同日 `ratio` 的中位数。

## 编译发布包

在项目根目录运行：

```bash
python3 tools/build_release.py
```

该命令会构建前端和 Rust release 二进制，并生成：

```text
release/
├── gold-price-backend
├── config/
│   └── .env
├── dist/
└── logs/
```

重复构建会替换 `release/dist/` 中的前端文件，但会保留已有的 `release/config/.env`，避免覆盖真实数据库配置。

## 配置

首次构建会根据 `backend/config/.env.example` 创建 `release/config/.env`。编辑该文件，填入实际数据库地址：

```dotenv
DATABASE_URL=postgres://用户名:密码@主机:5432/数据库名
```

后端只读取 `release/config/.env`，不会使用项目根目录 `.env` 或启动命令中的 `DATABASE_URL` 环境变量。

## 运行

```bash
./release/gold-price-backend --host 127.0.0.1 --port 8080
```

启动后在浏览器打开 <http://127.0.0.1:8080/>。前端页面和 API 由同一个服务提供：

- 页面：`/`
- 价格接口：`GET /api/gold-prices?start_date=2026-08-01&end_date=2026-08-31`
- 汇总接口：`POST /api/gold-prices/summary`

接口按日期升序返回：

```json
{
  "data": [
    { "date": "2026-08-01", "price": 0.0121 }
  ]
}
```

运行日志同时输出到终端，并写入 `release/logs/gold-price.log`。

前端首次打开时，日期条件默认是浏览器当前月份的第一天至最后一天；可在页面中修改后查询。请先点击「汇总数据」刷新日汇总，再查询最新的价格趋势。

## 验证

```bash
cargo test --manifest-path backend/Cargo.toml
cd frontend && npm test -- --run
cd .. && PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s spider/tests -v
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tests/test_build_release.py -v
```
