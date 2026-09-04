# Rust 金币价格后端设计

## 目标

在 `backend/` 下创建一个使用 Rust 编写的独立服务。服务只读 PostgreSQL 中由现有爬虫写入的数据，向 UI 提供按日期范围查询的金币价格数据，并同时托管生产构建后的 UI。

## 范围

- 使用 Axum 提供 HTTP 路由、静态文件和 CORS。
- 使用 SeaORM 管理 PostgreSQL 连接池和参数化查询。
- 使用 `Asia/Shanghai` 作为自然日边界；同一天内的 `ratio` 聚合为中位数。
- 从发行包中的 `config/.env` 加载 `DATABASE_URL`。
- 启动参数接收监听地址和端口。
- 将 UI 构建产物和 Rust 可执行文件一起发布到 `release/`。
- 记录启动、HTTP 访问和运行错误到 `release/logs/`。

不在本次范围内：触发或管理 Python 爬虫、写入数据库、用户鉴权、缓存和新的数据库迁移。

## 架构与数据流

`gold-price-backend` 在启动时解析 `--host` 和 `--port`，从可执行文件同级的 `config/.env` 读取 `DATABASE_URL`，建立 SeaORM PostgreSQL 连接池并初始化日志。

浏览器访问页面路径时，Axum 从可执行文件同级的 `dist/` 返回 Vite 的构建产物。浏览器请求 `GET /api/gold-prices` 时，处理器校验日期参数，调用仓储层，再将结果序列化为 JSON。

仓储层通过 SeaORM 执行带绑定参数的 PostgreSQL 查询：先将 `gold_price_records.fetched_at` 转为 `Asia/Shanghai` 日期，按日期分组，再使用 `percentile_cont(0.5) WITHIN GROUP (ORDER BY ratio)` 计算当日中位数。结果按日期升序返回。

## API

## 查询金币日价格 / Get Daily Gold Prices

### 基本信息

- **请求方式（Method）：** `GET`
- **请求路径（Path）：** `/api/gold-prices`
- **请求参数（Query）：** `start_date`、`end_date`
- **鉴权：** 无

### 请求参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `start_date` | string | 是 | 起始日期，格式为 `YYYY-MM-DD`，包含当天。 |
| `end_date` | string | 是 | 结束日期，格式为 `YYYY-MM-DD`，包含当天，且不得早于 `start_date`。 |

### 成功响应

状态码为 `200`。无匹配记录同样返回 `200` 和空数组。

```json
{
  "data": [
    { "date": "2026-08-01", "price": 0.0121 },
    { "date": "2026-08-02", "price": 0.0119 }
  ]
}
```

`price` 为 JSON 数字，代表当天 `ratio` 的中位数；`data` 始终按日期升序排列。

### 错误响应

```json
{
  "error": "结束日期不能早于开始日期"
}
```

| 状态码 | 情况 |
| --- | --- |
| `400` | 参数缺失、日期格式不是 `YYYY-MM-DD`，或日期范围倒置。 |
| `500` | 数据库查询失败或服务内部异常；响应不包含连接字符串或数据库细节。 |

## 运行与发布

发布脚本依次构建 UI、编译 Rust 的 release 二进制、创建以下目录结构，并复制构建产物：

```text
release/
├── gold-price-backend
├── config/
│   └── .env
├── dist/
│   └── … Vite 构建产物 …
└── logs/
```

`config/.env` 的必需内容如下：

```dotenv
DATABASE_URL=postgres://user:password@host:5432/database
```

服务示例：

```bash
./release/gold-price-backend --host 127.0.0.1 --port 8080
```

二进制以自身所在目录为根目录查找 `config/.env`、`dist/` 和 `logs/`，所以可从任意工作目录执行。日志写入 `logs/gold-price.log`，同时输出到标准输出；日志包含 UTC 时间、级别、模块和消息。日志目录不存在时由服务创建。

## UI 集成

前端的 `getDailyGoldPrices` 改为请求相对路径 `/api/gold-prices`，解析响应中的 `data` 数组。开发模式下，Vite 配置代理该路径到本地 Rust 服务；生产模式下页面和 API 来自同一来源，因此不依赖浏览器跨域访问。后端保留仅允许开发 UI 源的 `GET` CORS 设置，便于单独启动前端进行开发。

## 文件边界

- `backend/src/main.rs`：进程启动、命令行参数、路径解析、日志和监听。
- `backend/src/app.rs`：Axum 路由、共享状态、静态文件与 CORS。
- `backend/src/api/gold_prices.rs`：请求校验、HTTP 响应和错误映射。
- `backend/src/repository/gold_prices.rs`：SeaORM 查询与数据库行映射。
- `backend/src/config.rs`：发行包 `.env` 加载和配置校验。
- `backend/tests/`：API 行为、日期校验和 SQL 查询映射测试。
- `tools/build_release.py`：构建并组装 `release/`。

## 测试策略

- API 单元测试覆盖合法范围、空数据、缺参、无效日期和倒置日期。
- 仓储测试验证查询结果按日期排序并保留中位数价格映射。
- 运行 PostgreSQL 集成测试时使用显式测试数据库 URL，不读取发行配置。
- 发布脚本测试验证 `release/gold-price-backend`、`release/dist/`、`release/config/.env` 与 `release/logs/` 均被正确创建。
