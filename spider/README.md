# DD373 金币比例爬虫

## 简介

本程序抓取指定 DD373 商品列表页中前 10 条商品的金币比例，并将结果写入 PostgreSQL 数据库和本地 JSON 文件。

每条商品读取其第二条比例文本（对应页面 XPath 中的 `p[2]`）：

```text
/html/body/div[2]/div[3]/div/div[1]/div[2]/div[1]/div/div[3]/div/p[2]
```

例如，页面文本为 `1金=0.0124元` 时，程序保存等号后的数值 `0.0124`，同时记录原始文本、排名及抓取时间戳。

程序仅访问公开商品列表页；不会登录、提交表单、购买商品或绕过访问限制。

## 运行环境

- Python 3.9 或更高版本（需要标准库 `zoneinfo`）
- 可访问的 PostgreSQL 数据库
- 已安装 `psql` 命令行工具（仅首次建表时需要）

## 安装依赖

建议在仓库根目录创建并启用虚拟环境：

```bash
python3 -m venv .venv
source .venv/bin/activate
python3 -m pip install --upgrade pip
python3 -m pip install -r requirements.txt
```

`requirements.txt` 中包含 PostgreSQL 驱动 `psycopg`。如果不使用虚拟环境，也可以直接执行最后一条安装命令。

## 配置数据库

程序从环境变量 `DATABASE_URL` 读取 PostgreSQL 连接地址。请在当前终端中设置该变量；不要把包含密码的连接地址提交到 Git 仓库。

```bash
export DATABASE_URL='postgresql://<用户名>:<密码>@<主机>:<端口>/<数据库名>'
```

首次运行前，需由部署者手动创建数据表：

```bash
psql "$DATABASE_URL" -f spider/sql/create_tables.sql
```

建表脚本会创建以下表：

| 表名 | 用途 |
| --- | --- |
| `crawl_runs` | 保存每次抓取任务的开始/结束时间、状态、数量及错误信息。 |
| `gold_price_records` | 保存每次成功抓取的 10 条金币比例记录。 |

爬虫只会插入数据，不会执行建表、迁移或其他 DDL 操作。若数据表未创建、结构不匹配或数据库连接失败，程序会报错。

## 运行程序

所有命令均在仓库根目录执行。

### 只运行一次

默认参数为 `--interval-minutes 0`：立即抓取、写入数据库和 JSON 文件后退出。

```bash
python3 -m spider.main
```

也可以显式指定：

```bash
python3 -m spider.main --interval-minutes 0
```

### 定时运行

传入大于 0 的分钟数后，程序启动时会立即执行一次，随后按该间隔持续执行。以下示例为每 10 分钟运行一次：

```bash
python3 -m spider.main --interval-minutes 10
```

使用 `Ctrl+C` 可停止定时任务。

### 可用参数

| 参数 | 默认值 | 说明 |
| --- | --- | --- |
| `--url` | 内置 DD373 商品列表页地址 | 指定要抓取的商品列表页 URL。 |
| `--output` | `spider/output/results.json` | 指定本地 JSON 输出文件。 |
| `--interval-minutes` | `0` | `0` 表示只运行一次；正数表示循环间隔（单位：分钟）；负数会被拒绝。 |

示例：指定页面、输出文件并每 10 分钟运行一次：

```bash
python3 -m spider.main \
  --url 'https://www.dd373.com/...' \
  --output /tmp/dd373-ratios.json \
  --interval-minutes 10
```

## 数据输出

每次成功抓取时，程序会在一个数据库事务中写入 1 条 `crawl_runs` 记录和 10 条 `gold_price_records` 记录，然后更新本地 JSON 文件。JSON 示例：

```json
[
  {
    "rank": 1,
    "ratio": 0.0124,
    "raw_text": "1金=0.0124元",
    "fetched_at": "2026-09-03T14:30:15+08:00"
  }
]
```

字段说明：

| 字段 | 说明 |
| --- | --- |
| `rank` | 商品在本次抓取结果中的顺序，范围为 1 到 10。 |
| `ratio` | 原始比例文本中等号后的数值。 |
| `raw_text` | 页面中的完整比例文本，例如 `1金=0.0124元`。 |
| `fetched_at` | 本次抓取的统一时间戳，时区为 `Asia/Shanghai`。 |

## 打包部署

在已安装依赖、完成建表验证后，可在仓库根目录打包运行所需文件：

```bash
tar -czf dd373-gold-price-spider.tar.gz \
  requirements.txt \
  spider/__init__.py \
  spider/main.py \
  spider/database.py \
  spider/fetcher.py \
  spider/parser.py \
  spider/sql/create_tables.sql
```

将压缩包复制到目标机器并解压后，依次执行以下操作：

```bash
tar -xzf dd373-gold-price-spider.tar.gz
python3 -m venv .venv
source .venv/bin/activate
python3 -m pip install -r requirements.txt
export DATABASE_URL='postgresql://<用户名>:<密码>@<主机>:<端口>/<数据库名>'
psql "$DATABASE_URL" -f spider/sql/create_tables.sql
python3 -m spider.main --interval-minutes 10
```

若目标数据库已按同一脚本建表，可跳过 `psql` 命令。

## 故障排查

- `DATABASE_URL is required`：尚未在当前终端设置数据库连接地址。
- 数据库连接或建表错误：检查数据库服务、连接地址、账号权限及数据表是否按 `create_tables.sql` 创建。
- 有效记录不足 10 条：页面结构或内容可能变化，程序不会把不完整结果作为成功批次写入。
- 请求失败：检查网络连接、目标页面是否可访问，以及是否出现临时访问限制。
