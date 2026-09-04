# Rust 金币价格后端实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 在 `backend/` 中交付一个以 Axum 和 SeaORM 构建的只读 PostgreSQL 服务，向 UI 提供按上海自然日聚合的金币价格接口，并在发布包中托管编译后的 UI。

**架构：** `main` 负责 CLI、可执行文件同级路径、配置、日志、数据库连接与监听；应用层将 HTTP、静态文件和 CORS 装配进 Axum Router；API 层校验日期并映射错误；仓储层用 SeaORM 的参数化原生查询计算每日中位数。前端以同源相对路径请求 API，发布脚本把 Rust release 二进制、UI `dist/`、配置模板和日志目录组装到 `release/`。

**技术栈：** Rust 1.98、Axum 0.8、SeaORM 1.1（PostgreSQL + Tokio + Rustls）、Tokio、Serde、Chrono、Clap、Dotenvy、Tracing、Tracing Appender、Tower HTTP、Vite、React、Vitest。

---

## 文件结构

- `backend/Cargo.toml`：Rust 包、特性和测试依赖。
- `backend/src/lib.rs`：公开模块与可测试的应用工厂。
- `backend/src/main.rs`：CLI、路径解析、配置加载、日志、数据库连接、监听。
- `backend/src/config.rs`：发布目录内 `.env` 的加载与配置错误。
- `backend/src/app.rs`：`AppState`、路由、静态 UI 回退和 CORS。
- `backend/src/api/mod.rs`：API 模块导出。
- `backend/src/api/error.rs`：统一 JSON 错误响应与状态码映射。
- `backend/src/api/gold_prices.rs`：日期查询参数、JSON DTO 和 `GET` 处理器。
- `backend/src/repository/mod.rs`：仓储模块导出。
- `backend/src/repository/gold_prices.rs`：SeaORM 查询、数据库行和领域模型。
- `backend/config/.env.example`：发布包配置模板，不含真实凭据。
- `backend/tests/api_gold_prices.rs`：HTTP 参数、成功与错误行为测试。
- `backend/tests/config.rs`：发行配置路径与缺失变量测试。
- `frontend/src/features/gold-price/goldPriceRepository.ts`：从样例数据改为调用后端 API。
- `frontend/src/features/gold-price/goldPriceRepository.test.ts`：前端 HTTP 请求与响应解析测试。
- `frontend/vite.config.ts`：开发时 `/api` 代理到 Rust 服务。
- `tools/build_release.py`：构建并组装 `release/`。
- `tests/test_build_release.py`：发布目录组装测试。
- `.gitignore`：忽略生成的 `release/` 与运行日志。

### 任务 1：建立 Rust 包、发行配置和启动参数

**文件：**

- 创建：`backend/Cargo.toml`
- 创建：`backend/src/lib.rs`
- 创建：`backend/src/config.rs`
- 创建：`backend/src/main.rs`
- 创建：`backend/config/.env.example`
- 创建：`backend/tests/config.rs`

- [ ] **步骤 1：编写配置与 CLI 的失败测试**

```rust
use gold_price_backend::config::{load_database_url, ReleasePaths};
use tempfile::tempdir;

#[test]
fn loads_database_url_from_the_release_config_directory() {
    let release = tempdir().unwrap();
    std::fs::create_dir(release.path().join("config")).unwrap();
    std::fs::write(
        release.path().join("config/.env"),
        "DATABASE_URL=postgres://example/db\n",
    ).unwrap();

    assert_eq!(
        load_database_url(&ReleasePaths::from_release_dir(release.path())).unwrap(),
        "postgres://example/db",
    );
}

#[test]
fn reports_a_missing_database_url() {
    let release = tempdir().unwrap();
    std::fs::create_dir(release.path().join("config")).unwrap();
    std::fs::write(release.path().join("config/.env"), "OTHER=value\n").unwrap();

    assert!(load_database_url(&ReleasePaths::from_release_dir(release.path()))
        .unwrap_err()
        .to_string()
        .contains("DATABASE_URL"));
}
```

