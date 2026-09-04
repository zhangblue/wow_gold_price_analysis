# 金币价格后端

后端使用 Rust、Axum 和 SeaORM 构建。它只读 PostgreSQL 中的 `gold_price_records` 表，按 `Asia/Shanghai` 自然日分组，并对同日 `ratio` 计算中位数，为前端提供价格趋势数据。

后端还负责托管发布包中的前端静态文件。

## 编译

仅编译 Rust release 二进制：

```bash
cargo build --release --manifest-path backend/Cargo.toml
```

推荐在项目根目录构建完整发布包：

```bash
python3 tools/build_release.py
```

该方式会同时构建前端，并把二进制、前端文件、配置目录和日志目录放入 `release/`。

## 配置

完整发布包生成后，编辑 `release/config/.env`：

```dotenv
DATABASE_URL=postgres://用户名:密码@主机:5432/数据库名
```

数据库必须已创建爬虫使用的表结构，并包含价格数据。可运行 [spider/sql/create_tables.sql](/Users/zhangdi/works/workspace/github/gold_price_analysis/spider/sql/create_tables.sql) 初始化表结构。

后端只从可执行文件同级的 `config/.env` 读取数据库地址。发布包运行时即为 `release/config/.env`；项目根目录 `.env` 不会被后端读取。

## 运行

先确认 `release/dist/` 和 `release/config/.env` 存在，再执行：

```bash
./release/gold-price-backend --host 127.0.0.1 --port 8080
```

服务提供：

- `GET /api/gold-prices?start_date=YYYY-MM-DD&end_date=YYYY-MM-DD`
- `dist/` 中的前端页面及单页应用回退路由

日期参数缺失、格式错误或结束日期早于开始日期时返回 `400`；数据库错误返回不含连接信息的 `500`。

日志写入 `release/logs/gold-price.log`，同时输出到终端。

## 测试

```bash
cargo fmt --manifest-path backend/Cargo.toml -- --check
cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path backend/Cargo.toml
```
