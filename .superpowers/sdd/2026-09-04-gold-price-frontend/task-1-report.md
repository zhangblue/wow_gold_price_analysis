# 任务 1 报告：建立可测试的 React 前端骨架

## 实现内容

- 新建 `frontend` Vite + React + TypeScript 应用骨架。
- 配置 `react`、`react-dom`、`recharts` 运行时依赖，以及 Testing Library、Vitest、Vite 和 TypeScript 开发依赖。
- 配置 Vitest 使用 `jsdom`、`./src/test/setup.ts` 和全局测试 API，并在 setup 中加载 `@testing-library/jest-dom/vitest`。
- 提供 `main.tsx`、最小 `App.tsx` 页面入口和基础样式。
- 页面入口渲染精确标题“金币价格走势”。

## TDD 红绿证据

### 红灯

先创建 `frontend/src/App.test.tsx`，再运行：

```text
$ npm test -- --run src/App.test.tsx
npm error code ENOENT
npm error path .../frontend/package.json
npm error enoent Could not read package.json
```

失败原因是待实现的前端骨架尚不存在，符合预期。

### 绿灯

创建最小实现和配置后，运行：

```text
$ npm test -- --run src/App.test.tsx
Test Files  1 passed (1)
Tests  1 passed (1)
```

## 测试与构建

- `npm install`：成功，安装 152 个包；审计发现 0 个漏洞。
- `npm test -- --run src/App.test.tsx`：通过，1 个测试文件、1 个测试。
- `npm run build`：通过，`tsc -b` 和 `vite build` 均成功。
- `git diff --check`：通过，无空白错误。

## 修改文件

- `frontend/package.json`
- `frontend/package-lock.json`
- `frontend/index.html`
- `frontend/tsconfig.json`
- `frontend/vite.config.ts`
- `frontend/src/main.tsx`
- `frontend/src/App.tsx`
- `frontend/src/styles.css`
- `frontend/src/test/setup.ts`
- `frontend/src/App.test.tsx`

## 自审

- 入口测试先于生产实现编写，并观察到预期红灯。
- 生产实现保持为简报要求的最小入口，没有加入任务 2+ 的业务功能。
- `globals: true` 是为兼容简报提供的无导入 `test(...)` 写法；`types: ["vitest/globals"]` 使 TypeScript 构建能识别该全局 API。
- 依赖安装生成的 `node_modules`、构建产物和 TypeScript 增量文件已从提交索引移除，但当前仓库未配置前端专用忽略规则，因此它们会作为本地未跟踪文件保留。

## 疑虑

无。