- [ ] **步骤 2：运行测试并确认正确失败**

运行：`cargo test --manifest-path backend/Cargo.toml --test config`

预期：FAIL，提示 `gold_price_backend` 包或 `config` 模块尚不存在。

- [ ] **步骤 3：创建最小可测试的包、配置模块与 CLI**

在 `Cargo.toml` 定义二进制包 `gold-price-backend` 和库 `gold_price_backend`，并添加以下核心依赖：

```toml
[dependencies]
axum = "0.8"
chrono = { version = "0.4", features = ["serde"] }
clap = { version = "4", features = ["derive"] }
dotenvy = "0.15"
sea-orm = { version = "1.1", features = ["sqlx-postgres", "runtime-tokio-rustls", "macros"] }
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread", "net", "fs"] }
tracing = "0.1"
tracing-appender = "0.2"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
tower-http = { version = "0.6", features = ["cors", "fs", "trace"] }

[dev-dependencies]
tempfile = "3"
tower = { version = "0.5", features = ["util"] }
```

实现 `ReleasePaths::from_release_dir`，用显式 `PathBuf` 保存 `release_dir`、`config_file`、`dist_dir`、`logs_dir`。`load_database_url` 必须使用 `dotenvy::from_path` 加载传入的 `config/.env`，随后读取 `DATABASE_URL`；不要回退到调用进程的环境变量。`main.rs` 使用 Clap：

```rust
#[derive(clap::Parser)]
struct Cli {
    #[arg(long)]
    host: std::net::IpAddr,
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,
}
```

添加 `backend/config/.env.example`：

```dotenv
DATABASE_URL=postgres://username:password@127.0.0.1:5432/wow_gold_price
```

- [ ] **步骤 4：运行配置测试并确认通过**

运行：`cargo test --manifest-path backend/Cargo.toml --test config`

预期：PASS，两个测试通过；读取真实根目录 `.env` 不是测试前提。

- [ ] **步骤 5：提交配置和启动骨架**

```bash
git add backend/Cargo.toml backend/Cargo.lock backend/src backend/config/.env.example backend/tests/config.rs
git commit -m "feat(backend): add release configuration and cli"
```

### 任务 2：实现 SeaORM 每日中位数仓储

**文件：**

- 创建：`backend/src/repository/mod.rs`
- 创建：`backend/src/repository/gold_prices.rs`
- 创建：`backend/tests/repository_gold_prices.rs`

- [ ] **步骤 1：编写失败的仓储映射与查询参数测试**

```rust
use chrono::NaiveDate;
use gold_price_backend::repository::gold_prices::{DailyGoldPrice, GoldPriceRepository};

#[test]
fn builds_a_bounded_daily_median_query() {
    let query = GoldPriceRepository::daily_median_statement(
        NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 8, 31).unwrap(),
    );
    let debug = format!("{query:?}");

    assert!(debug.contains("percentile_cont(0.5)"));
    assert!(debug.contains("Asia/Shanghai"));
}

#[test]
fn serializable_daily_price_keeps_its_date_and_price() {
    let item = DailyGoldPrice {
        date: NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        price: 0.0121,
    };
    assert_eq!(item.date.to_string(), "2026-08-01");
    assert_eq!(item.price, 0.0121);
}
```

- [ ] **步骤 2：运行测试并确认正确失败**

运行：`cargo test --manifest-path backend/Cargo.toml --test repository_gold_prices`

预期：FAIL，提示仓储模块和领域模型尚不存在。

- [ ] **步骤 3：实现参数化 SeaORM 查询**

定义 `DailyGoldPrice { date: NaiveDate, price: f64 }` 和 `GoldPriceRepository { db: DatabaseConnection }`。`daily_median_statement` 返回 `Statement::from_sql_and_values`，不能将日期字符串插入 SQL 文本：

