# finance-brief 在 OctoSense shell 上的真实状态 — 归档

> **归档档（2026-10-04）**：本文档记录 Path 1 / Path 3 lite 双路径并存的
> 中间形态，已被 `feat/one-octoscript` 分支的单一 Octoscript 方案取代。
> 仅保留历史事实陈述；当前架构以 `docs/ARCHITECTURE.md §3` 为准。

---

## §0. 迁移记录

**2026-10-04**：`feat/one-octoscript` 分支合入后，本文档所述双路径
（Path 1 = `bundle/main.splash` card-host + Path 3 lite = 12 个 `.card`
glance tile）已全部删除。当前 finance-brief 仅有**一条**渲染路径：
`bundle/screens/*.octoscript` × 14 经 `octoscript-render` 评估 + `octoscript-makepad`
翻译 + `Splash.view` 挂载。

后续 shell fullscreen launch 仍依赖 OctoScript#56 桥接（§2），不在
v3 单一方案中处理。

---

## §1. 历史：曾 work 的双路径（2026-10-04 之前）

### §1.1 Path 1（card-host 交互 UI）—— 已删除

- 入口：`bundle/main.splash`（572 行 splash DSL，flat-let 单文件）
- 启动命令：`sh scripts/run-finance-brief.sh`
- 后端：9 个 Rust adapter（`native/src/adapters/`）
- 状态：✅ 曾在 HEAD `b19656c` work
- 现状：**已删除**（2026-10-04，`feat/one-octoscript`）

### §1.2 Path 3 lite（OctoSense shell glance tile）—— 已删除

- 入口：12 个 `.card` 文件（`bundle/*.card`，静态 L0 模板）
- 渲染：`octoscript-ui-l0::check_ui_l0` + `realize` + `makepad::lower` + `octoscript-makepad::to_makepad_l0_ui`
- 状态：✅ 曾 work（2026-10-04 重写为静态模板）
- 现状：**已删除**（同次迁移）

---

## §2. 未来 shell fullscreen launch（OctoScript#56 解锁后）

**含义**：finance-brief 作为 fullscreen app 从 OctoSense shell 启动，
替代"glance tile 入口 → 切到 card-host"的两步流程。

**阻塞项**：OctoScript#56（splash widget tree → L0 ledger 桥接）。

> **状态**：🎯 目标。`feat/one-octoscript` 后此阻塞项依然存在 —— shell
> 集成与 Octoscript 方案正交。#56 合并后，shell 启动 finance-brief
> fullscreen 走 octoscript-makepad 完整管线，`apps/finance-brief/`
> 已经能直接被 shell 加载。

---

## §3. 历史：`sys.finance_brief.*` in `.card` —— 不再需要

**历史含义**：finance-brief 17 capability 映射到 sys.X 命名空间
（如 `sys.finance_brief.news.refresh`）。

**v3 后**：不再需要。`mod.fb.<cap>` 命名空间直接在 `apps/finance-brief/src/datasources.rs`
注册到主 VM，与 sys.X 命名空间解耦。catalog::ANSWERS 硬编码表不必扩展。

---

## §4. 历史：host-service crate —— 不在 v3 scope

**含义**：仿 `OctoSense/apps/news/{bundle,host-service}` 双目录，
给 finance-brief 写一个 `host-service` crate（暴露 `news / quote / stream`
等 host service）。

**阻塞项**：OctoSense shell 不支持运行时动态 host-service 注册（编译期绑定）。

> **v3 状态**：跳过。`apps/finance-brief/` 自身就是 host service 的承载
> （dialog 等价物），不需要单独 crate。

---

## §5. 数据源现状（未变）

同 `docs/ARCHITECTURE.md §11` / README §2。

---

## §6. 相关上游 issue（仍相关）

| Issue | 状态 | 链接 |
|-------|------|------|
| OctoScript#56 | open | https://github.com/OctoSense-org/OctoScript/issues/56 |
| OctoSense shell host-service 动态注册 | open | `OctoSense/crates/shell/src/apps.rs:184-220` |

---

**TL;DR**：Path 1 + Path 3 lite 双路径已于 2026-10-04 在 `feat/one-octoscript`
分支删除并替换为单一 Octoscript 方案。当前架构以 `docs/ARCHITECTURE.md`
为准，本文档仅作历史归档。