# Finance Brief — OctoSense Market MVP TODO

> OctoSense Market (OSM) — Phase 1 MVP 实现计划，落地在 `finance-brief/`。
> 需求来源：`OctoSense Market Agent - MVP产品需求文档.md`（10 章 PRD）。
> 工作流：调研 → 实现 → 截图 → commit（全部本地，不推远端）。

## 0. 决策纪要（Q1–Q6，与用户对齐）

| # | 问题 | 决策 |
|---|---|---|
| Q1 | 在哪扩展？ | `finance-brief/` 直接演进，**不开新目录** |
| Q2 | Agent 面板（右侧栏）| **后置**，Phase 2 再说 |
| Q3 | 数据源 | 保留 4 个免费源；金十/Wind/Binance WS/Glassnode/Polygon **不接** |
| Q4 | K 线实现 | 先调研 Makepad Shader + Octoscript 60FPS 可行性，再定方案 |
| Q5 | 范围 | 只做 **Phase 1** |
| Q6 | 提交策略 | 独立仓库 octoOs/finance-brief（main），commit 后推送 origin/main |
| Q7 | 谁写代码 | sub-agent 实现；主对话负责 TODO / review / 汇报 |
| Q8 | 改哪个仓库 | octoOs/finance-brief；同步到 OctoSense/apps/finance-brief/bundle/ 以便在 shell 里运行 |

git 作者：仓库发布者（用 `GIT_AUTHOR_*` / `GIT_COMMITTER_*` 环境变量设置）。

## 1. 数据源现状（v0.1.0 已工作）

| 来源 | URL | 内容 | 状态 |
|---|---|---|---|
| Sina 要闻 | `feed.mix.sina.com.cn` | 中文财经要闻 RSS | OK（GBK，需 ASCII 显示名兜底） |
| Tencent 行情 | `qt.gtimg.cn` | A股 / **HK 美股上一交易日收盘** | OK；美股不是实时，需换源 |
| Hyperliquid | `api.hyperliquid.xyz` | 加密货币实时（BTC/ETH/SOL 等） | OK（POST JSON） |
| Frankfurter | `api.frankfurter.dev` | ECB 外汇汇率 | OK |

**美股实时源待调研**，候选：stooq / query1.finance.yahoo.com / finnhub.io / alphavantage.co / twelvedata / tiingo / marketdata.app / nasdaq.com / cnbc.com / marketstack。

## 2. 调研任务（先派 2 个并行 agent）

### R-1：美股实时源
- 测：可达 / 是否 key / 延迟字段（秒级实时 vs 收盘价）/ 编码 / JSON 格式
- 输出：可用清单 + 推荐首选 + manifest hosts 片段
- 验收：返回 ≥3 个可达候选 + 1 个首选（含为什么）

### R-2：Makepad Shader + Octoscript 可行性（K 线 60FPS ≥100 根）
- 读 `OctoScript-App-Design-Flow/docs/SCRIPT-API.md`（Widget / Stdlib / Draw / Shader / CustomShader 关键字）
- 读 `.sources/makepad/{widgets,draw}/src/` 找最少原型代码量
- 读 `.sources/makepad/examples/` 找自定义绘图（柱图/折线）例子
- 读 `OctoScript-App-Design-Flow/examples/` 6 样例的 main.splash 有无 canvas 类用法
- 读参考实现的 K 线思路（**仅思路**，本项目 makepad 不是 iced）
- 输出：能做 / 不能做 / 有条件做 + 性能预期 + 实现路径 + 备选（纯 DrawColor 矩形拼接）
- 验收：明确结论 + 推荐路径或 TODO 占位

## 3. 界面拆分（1–12）

| # | 界面 | 来源 | 任务 | 依赖 | 验收 | 状态 |
|---|---|---|---|---|---|---|
| 1 | Launcher（App Card 网格）| 6.1 | I-1 | — | 6 个 tile 网格 + 点击进入对应界面 | done (v0.2.0) |
| 2 | 新闻简报列表 | 3.1 | I-2 | — | 已有要闻 tab 升级，主题筛选 UI | done (D-1) |
| 3 | 新闻详情 | 当前 detail | I-3 | I-2 | 加来源链接 UI（一键跳转原始 URL） | done (D-1) |
| 4 | 研究卡列表 | 3.2 | I-4 | — | 模板：公告 / 财报 / 异动 / 宏观 / 链上 5 类 | done (D-2) |
| 5 | 研究卡详情（三段式）| 3.2 | I-5 | I-4 | MVP **只做事实层**（蓝色实线）；分析/建议留 TODO 显式占位 | done (D-2)；分析层 / 建议层为 Phase 2 占位，确认/驳回按钮禁用 |
| 6 | K线看盘（OHLC + 周期）| 3.3 | I-6 | R-2 | OHLC 5 柱 + 时间周期（1m/5m/1d）+ 缩放；实现方式由 R-2 定 | TBD |
| 7 | 行情列表合并（A股/美股/加密/外汇合一）| 当前 5 tab | I-7 | R-1 | 单一列表 + 4 个主题筛选 chip | TBD |
| 8 | 收藏 / 关注 | 当前收藏 tab | I-8 | — | 可按类型分组；点击进入对应详情 | done (v0.2.0) |
| 9 | 设置（数据源/主题/推送）| 6.3 + 7 | I-9 | — | 主题切换 + 数据源开关 + 推送时段 | done (v0.2.0) |
| 10 | 免责声明 | 7.3 | I-10 | — | 首次启动显示 + 设置入口 | done (v0.2.0) |
| 11 | 事件流 stub（mock 快讯 + 订单簿）| 3.1 + 3.3 | I-11 | I-2 / I-6 | **条件做**：能跑就实现；不可行写 TODO 占位 | TBD |
| 12 | 备用槽 | — | — | — | 视余量 | TBD |