```sql
SELECT
  (fetched_at AT TIME ZONE 'Asia/Shanghai')::date AS date,
  percentile_cont(0.5) WITHIN GROUP (ORDER BY ratio::double precision) AS price
FROM gold_price_records
WHERE (fetched_at AT TIME ZONE 'Asia/Shanghai')::date >= $1
  AND (fetched_at AT TIME ZONE 'Asia/Shanghai')::date <= $2
GROUP BY (fetched_at AT TIME ZONE 'Asia/Shanghai')::date
ORDER BY date ASC
```

使用 SeaORM 的 `FromQueryResult` 行类型和 `find_by_statement(...).all(&self.db).await` 映射结果。仓储错误统一转换为内部 `RepositoryError`，不得记录或返回数据库 URL。SQL 只读取 `gold_price_records`，不迁移也不写入数据。

- [ ] **步骤 4：运行仓储测试并确认通过**

运行：`cargo test --manifest-path backend/Cargo.toml --test repository_gold_prices`

预期：PASS，测试确认中位数表达式、上海时区、日期边界和领域模型。

- [ ] **步骤 5：提交仓储实现**

```bash
git add backend/src/repository backend/tests/repository_gold_prices.rs
git commit -m "feat(backend): query daily median gold prices"
```

### 任务 3：实现 Axum API、静态 UI、CORS 与日志

**文件：**

- 创建：`backend/src/api/mod.rs`
- 创建：`backend/src/api/error.rs`
- 创建：`backend/src/api/gold_prices.rs`
- 创建：`backend/src/app.rs`
- 修改：`backend/src/lib.rs`
- 修改：`backend/src/main.rs`
- 创建：`backend/tests/api_gold_prices.rs`

- [ ] **步骤 1：编写失败的 HTTP 行为测试**

```rust
use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;

#[tokio::test]
async fn rejects_a_reversed_date_range() {
    let response = test_app().oneshot(
        Request::builder()
            .uri("/api/gold-prices?start_date=2026-08-31&end_date=2026-08-01")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_a_malformed_date() {
    let response = test_app().oneshot(
        Request::builder()
            .uri("/api/gold-prices?start_date=08-01-2026&end_date=2026-08-31")
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
```

`test_app()` 使用可注入的仓储 trait 或测试实现，避免 HTTP 参数测试连接真实 PostgreSQL。

- [ ] **步骤 2：运行测试并确认正确失败**

运行：`cargo test --manifest-path backend/Cargo.toml --test api_gold_prices`

预期：FAIL，提示 `test_app` 或 API 路由不存在。

- [ ] **步骤 3：实现路由、错误协议、静态文件和日志初始化**

实现严格的查询 DTO：

```rust
#[derive(serde::Deserialize)]
struct GoldPricesQuery {
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
}

#[derive(serde::Serialize)]
struct GoldPricesResponse {
    data: Vec<GoldPriceResponse>,
}
```

处理器首先比较 `start_date > end_date` 并返回 `400` 和 `{"error":"结束日期不能早于开始日期"}`；Axum 的查询提取失败同样映射到 `400`。成功时将仓储数据映射为 `{date: "YYYY-MM-DD", price: number}`；数据库错误记录 `error!`，响应 `500` 和通用 `{"error":"服务器内部错误"}`。

`app.rs` 先注册 `.route("/api/gold-prices", get(get_gold_prices))`，再用 `fallback_service(ServeDir::new(dist_dir).not_found_service(ServeFile::new(dist_dir.join("index.html"))))` 服务静态 UI。CORS 仅允许配置的开发来源和 `GET`；添加 `TraceLayer` 记录 HTTP 访问。

