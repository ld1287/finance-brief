# finance-brief

## OctoSense 应用 — 单一 Octoscript 路径

finance-brief 是一个 OctoSense 应用，**完全由 `.octoscript` 文件**经由
octoscript-makepad 流水线渲染呈现。只有一条渲染路径；splash DSL 入口
（`bundle/main.splash`）以及 12 个 `.card` 概览卡片已于 2026-10-04 的
`feat/one-octoscript` 迁移中移除。

| | |
|---|---|
| **UI** | `bundle/screens/` 中 12 个 `.octoscript` 界面 |
| **入口** | `apps/desktop/src/lib.rs`（对应 `Octoscript-Makepad/apps/flutter-samples/src/lib.rs`） |
| **数据** | `native/src/adapters/` 下 9 个 Rust 适配器，通过 `apps/desktop/src/datasources.rs` 以 `mod.fb.<cap>` 形式暴露给 DSL |
| **工作流** | `bundle/workflow.octoscript`（7 个函数，原本已存在；现已真正接入） |
| **流水线** | `octoscript-render`（VM → UiNode）→ `octoscript-makepad::to_makepad_ui` → `Splash.view` → native makepad widgets |
| **热重载** | 设备端 `/data/local/tmp/finance_brief.octoscript`（对应 flutter-samples 的 DEVICE_PATH） |

## 1. 项目

一个 OctoSense 受控脚本应用（`bundle/` 是唯一的提交单元；
`apps/desktop/` 是桌面端 / 手机端的打包层，对应
`Octoscript-Makepad/apps/flutter-samples/`）。聚合公开、无需密钥的金融新闻与行情 API，
覆盖五个 Tab（要闻 / A股 / 美股 / 加密 / 外汇），并提供收藏功能以及离线示例数据兜底。
所有数据仅用于演示，**不构成投资建议**。

## 2. 数据源

五个数据源，六项能力。TTL 定义于 `native/capabilities.toml`。

| 名称 | URL | 内容 | TTL |
|------|-----|---------|-----|
| Sina 要闻 | `feed.mix.sina.com.cn` | 中文财经 RSS（GBK） | `news.refresh` 300s；`news.read` 600s |
| Tencent 行情 | `qt.gtimg.cn` | A股 / 港股快照 | 5s |
| Stooq 美股 | `stooq.com` | 美股实时 CSV | 5s |
| Hyperliquid 加密 | `api.hyperliquid.xyz` | 加密货币（BTC/ETH/SOL，`allMids`） | 5s |
| Frankfurter 外汇 | `api.frankfurter.dev` | ECB 每日定盘汇率 | 3600s |

## 3. 运行

单一命令序列：

```sh
# 1) Native 适配器单元 / smoke 测试
cargo test --manifest-path native/Cargo.toml      # 65/65 PASS

# 2) 运行 Octoscript 驱动的应用（桌面端，对应 flutter-samples）
cd apps/desktop && cargo run --release

# 输出：
#   finance-brief MOUNT route=launcher src_len=NNNNN built=true
# = 12 个界面通过 octoscript-makepad 求值，并挂载到
#   Splash.view 上成为原生控件。

# 3) 手机端构建（对应 flutter-samples）
cargo makepad android run -p finance-brief --release
```

`$OCTO` 指向 `OctoScript-App-Design-Flow/tools/octo`。本仓库在交互式 UI 上
已不再依赖它。

## 4. 仓库结构

```
finance-brief/
├── apps/
│   └── desktop/                            # Rust 应用入口（对应 flutter-samples）
│       ├── Cargo.toml
│       └── src/{main.rs, lib.rs, datasources.rs}
├── bundle/                                 # 提交单元（对应 OctoSense 应用结构）
│   ├── screens/                            # 14 个 .octoscript 文件（kit + 12 个界面 + 索引）
│   ├── workflow.octoscript                 # 7 个工作流函数（原本已存在）
│   ├── capabilities.toml                   # 能力白名单（按 capabilities.toml）
│   ├── schema/                             # 17 个 JSON schema
│   ├── kit/                                # 144 个 palette / axis token 文件
│   ├── assets/  listing.json  screenshots/
│   └── manifest.json                       # 应用元数据
├── native/                                 # Rust 适配器（5 个数据源 + cache + 9 个模块）
│   ├── src/adapters/
│   └── capabilities.toml
├── scripts/                                # 启动 / 安装脚本
├── docs/                                   # 平台分层文档
├── .todo-*.md                              # 被 .gitignore 忽略的本地协调文档
└── README.md
```

## 5. 十二个启动器界面

依据 `bundle/workflow.octoscript` 中的映射，共 12 个界面。每个界面均为
`bundle/screens/` 下的一个独立 `.octoscript` 文件。路由逻辑位于
`bundle/screens/_index.octoscript`。

| # | 界面 | 文件 |
|---|--------|------|
| 1 | Launcher（主页） | `screens/launcher.octoscript` |
| 2 | 新闻简报 list | `screens/news_list.octoscript` |
| 3 | 新闻详情 | `screens/news_detail.octoscript` |
| 4 | 研究卡 list | `screens/research_list.octoscript` |
| 5 | 研究卡详情 | `screens/research_detail.octoscript` |
| 6 | K线看盘（`mod.plot.CandlestickChart`） | `screens/kline.octoscript` |
| 7 | 行情 list（4 个 Tab：A / US / crypto / FX） | `screens/quote_list.octoscript` |
| 8 | 收藏 | `screens/favorites.octoscript` |
| 9 | 设置 | `screens/settings.octoscript` |
| 10 | 免责声明 | `screens/disclaimer.octoscript` |
| 11 | 事件流 | `screens/event_stream.octoscript` |
| 12 | 数据源状态 | `screens/datasource_status.octoscript` |

依据 `docs/R-4-l0-cards.md §5`：9 个界面为纯 L0 声明式；3 个
（K线、event_stream、datasource_status）允许 L1 算术运算。

## 6. 测试

```sh
cargo test --manifest-path native/Cargo.toml
…
test result: ok. 62 passed; 0 failed   # unit
test result: ok.  3 passed; 0 failed   # smoke
                             ─────
                             65 / 65
```

应用级 smoke：执行 `cargo run -p finance-brief` 并验证启动器呈现 11 个卡片。
点击卡片会经由 `_index.octoscript` 路由到对应界面（已通过视觉 QA 验证；本 MVP
未接入自动化截图工具）。

## 7. 已知问题

| 项目 | 状态 |
|------|--------|
| 9 个 native 适配器尚未接入 `apps/desktop/src/datasources.rs` | `host.fetch` 桩函数仅打印日志；数据仍通过 `native/capabilities.toml` 走 MVP 路径 |
| `kline.octoscript` 中 `chart_candlestick` widget 注册 | 依赖于在 app VM 上调用 `octoscript_widgets::script_mod(vm)` 与 `makepad_plot::script_mod(vm)`；暂未接入 `datasources.rs` |
| `bundle/listing.json` 的 `publisher.privacy_policy_url` | 当前为仓库的 GitHub URL；公开提交前需要独立的隐私政策文档 |
| `screens/*.octoscript` 尚未在真机上经由 `octoscript_render::build` 完整跑通 | 在标记 v0.4.0 之前需在 Android / 桌面端验证 |

## 8. Git 信息

- Branch: `feat/one-octoscript`（基于 `b19656c`）
- Tracking: `origin/main`
- HEAD on main: `b19656c` "review, re-arch ,update docs"
- 迁移基线：2026-10-04

## 9. 许可证

Apache-2.0（见 `LICENSE`）。