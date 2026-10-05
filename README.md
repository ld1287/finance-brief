# finance-brief

## OctoSense 应用 — 单一 Octoscript 路径

finance-brief 是一个 OctoSense 应用，**完全由 `.octoscript` 文件**经由
octoscript-makepad 流水线渲染呈现。只有一条渲染路径；splash DSL 入口
（`bundle/main.splash`）以及 12 个 `card` 概览卡片已于 2026-10-04 的
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

四种模式，CLI 全在 `scripts/*.py`（2026-10-05 从原 `.sh` 全部转为 `.py`，stdlib only，跨 Windows / macOS / Linux）。

### 3.1 独立运行（桌面端窗口）

```sh
# 1) Native 适配器单元 / smoke 测试
cargo test --manifest-path native/Cargo.toml      # 65/65 PASS

# 2) 桌面端窗口（对应 flutter-samples）
cd apps/desktop && cargo run --release
#   输出：finance-brief MOUNT route=launcher src_len=NNNNN built=true
# = 12 个界面通过 octoscript-makepad 求值，并挂载到 Splash.view 成为原生控件。
```

### 3.2 注册到 OctoSense shell 启动器

```sh
# 用户级注册（不写 dep 仓库；写到 ~/.octosense/apps.json）
python scripts/install-as-makepad-app.py

# shell 启动时优先读 ~/.octosense/apps.json
cd ../OctoSense && cargo run --release -p octosense
# → 启动器 tile "财经简报" 点击后拉起 finance-brief.exe 独立窗口
```

`install-as-makepad-app.py` 做：① `cargo build --release` ② 检查本地 `../makepad` HEAD 与 `apps/desktop/Cargo.toml` pin 的 rev 对齐（OctoSense/AGENTS.md §2 "One revision per external dependency"）③ 写 `~/.octosense/apps.json` 条目（label=财经简报，executable=绝对路径）。支持 `--dry-run` / `--uninstall` / `--help`。

> **v6 修复（2026-10-05）**：之前 session 误跑 `install-as-system-app.py`（legacy Page-format 脚本）会在 `OctoSense/desktop/system-apps.json` 注册 finance-brief 并拷 bundle 到 `OctoSense/apps/finance-brief/bundle/`，但源 bundle 没有 `launcher.card` 所以 page.card 也没创建，shell 把 system-apps.json 当 system app 加载时报 `page.card 系统找不到指定文件 (os error:2)`。
>
> **正确路径**：finance-brief 是 developer program（不是 system app），应该走 `~/.octosense/apps.json` 的 `executable=` 字段。`install-as-makepad-app.py` 已正确实现此路径。
>
> **如果 shell 仍报 page.card**：
> 1. `python scripts/install-as-system-app.py --uninstall`（v6 新加）—— 清理 OctoSense 侧污染
> 2. 删 `~/.octosense/apps/.system/os.finance-brief/`（如果存在）—— 清理 shell 缓存
> 3. 重跑 `python scripts/install-as-makepad-app.py` —— 重新写 user catalog

> **为什么不直接写 `OctoSense/desktop/config/apps.json`?**  
> `config/apps.json` 是 `OctoSense` 仓文件，finance-brief 不是它的 owner。修改 dep 仓代码会引入同步负担（finance-brief 升级 → OctoSense 跟着改 → PR 流程）。改用 `~/.octosense/apps.json`（用户级 catalog，OctoSense shell 的 catalog 查找优先级在 `config/apps.json` 之前）写注册 — `register-with-shell.py` 是这个入口；`install-as-makepad-app.py` 是它的 build+register 一步式版本。

### 3.3 Headless 验证（remote bridge）

不需要窗口，远程验证 widget tree / 像素：

```sh
# 后台启动 finance-brief，绑 remote bridge
nohup ./apps/desktop/target/release/finance-brief.exe --remote=0 > /tmp/fb.log 2>&1 &
disown
sleep 5

# 从 log 提端口（--remote=0 = 桥派 ephemeral port）
PORT=$(grep -oE 'listening on 127.0.0.1:[0-9]+' /tmp/fb.log | grep -oE '[0-9]+$' | head -1)

curl http://127.0.0.1:$PORT/status     # 窗口状态 + 大小
curl http://127.0.0.1:$PORT/snap       # widget tree
curl http://127.0.0.1:$PORT/g          # 抓 PNG（capture_kind=backend_png）
```

或者用 `python scripts/drive-test.py <port>` 自动跑 launcher 5 个 tab 点击序列，或 `python scripts/verify.py <port>` 验证 5 个行情 tab + 刷新按钮的显示内容。

**v8 验证结果**：Splash widget r=[0,29,440,997]，11 个 OctoscriptTap tile r=[24,167,192,96]/[224,167,192,96]/...，`/g` PNG 抓到完整 launcher UI（标题 + 4 个 section + 12 个 tile）。详见 `.todo-finance-brief-splash-zero-rect-2026-10-05.md` §3 改动 5 + §6 验收。

### 3.4 手机端

```sh
cargo makepad android run -p finance-brief --release
```

`$OCTO` 指向 `OctoScript-App-Design-Flow/tools/octo`。本仓库在交互式 UI 上已不再依赖它。

## 4. 脚本（`scripts/*.py`）

2026-10-05 从原 `.sh` 全部转为 `.py`：stdlib only（argparse / subprocess / urllib.request / pathlib / shutil），Windows 不需 Git Bash / WSL 直接 `python foo.py` 即可跑。每个脚本支持 `--help`，参数错 exit 2，bridge 不可达 exit 7。