`main.rs` 必须在绑定端口前执行 `std::fs::create_dir_all(&paths.logs_dir)`，再创建 `logs/gold-price.log` 的 non-blocking tracing appender。保留 `WorkerGuard` 到 `axum::serve` 返回后，确保日志不会提前停止。启动日志须包含实际绑定的 `host:port`、`dist` 路径和日志文件路径，但不得记录 `DATABASE_URL`。

- [ ] **步骤 4：运行 API 测试并确认通过**

运行：`cargo test --manifest-path backend/Cargo.toml --test api_gold_prices`

预期：PASS，覆盖 `400` 日期倒置、`400` 无效格式、`200` 成功 JSON 和 `500` 仓储失败，不需要真实数据库。

- [ ] **步骤 5：提交 HTTP 服务**

```bash
git add backend/src backend/tests/api_gold_prices.rs
git commit -m "feat(backend): serve price api and ui assets"
```

### 任务 4：将前端数据边界替换为后端 API

**文件：**

- 修改：`frontend/src/features/gold-price/goldPriceRepository.ts`
- 修改：`frontend/src/features/gold-price/goldPriceRepository.test.ts`
- 修改：`frontend/vite.config.ts`

- [ ] **步骤 1：将现有样例仓储测试改为失败的 HTTP 契约测试**

```ts
import { vi } from 'vitest';
import { getDailyGoldPrices } from './goldPriceRepository';

test('requests the inclusive date range from the backend api', async () => {
  const fetchMock = vi.fn().mockResolvedValue(
    new Response(JSON.stringify({ data: [{ date: '2026-08-02', price: 0.0119 }] }), { status: 200 }),
  );
  vi.stubGlobal('fetch', fetchMock);

  await expect(getDailyGoldPrices('2026-08-02', '2026-08-02')).resolves.toEqual([
    { date: '2026-08-02', price: 0.0119 },
  ]);
  expect(fetchMock).toHaveBeenCalledWith('/api/gold-prices?start_date=2026-08-02&end_date=2026-08-02');
});

test('throws when the backend responds with an error', async () => {
  vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response('{"error":"bad request"}', { status: 400 })));
  await expect(getDailyGoldPrices('2026-08-02', '2026-08-02')).rejects.toThrow();
});
```

在每个测试结束后恢复 `fetch` mock，避免污染页面测试。

- [ ] **步骤 2：运行测试并确认正确失败**

运行：`cd frontend && npm test -- --run src/features/gold-price/goldPriceRepository.test.ts`

预期：FAIL，现有实现仍读取 `sampleDailyGoldPrices`，不会调用 `fetch`。

- [ ] **步骤 3：实现 API 仓储和开发代理**

用 `URLSearchParams` 构建查询字符串，调用相对 URL，非 `response.ok` 时抛出 `Error`，成功时仅返回 `body.data`。运行时不再导入 `sampleData.ts`。

在 Vite 配置中添加：

```ts
server: {
  proxy: {
    '/api': 'http://127.0.0.1:8080',
  },
},
```

该代理只影响开发服务器；发布时浏览器向同一 Axum 服务请求相对路径。

- [ ] **步骤 4：运行前端相关测试并确认通过**

运行：`cd frontend && npm test -- --run src/features/gold-price/goldPriceRepository.test.ts src/features/gold-price/GoldPricePage.test.tsx`

预期：PASS，HTTP 仓储契约和原有加载/错误 UI 流程都通过。

- [ ] **步骤 5：提交前端联调**

```bash
git add frontend/src/features/gold-price/goldPriceRepository.ts frontend/src/features/gold-price/goldPriceRepository.test.ts frontend/vite.config.ts
git commit -m "feat(frontend): load gold prices from backend api"
```

### 任务 5：构建并验证自包含 release 包

**文件：**

- 创建：`tools/build_release.py`
- 创建：`tests/test_build_release.py`
- 修改：`.gitignore`

- [ ] **步骤 1：编写失败的发布组装测试**

