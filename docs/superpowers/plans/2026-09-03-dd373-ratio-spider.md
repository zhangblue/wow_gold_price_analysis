# DD373 比例爬虫实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 在 `spider/` 创建一个命令行爬虫，按列表页顺序保存前 10 条商品的金币比例数值、原始比例文本和统一采集时间。

**架构：** 标准库 `urllib.request` 下载公开页面；`html.parser` 在每个商品标题 (`h2`) 的范围内收集段落，并从符合 `1元=<数值>金` 的文本解析比例。抓取层在解析成功后生成一份 Asia/Shanghai 的 ISO 8601 时间戳，为每条记录写入同一个 JSON 文件。

**技术栈：** Python 3 标准库、unittest。

---

## 文件结构

- `spider/__init__.py`：包标记。
- `spider/fetcher.py`：带 User-Agent、超时和明确异常的只读 HTTP 获取函数。
- `spider/parser.py`：商品卡片范围识别与比例文本解析。
- `spider/main.py`：命令行入口、时间戳和 JSON 输出。
- `spider/tests/test_parser.py`：解析前 10 条及格式异常测试。
- `spider/tests/test_main.py`：抓取结果的时间戳和输出模型测试。
- `spider/README.md`：安装前提、运行方式、输出格式和约束说明。

### 任务 1：定义可测试的解析边界

**文件：**
- 创建：`spider/tests/test_parser.py`
- 创建：`spider/parser.py`

- [ ] **步骤 1：编写失败的测试**

```python
assert parse_ratio_text("1元=80.3859金") == 80.3859
assert parse_result_html(page_with_eleven_product_cards) == [80.1, ..., 80.10]
```

- [ ] **步骤 2：运行测试验证失败**

运行：`python3 -m unittest spider.tests.test_parser -v`

预期：FAIL，因 `spider.parser` 尚未定义。

- [ ] **步骤 3：编写最少实现代码**

```python
def parse_ratio_text(text: str) -> float | None:
    match = RATIO_PATTERN.fullmatch(" ".join(text.split()))
    return float(match.group("ratio")) if match else None
```

- [ ] **步骤 4：运行测试验证通过**

运行：`python3 -m unittest spider.tests.test_parser -v`

预期：PASS。

### 任务 2：封装网络获取和输出模型

**文件：**
- 创建：`spider/tests/test_main.py`
- 创建：`spider/fetcher.py`
- 创建：`spider/main.py`

- [ ] **步骤 1：编写失败的测试**

```python
records = crawl("https://example.test/list", fetch_html=lambda _: FIXTURE, captured_at=fixed_time)
assert records[0]["fetched_at"] == "2026-09-03T14:30:15+08:00"
```

- [ ] **步骤 2：运行测试验证失败**

运行：`python3 -m unittest spider.tests.test_main -v`

预期：FAIL，因 `crawl` 尚未定义。

- [ ] **步骤 3：编写最少实现代码**

```python
def crawl(url, fetch_html=fetch_url, captured_at=None):
    timestamp = (captured_at or now_in_shanghai()).isoformat()
    return [{"rank": i, "ratio": ratio, "raw_text": raw, "fetched_at": timestamp} for i, (ratio, raw) in enumerate(..., 1)]
```

- [ ] **步骤 4：运行测试验证通过**

运行：`python3 -m unittest spider.tests.test_main -v`

预期：PASS。

### 任务 3：交付命令行工程并进行端到端验证

**文件：**
- 创建：`spider/README.md`
- 修改：`spider/main.py`

- [ ] **步骤 1：实现命令行参数和原子 JSON 写入**

```python
parser.add_argument("--url", default=DEFAULT_URL)
parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
```

- [ ] **步骤 2：运行完整自动化测试**

运行：`python3 -m unittest discover -s spider/tests -v`

预期：PASS。

- [ ] **步骤 3：运行真实页面抓取**

运行：`python3 -m spider.main --output /tmp/dd373-ratios.json`

预期：输出 10 条记录，每条具备数值比例、原始文本和同一带 `+08:00` 的时间戳。