| 脚本 | 用途 |
|------|------|
| `install-as-makepad-app.py` | **当前推荐**。Build + 注册 finance-brief 到 OctoSense shell 用户级 catalog（`~/.octosense/apps.json`）。含 makepad rev 对齐检查 + UTF-8 + Python bool fix（Windows 中文 label 兼容）。 |
| `register-with-shell.py` | 把 finance-brief 注册到 OctoSense shell 用户级 catalog（`~/.octosense/apps.json`），不 rebuild。详细见 [scripts/README.md](scripts/README.md)。 |
| `install-as-system-app.py` | **legacy**。把 bundle 拷到 `OctoSense/apps/finance-brief/bundle/`（写 dep 仓库），为老 Page-format bundle（`*.card` 文件）设计。已加防御式：找不到源文件就 warning 跳过不崩；当前 Path-1 bundle 下装到空 bundle，WARNING 提示用 `install-as-makepad-app.py`。（支持 `--uninstall`，v6 新增） |
| `run-octosense.py` | 调 `install-as-makepad-app.py` + `cargo run --release -p octosense`。当前 Path-1 bundle 下推荐直接 `cargo run -p octosense`（已通过 `install-as-makepad-app.py` 注册过 finance-brief 即可）。 |
| `drive-test.py <port>` | 通过 remote bridge 自动点 launcher 5 个 tab，验证屏幕 label 序列。 |
| `verify.py <port>` | 通过 remote bridge 验证 5 个行情 tab + 刷新按钮（坐标硬编码，对应 `quote_list` 屏幕布局）。 |

旧 `.sh` 文件已删除。

## 5. 仓库结构

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

## 6. 十二个启动器界面

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

## 7. 测试

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

## 8. 已知问题

| 项目 | 状态 |
|------|--------|
| 9 个 native 适配器尚未接入 `apps/desktop/src/datasources.rs` | `host.fetch` 桩函数仅打印日志；数据仍通过 `native/capabilities.toml` 走 MVP 路径 |
| `kline.octoscript` 中 `chart_candlestick` widget 注册 | 依赖于在 app VM 上调用 `octoscript_widgets::script_mod(vm)` 与 `makepad_plot::script_mod(vm)`；暂未接入 `datasources.rs` |
| `bundle/listing.json` 的 `publisher.privacy_policy_url` | 当前为仓库的 GitHub URL；公开提交前需要独立的隐私政策文档 |
| `screens/*.octoscript` 尚未在真机上经由 `octoscript_render::build` 完整跑通 | 在标记 v0.4.0 之前需在 Android / 桌面端验证 |
| Splash 子树在 headless `/snap?all=1` 里 rect 全 0×0 | **已修复 (v8, 2026-10-05)**。根因 `apps/desktop/src/app.rs:60` 的 `View{height:Fit, {ui}}` 包裹 + `view.walk = host.walk` walk 冻结 + 无 `WindowGeomChange` remount；flutter-samples 同样包裹 Fit 但其 kit content 用 plain View+像素高，与 finance-brief `fb_page` 的 `ScrollYView+fillh:1` 不同。5 处改动见 `.todo-finance-brief-splash-zero-rect-2026-10-05.md` §3。验证：`/snap?all=1` Splash r=[0,29,440,997]、11 个 tile r=[24,167,192,96]/[224,167,192,96]/...、`/g` PNG 抓到完整 launcher UI |
| `install-as-system-app.py` 是 legacy Page-format 脚本，对 Path-1 bundle 无效 | 不要运行；如果已经运行过污染了 OctoSense 状态，用 `python scripts/install-as-system-app.py --uninstall` 清理（v6 新增） |
| 点击 launcher tile 不切换屏幕 | **已知限制 (2026-10-05)，pending dep fix**。v13 DBG 确认 `mem::replace(&mut host.view, view)` 真的换了 view（每次 mount pre→post view_uid 都不同），但 `/d` 和 `/g` PNG 在 click 后仍显示 launcher。v14 加 `compact_dump` 诊断发现 click 时 graph 只多了 11 widgets（host shell）；v15 把 host 从 `Splash` 改成 `View` 后 graph 能正常累计（每 mount 100+ widgets），但 `/d` 仍 byte-identical before vs after click。v16 加 `cx.widget_tree().set_root_widget(host_ref.clone())` 在 `mem::replace` 之前——cargo test 11/11 pass，但运行时 `/d` before vs after click 仍 SHA256-identical，路由仍不切换。已尝试 v10 / v12 / v15 / v16 四种策略，全部在 dep 层 widget_tree.rs 上撞墙：`refresh_from_borrowed` 用 stale uid 调 `parent.placeholder=true` 节点（L1622-1626 early-return），新 route 的 View uid 永不被 graph 收录。已写 RFC `docs/rfc-splash-view-swap.md`（3 个候选修复方案 A1/B/C2 + 8 章节根因分析）+ issue 草稿 `docs/issue-splash-view-swap.md`，等待 makepad / Octoscript-Makepad 维护者决定方案后再合并 |

## 9. Git 信息

- Branch: `feat/one-octoscript`（基于 `b19656c`）
- Tracking: `origin/main`
- HEAD on main: `b19656c` "review, re-arch ,update docs"
- 迁移基线：2026-10-04

## 10. 许可证

Apache-2.0（见 `LICENSE`）。
