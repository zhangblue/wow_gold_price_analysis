# DD373 金币比例爬虫

抓取指定 DD373 商品列表页中按页面顺序排列的前 10 条商品的第二条比例文本。定位规则对应用户指定 XPath 中的 `p[2]`：`/html/body/div[2]/div[3]/div/div[1]/div[2]/div[1]/div/div[3]/div/p[2]`。当前页面该节点的格式为 `1金=0.0124元`，程序提取等号后的 `0.0124`。

运行数据库持久化版本前安装依赖：

```bash
python3 -m pip install -r requirements.txt
```

并设置 `DATABASE_URL` 环境变量。该值不会写入代码或日志。

## 首次建表

在首次运行数据库持久化版本前，由部署者手动执行 [create_tables.sql](sql/create_tables.sql) 创建 `crawl_runs` 和 `gold_price_records`。爬虫自身不会执行建表、迁移或其他 DDL。

## 运行

在仓库根目录执行：

```bash
python3 -m spider.main
```

默认 `--interval-minutes 0`，只执行一次；使用 `--interval-minutes 10` 每十分钟执行一次。

默认输出到 `spider/output/results.json`。可指定不同的页面或输出路径：

```bash
python3 -m spider.main --url 'https://www.dd373.com/...' --output /tmp/dd373-ratios.json
```

## 输出

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

`fetched_at` 为单次抓取的统一 Asia/Shanghai 时间戳。页面结构变化、请求失败或有效记录少于 10 条时，程序会以非零状态退出，且不会写入结果文件。

本工具只读取公开商品列表页；不会登录、提交表单、购买商品或尝试绕过访问限制。
