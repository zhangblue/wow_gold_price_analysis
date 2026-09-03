# DD373 PostgreSQL 持久化与可配置调度实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 将每轮 DD373 前 10 条 XPath 比例原子写入 PostgreSQL，并以默认单次或可配置分钟间隔执行。

**架构：** `main.py` 负责编排和调度；`database.py` 用 Psycopg 3 参数化 SQL 写入部署者预先创建的批次/明细表。爬虫不包含迁移或任何 DDL。

**技术栈：** Python 3.9+、Psycopg 3、PostgreSQL、unittest。

---

## 文件结构

- `requirements.txt`：Psycopg 3 二进制依赖。
- `spider/database.py`：成功/失败批次写入。
- `spider/main.py`：`--interval-minutes`、单轮和循环调度。
- `spider/tests/test_database.py`：迁移和持久化行为测试。
- `spider/tests/test_main.py`：调度与命令行参数测试。
- `spider/README.md`：依赖、环境变量和运行方式。

### 任务 1：数据库写入边界

**文件：** 创建 `spider/database.py`、`spider/tests/test_database.py`、`requirements.txt`。

- [ ] **步骤 1：编写失败的成功批次测试**

```python
repository.save_success(url, started_at, records)
assert len(fake_cursor.executemany_calls[0][1]) == 10
assert "INSERT INTO crawl_runs" in fake_cursor.executed[0][0]
```

- [ ] **步骤 2：运行并确认失败**

运行：`python3 -m unittest spider.tests.test_database.DatabaseWriteTests -v`

预期：FAIL，`spider.database` 尚不存在。

- [ ] **步骤 3：实现最小写入接口**

```python
with psycopg.connect(database_url) as conn:
    with conn.transaction():
        with conn.cursor() as cur:
            cur.execute(INSERT_RUN_SQL, (url, started_at, finished_at, "success", 10))
            crawl_run_id = cur.fetchone()[0]
            cur.executemany(INSERT_RECORD_SQL, rows)
```

所有 SQL 使用 `%s` 占位符，且仅包含 `INSERT` 操作。`DATABASE_URL` 只从环境变量读取，缺失时抛出不含连接信息的配置错误；不存在的表或列错误原样作为安全的数据库异常向上交付，不执行补救 DDL。

- [ ] **步骤 4：运行写库测试并确认通过**

运行：`python3 -m unittest spider.tests.test_database.DatabaseWriteTests -v`

预期：PASS。

- [ ] **步骤 5：Commit**

运行：`git add requirements.txt spider/database.py spider/tests/test_database.py && git commit -m "feat: persist crawler runs to postgres"`

### 任务 2：单轮编排与调度参数

**文件：** 修改 `spider/main.py`、`spider/tests/test_main.py`。

- [ ] **步骤 1：编写失败的调度测试**

```python
assert interval_minutes("0") == 0
assert interval_minutes("2.5") == 2.5
with self.assertRaises(argparse.ArgumentTypeError):
    interval_minutes("-1")
run_scheduled(0, run_once_fn=record_call, sleep_fn=fail_if_called)
assert calls == [None]
```

- [ ] **步骤 2：运行并确认失败**

运行：`python3 -m unittest spider.tests.test_main.SchedulerTests -v`

预期：FAIL，调度函数尚未定义。

- [ ] **步骤 3：实现单轮和循环**

```python
def run_scheduled(interval_minutes, run_once_fn, sleep_fn=time.sleep):
    while True:
        run_once_fn()
        if interval_minutes == 0:
            return
        sleep_fn(interval_minutes * 60)
```

`run_once()` 抓取 10 条后调用成功批次写入；异常时调用失败批次写入并返回非零。`--interval-minutes` 默认 `0`，正数循环，负数在 argparse 阶段拒绝。

- [ ] **步骤 4：运行测试并确认通过**

运行：`python3 -m unittest spider.tests.test_main -v`

预期：PASS。

- [ ] **步骤 5：Commit**

运行：`git add spider/main.py spider/tests/test_main.py && git commit -m "feat: add configurable crawl scheduling"`

### 任务 3：文档与真实集成验证

**文件：** 修改 `spider/README.md`。

- [ ] **步骤 1：记录运行要求**

README 明确 `DATABASE_URL` 必填、两张表必须预先存在，并给出默认单次和 `--interval-minutes 10` 的运行示例；不得写入真实连接字符串。

- [ ] **步骤 2：安装依赖并执行完整测试**

运行：`python3 -m pip install -r requirements.txt && python3 -m unittest discover -s spider/tests -v`

预期：Psycopg 可导入，所有测试 PASS。

- [ ] **步骤 3：运行一次真实集成验证**

运行：通过已设置的 `DATABASE_URL` 执行 `python3 -m spider.main --interval-minutes 0`，随后使用参数化查询校验最新成功批次关联 10 条 `rank` 为 1–10 的明细。

预期：退出码 0，批次 `record_count=10`，10 条记录共享抓取时间戳。

- [ ] **步骤 4：Commit**

运行：`git add spider/README.md docs/superpowers/plans/2026-09-03-dd373-postgres-scheduler.md && git commit -m "docs: document postgres crawler operation"`
