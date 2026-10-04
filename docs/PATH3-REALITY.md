# finance-brief 在 OctoSense shell 上的真实状态（Path 3 Reality）

> 状态文档：截至 2026-10-04，**用户约束已放宽**：上游修改走 issue+PR、main.splash 可改、sys.finance_brief.* 可用、Path 3 fullscreen 可实现。本文记录**当前已 work** + **未来可实现**两条线。

---

## §1. 当前已 work

### §1.1 Path 1（card-host 交互 UI）
- 入口：`bundle/main.splash`（572 行 splash DSL，flat-let 单文件 per `.todo-re-arch-2026-10-04.md`）
- 启动命令：`sh scripts/run-finance-brief.sh`
- 后端：9 个 Rust adapter（`native/src/adapters/`，详见 `docs/ARCHITECTURE.md` §7）
- 状态：✅ 已 work（`HEAD`）

### §1.2 Path 3 lite（OctoSense shell glance tile）
- 入口：12 个 `.card` 文件（`bundle/*.card`，静态 L0 模板，去 sys.X）
- 渲染：`octoscript-ui-l0::check_ui_l0` + `realize` + `makepad::lower` + `octoscript-makepad::to_makepad_l0_ui`（详见 `docs/ARCH-PLATFORM-REALITY.md` §1.1 + `docs/ARCHITECTURE.md` §8.1）
- 状态：✅ 已 work（2026-10-04 重写为静态模板）

---

## §2. Path 3 fullscreen launch — **可实现**（待 OctoScript#56 合并）

**含义**：finance-brief 作为 fullscreen app 从 OctoSense shell 启动，替代"glance tile 入口 → 切到 card-host"的两步流程。

**阻塞项**：OctoScript#56（splash widget tree → L0 ledger 桥接）— https://github.com/OctoSense-org/OctoScript/issues/56

**实施步骤**（#56 合并后）：
1. PR 上游到 `OctoSense-org/OctoScript`：把 `sys.finance_brief.*` 加到 `octoscript-ui-l0::catalog::ANSWERS`
2. 远程合并后，本地 pull 更新
3. finance-brief 改 `bundle/main.splash` → `bundle/main.octoscript`（或 `page.card`）作为 shell fullscreen 入口
4. finance-brief 给 `.card` 文件加 `source sys.finance_brief.*` 引用真实数据
5. shell 启动 finance-brief fullscreen，走 octoscript-makepad 完整管线

---

## §3. sys.finance_brief.* in `.card` — **可实现**（需上游 catalog 扩展）

**含义**：finance-brief 的 17 capability 映射到 sys.X 命名空间（如 `sys.finance_brief.news.refresh`），`.card` 文件可声明真实数据源而非空 glance tile。

**阻塞项**：`octoscript-ui-l0::catalog::ANSWERS`（lib.rs:3750-3911）是**编译期硬编码**的 30 个 helper 表，新加 sys.X 需改源码 + PR 上游。

**实施步骤**：
1. PR 上游到 `OctoSense-org/OctoScript`：在 `ANSWERS` 加 17 个 `sys.finance_brief.*` 条目 + 在 VM 注册 handler
2. 远程合并后，本地 pull 更新
3. finance-brief 改写 12 个 `.card` 文件，把 `source sys.news(...)` 改为 `source sys.finance_brief.news_refresh(...)` 等
4. `l0validate` 全过；glance tile 渲染含真实数据

---

## §4. host-service crate — **仍受 workspace 限制**

**含义**：仿 `OctoSense/apps/news/{bundle,host-service}` 双目录，给 finance-brief 写一个 `host-service` crate（暴露 `news / quote / stream` 等 host service）。

**阻塞项**：OctoSense shell 不支持运行时动态 host-service 注册（`OctoSense/crates/shell/src/apps.rs:184-220` 编译期绑定）。需用户授权修改 OctoSense/Cargo.toml 添加 workspace member。

**状态**：🎯 等用户授权。

---

## §5. 真正的迁移路径（按依赖关系排序）

| 步骤 | 动作 | 阻塞项 | 预计工作量 |
|------|------|--------|-----------|
| 1 | PR 上游：`sys.finance_brief.*` 加 catalog | 上游 review | 1-2 周 |
| 2 | PR 上游：OctoScript#56 桥接 | 上游 review | 1-2 周（已开） |
| 3 | 本地：12 `.card` 加 `source sys.finance_brief.*` | #1 完成 | 半天 |
| 4 | 本地：main.splash → main.octoscript（572 行 splash DSL → L0 ledger） | #2 完成 | 2-3 天 |
| 5 | 本地：shell fullscreen 接入（仿 news/{bundle,host-service}） | #4 + 用户授权 workspace | 1 天 |

---

## §6. 数据源现状

5 个免费源 + 1 个 socket 协议（per `bundle/manifest.json#network.hosts` + `native/src/adapters/`）：

| # | 源 | URL / 协议 | adapter |
|---|----|-----------|---------|
| 1 | Sina 财经要闻 | `https://feed.mix.sina.com.cn/api/rollout?...` | `news_sina` |
| 2 | Tencent 行情快照 | `https://qt.gtimg.cn/q={symbols}` | `quote_tencent` |
| 3 | Hyperliquid 加密币 | `https://api.hyperliquid.xyz/info` | `quote_hyperliquid` |
| 4 | Hyperliquid 订单流 | `wss://api.hyperliquid.xyz/ws` | `stream_tick`（orderbook 部分） |
| 5 | Frankfurter 外汇 | `https://api.frankfurter.dev/v1/latest?base=USD&symbols=...` | `quote_frankfurter` |
| 6 | Stooq 美股 | `https://stooq.com/q/l/?s={symbol}&f=sd2t2ohlcv&h&e=csv` | `quote_stooq` |

> 详细 adapter 实现见 `docs/ARCHITECTURE.md` §7。

---

## §7. 相关上游 issue + PR flow

| Issue / 任务 | 状态 | 链接 | 阻塞 |
|--------------|------|------|------|
| OctoScript#56（splash widget tree → L0 ledger 桥接） | open | https://github.com/OctoSense-org/OctoScript/issues/56 | Path 3 fullscreen + main.octoscript 迁移 |
| 上游 catalog 扩展 PR（`sys.finance_brief.*` 加 ANSWERS） | **待开** | — | sys.finance_brief.* in .card |
| OctoSense shell host-service 动态注册 | open | `OctoSense/crates/shell/src/apps.rs:184-220` | host-service crate |

**PR flow**（per 用户 2026-10-04 约束）：
- 在上游仓库开 issue 说明动机 + 设计
- 开 PR 附 issue 链接
- 等远程合并（review 由上游 owner 决定）
- 本地 pull 更新后才能用新特性

---

**TL;DR**：当前 Path 1 + Path 3 lite 两条线**都已 work**；未来 Path 3 fullscreen + sys.finance_brief.* in .card**已解锁**（走上游 PR flow），等 #56 合并后可推进。
