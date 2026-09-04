# 金币价格前端

这是一个使用 React、TypeScript、Vite 和 Recharts 构建的单页应用。用户可选择日期范围，查看金币日价格、最高/最低价格、涨跌和趋势图。

前端通过同源接口 `GET /api/gold-prices` 获取数据；服务端返回的数据已按上海自然日聚合，并对同日 `ratio` 取中位数。

## 编译

首次使用时安装依赖：

```bash
npm install
```

构建生产产物：

```bash
npm run build
```

构建结果输出到 `frontend/dist/`。项目级发布脚本会将它复制到 `release/dist/`：

```bash
cd ..
python3 tools/build_release.py
```

## 配置

前端不需要单独的环境变量配置。开发服务器已将 `/api` 代理到 `http://127.0.0.1:8080`，因此需要先以该地址启动后端。

生产环境不使用开发代理；前端由 Rust 服务托管，API 与页面处于同一来源。

## 运行

开发模式：

```bash
npm run dev
```

按终端显示的地址打开页面，通常为 <http://127.0.0.1:5173/>。

生产模式请按项目根目录的 `readme.md` 构建并启动 `release/gold-price-backend`，不要单独直接打开 `dist/index.html`。

## 测试

```bash
npm test -- --run
```