> Phase 1 不做：Agent 右侧栏（PRD §6.1）、Anti-Spoofing（§3.4）、研究卡"分析/建议"层（§3.2）、Footprint/Heatmap（§3.3）、TTS（§3.1）、TimescaleDB/Kafka/Milvus（§5）。

## 4. 实现任务（agent 拆分）

| Agent | 范围 | 输入 | 输出 |
|---|---|---|---|
| **A-调研** | R-1 美股实时源 | 当前 4 个 host 已知 | 报告 + 1 首选 + manifest hosts 片段 |
| **B-调研** | R-2 Makepad Shader | splash 自由读 | 报告 + 实现路径或 TODO 占位 |
| **C-简单界面** | I-1 + I-8 + I-9 + I-10 | splash + PRD §6/§7 | splash 增量代码 + listing/manifest 更新 + 4 张截图 |
| **D-内容界面** | I-2 + I-3 + I-4 + I-5 | splash + PRD §3.1/§3.2 | splash 增量代码 + 4 张截图 |

**Agent 拆分原则**：写权限完全独立（不重叠文件）。所有 agent 完成后，由主对话收集 + 派 review agent。

## 5. Code-review 检查点（每个 agent 后强制）

按 skill 规则 4，每次 commit 后派独立 review agent，检查：

1. **Splash 硬约束**：
   - 读不存在键用 `get(obj, key, fallback)` 不用裸 `o[k]`
   - 不写 `sqrt/pow/round(v*10^dec)`，只写 `floor(a)` + 小数 carry
   - 动态键写入对象改用 plain `let` + 显式分发函数
   - `on_render` 中 `if empty { } for ... { }` 永远不要 `else for ... { }`
   - 所有 hex 字面量带 `#x` 前缀
   - 不用保留字 `ok / tick / me / self / is / do / try / use` 作标识符
   - `net.http_request` 字节处理：`to_string()` 仅用于 ASCII，不依赖中文解码
2. **manifest**：
   - hosts 完整（含新增调研源）
   - capabilities 含 `storage / net`
   - version bump（v0.2.0 → v0.x）
3. **privacy**：不含本机路径 / 用户名 / 平台字符串
4. **gate**：`hub check bundle --allow-unsigned` PASSED
5. **截图**：`bundle/screenshots/` 含真实 PNG（`--test-action capture:` 抓的）

## 6. 验收标准（Phase 1 整体）

- [ ] R-1 美股实时源接入 + 延迟验证（秒级实时）
- [ ] R-2 K 线 60FPS 或 TODO 占位
- [ ] 12 界面全部可启动 / 可点击 / 不崩
- [ ] 设置 + 免责声明 + Launcher 三件套可访问
- [ ] `bundle_blake3` 重算 + `hub check` PASSED
- [ ] 8 张以上真实截图入库
- [ ] README 同步更新（含数据源、hosts、Phase 1 完成清单）
- [ ] 本地分支 commit 完毕，**未推远端**

## 7. 时间线（建议）

1. **Step 1**（now）：housekeeping commit ✅ 完成（`6a49928` / `d0ffd33`）
2. **Step 2**（now）：写 MVP-TODO.md ← 当前
3. **Step 3**：派 R-1 + R-2 调研 agent（并行）
4. **Step 4**：派 I-1/I-8/I-9/I-10 简单界面 agent（C）
5. **Step 5**：派 I-2/I-3/I-4/I-5 内容界面 agent（D）
6. **Step 6**：派 I-6/I-7 调研依赖界面 agent（E）
7. **Step 7**：派 I-11 事件流 stub（条件）
8. **Step 8**：每步后派 review agent
9. **Step 9**：主对话收集 + 写最终汇报

## 8. 参考

- **PRD**：`OctoSense Market Agent - MVP产品需求文档.md`
- **Octoscript 文档**：`OctoScript-App-Design-Flow/docs/SCRIPT-API.md` / `QUICKSTART.md`
- **Octoscript 样例**：`OctoScript-App-Design-Flow/examples/{aircon,calendar,health,reunion,school,shared}/`
- **Octoscript 教程**：`octoscript/examples/`
- **K 线思路**：参考 iced 实现的 K 线项目（仅思路，不是移植）
- **Hub 模板**：`OctoScript-App-Design-Flow/templates/script-app/`
- **当前 splash**：`apps/finance-brief/bundle/main.splash`（558 行）
- **当前 hosts**：`feed.mix.sina.com.cn, qt.gtimg.cn, api.hyperliquid.xyz, api.frankfurter.dev`
