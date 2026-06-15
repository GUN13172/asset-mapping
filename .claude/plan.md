# 前端重构 + 后端修复计划

## 一、前端 UI 重构 — 暖色专业风格 + 新布局

### 配色方案：暗金/琥珀色调

去掉科技蓝/蓝紫渐变，改为：
- **主色**：`#f59e0b`（琥珀/暖橙）作为强调色
- **背景**：深灰暖色系 `#1a1714` → `#231f1b` → `#2d2824`（带微暖棕底色，不是冷蓝灰）
- **文字**：`#faf5ef`（暖白）/ `#a89a8c`（暖灰）
- **边框**：`rgba(255, 200, 120, 0.08)` 极淡暖色
- **渐变**：`linear-gradient(135deg, #f59e0b, #d97706)` 单一暖色渐变
- **浅色模式**：奶白底 `#fdfcfa` + 暗棕文字 + 同样的琥珀强调色

### 布局重做

去掉当前 Header + Sider 双层结构，改为：
- **只保留左侧导航栏**（无顶部栏）
- 左侧导航：顶部放品牌 logo + 名称，中间放导航项（图标+文字），底部放设置和主题切换
- 导航栏宽度 220px，折叠时 64px
- 右侧内容区占满剩余空间，去掉 1600px 最大宽度限制
- 更紧凑的内间距（20px 而非 24px）

### 需要修改的文件

1. **`src/styles/theme.css`** — 完全重写深色主题变量
2. **`src/styles/theme-light.css`** — 重写浅色主题变量
3. **`src/styles/app.css`** — 重写布局（去掉 header，纯侧边栏布局）
4. **`src/styles/components.css`** — 更新组件样式适配新配色
5. **`src/styles/animations.css`** — 精简，去掉过多花哨动画，保留实用的
6. **`src/App.tsx`** — 重写布局组件（去掉 Header，侧边栏内置品牌+设置+主题切换）

---

## 二、后端修复（按优先级）

### HIGH — 必须修复

1. **key_manager.rs:47** — `expect()` 改为返回 `Result`，避免 panic
2. **fofa.rs:290** — `Path::parent().unwrap()` 改为 `ok_or_else()`

### MEDIUM — 应该修复

3. **daydaymap.rs + hunter.rs** — 将 `eprintln!` debug 日志用 `#[cfg(debug_assertions)]` 包裹，或改用 `log` crate
4. **main.rs:947,951** — `stdout/stderr take().unwrap()` 改为 `ok_or_else()`
5. **converter/*.rs** — Regex 编译从每次调用改为 `Lazy<Regex>` 静态缓存
6. **api/mod.rs** — `export_all_platforms` 失败时返回哪些平台失败了，而不是静默丢弃
7. **main.rs `export_results_with_progress`** — 实现时间范围过滤（当前参数被忽略）
8. **history.rs** — `vul_count` 正确传入实际漏洞数量

### LOW — 优化项

9. **converter/operators.rs** — 字符串替换需要感知引号内容，避免误改引号内的 `=`/`:`
10. **config/mod.rs** — 标注 XOR 是 obfuscation 而非 encryption（加注释即可）
11. **api/mod.rs:151** — CSV 列使用 `BTreeSet` 或 `IndexSet` 保证确定性顺序

---

## 执行顺序

1. 先做前端 UI 重构（配色 → 布局 → 组件适配）
2. 再做后端 HIGH/MEDIUM 修复
3. 最后验证 `cargo check` 和 `npm run build` 通过