```python
from pathlib import Path
from tools.build_release import assemble_release

def test_assemble_release_copies_binary_ui_and_runtime_directories(tmp_path: Path):
    binary = tmp_path / "gold-price-backend"
    binary.write_text("binary")
    ui = tmp_path / "ui-dist"
    ui.mkdir()
    (ui / "index.html").write_text("<main>gold</main>")
    template = tmp_path / ".env.example"
    template.write_text("DATABASE_URL=postgres://example/db\n")
    release = tmp_path / "release"

    assemble_release(binary, ui, template, release)

    assert (release / "gold-price-backend").is_file()
    assert (release / "dist/index.html").is_file()
    assert (release / "config/.env").read_text() == template.read_text()
    assert (release / "logs").is_dir()
```

- [ ] **步骤 2：运行测试并确认正确失败**

运行：`PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tests/test_build_release.py -v`

预期：FAIL，提示 `tools.build_release` 尚不存在。

- [ ] **步骤 3：实现安全、可重复的发布脚本**

实现 `assemble_release`：创建目标目录、复制可执行文件至 `release/gold-price-backend`、复制 UI 目录至 `release/dist`、创建 `release/logs`。若 `release/config/.env` 已存在，保留它；若不存在，则由 `backend/config/.env.example` 初始化，以免构建覆盖操作者已经配置好的真实数据库 URL。

`main()` 依次运行：

```python
subprocess.run(["npm", "run", "build"], cwd=ROOT / "frontend", check=True)
subprocess.run(["cargo", "build", "--release", "--manifest-path", "backend/Cargo.toml"], cwd=ROOT, check=True)
```

再将 `frontend/dist`、`backend/target/release/gold-price-backend` 和配置模板交给 `assemble_release`。添加 `release/` 与 `backend/target/` 到 `.gitignore`；不要忽略受版本控制的 `backend/config/.env.example`。

- [ ] **步骤 4：运行发布测试并确认通过**

运行：`PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tests/test_build_release.py -v`

预期：PASS，二进制、`dist/index.html`、`config/.env` 和 `logs/` 都存在，并验证已有配置不被覆盖。

- [ ] **步骤 5：创建实际 release 包并做端到端烟雾验证**

运行：

```bash
python3 tools/build_release.py
DATABASE_URL='postgres://root:123456@127.0.0.1:5432/wow_gold_price' \
  ./release/gold-price-backend --host 127.0.0.1 --port 18080
```

另一个终端执行：

```bash
curl -fsS 'http://127.0.0.1:18080/api/gold-prices?start_date=2026-08-01&end_date=2026-08-31'
curl -fsS 'http://127.0.0.1:18080/'
test -f release/logs/gold-price.log
```

预期：接口返回 JSON（空数组亦可），根路径返回 UI HTML，且日志文件存在。完成后终止临时服务进程。

- [ ] **步骤 6：提交发布流程**

```bash
git add tools/build_release.py tests/test_build_release.py .gitignore backend/config/.env.example
git commit -m "build: package backend and ui release"
```

### 任务 6：执行完整验证并记录结果

**文件：**

- 修改：必要时仅修改受前述失败测试驱动的文件。

- [ ] **步骤 1：运行 Rust 格式、静态检查与完整测试**

运行：

```bash
cargo fmt --manifest-path backend/Cargo.toml -- --check
cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path backend/Cargo.toml
```

预期：所有命令退出码为 `0`。

- [ ] **步骤 2：运行完整前端测试和生产构建**

运行：

```bash
cd frontend && npm test -- --run && npm run build
```

预期：Vitest 全部通过，Vite 生成 `frontend/dist/`。

- [ ] **步骤 3：运行发布脚本测试与发布构建**

运行：

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tests/test_build_release.py -v
python3 tools/build_release.py
git diff --check
git status --short
```

预期：发布测试通过，`release/` 布局符合规格，且差异没有空白错误。

- [ ] **步骤 4：提交验证后的收尾改动**

```bash
git add -A
git commit -m "test: verify gold price backend release"
```
