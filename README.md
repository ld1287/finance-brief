# finance-brief

## 1. 项目

OctoSense 受控脚本 App（`bundle/` 为唯一提交单元）。把公开免密接口的财经要闻与行情聚合到「要闻 / A股 / 美股 / 加密 / 外汇」五个标签下，支持收藏与离线示例数据回退；所有数据仅用于演示，不构成投资建议。

## 2. 数据源

5 个数据源、6 个 capability（`quote.snapshot` 由 3 个 adapter 各自承担一类标的）。TTL 来自 `native/capabilities.toml`。

| name | URL | 内容 | TTL |
|------|-----|------|-----|
| Sina 要闻 | `feed.mix.sina.com.cn` | 中文财经要闻 RSS（GBK） | `news.refresh` 300s；`news.read` 600s |
| Tencent 行情 | `qt.gtimg.cn` | A股 / 港股快照 | 5s |
| Stooq 美股 | `stooq.com` | 美股实时 CSV | 5s |
| Hyperliquid 加密 | `api.hyperliquid.xyz` | 加密货币（BTC/ETH/SOL 等，`allMids`） | 5s |
| Frankfurter 外汇 | `api.frankfurter.dev` | ECB 日定盘汇率 | 3600s |

## 3. 启动

三个命令即可运行（按顺序）：

```sh
# 1) 单元 / smoke 测试
cargo test --manifest-path native/Cargo.toml      # 65/65 PASS

# 2) 编译 card-host harness（一次性，之后产物在 target/debug/card-host）
cd OctoSense-App-Hub && cargo build -p octosense-card-host

# 3) 加载 finance-brief/bundle 并通过 makepad-remote 暴露 :8180
./target/debug/card-host \
    --bundle    /home/lumina/octoOs/finance-brief/bundle \
    --app-data  /tmp/finance-brief-cardhost \
    --allow-unsigned --stamp --remote 8180
# 输出："finance-brief 0.3.0 admitted" + "isolate jailed" + "[SPLASH] eval: 6841 bytes"
#     = splash parse 成功，bundle 可在 App-Hub 上运行

# 4) 远程探测窗口 / 状态（可选）
curl -s http://127.0.0.1:8180/s | python3 -m json.tool
# 注：headless 环境下 grab PNG（/g）会因 GPU 渲染管线 (MESA/ZINK) 失败，
#     但 widget-draw phase 已被 splash 触发，draw-tree 已 instantiate。
```

`$OCTO` 指向 OctoScript-App-Design-Flow 仓库内的 CLI，本仓库不持有。

## 4. 项目结构

```
finance-brief/
├── bundle/                 # 提交到 App Hub 的唯一单元
│   ├── main.splash         # 入口（host.call 15 个 wrapper）
│   ├── capabilities.toml   # capability 白名单（v1 14 + Q-B 3）
│   └── listing.json        # publisher 元数据
├── native/                 # Rust adapters（5 数据源 + cache）
│   └── src/adapters/       # news_sina / quote_tencent / quote_stooq /
│                           # quote_hyperliquid / quote_frankfurter + …
├── scripts/                # 启动 / 安装脚本
├── docs/                   # 平台分层与 catalog 文档
└── MVP-TODO.md             # v0.1.0 → v2 octoscript 平台迁移计划
```

## 5. launcher 12 屏

12 屏为 `MVP-TODO.md §5` 的目标拆解。当前 `bundle/main.splash` 的 launcher 实际暴露 6 tile + 备用槽 = 7 nav 目标；4 屏（新闻详情 / 研究详情 / K线 / 免责声明）尚未进入 launcher。任意子屏目前统一通过 `render_screen_placeholder()` 渲染占位（`(host loads the corresponding .card)`）。

| # | screen | 当前状态 |
|---|--------|----------|
| 1 | Launcher（home） | splash 内已渲染 |
| 2 | 新闻简报列表 (`news_list`) | launcher tile；占位 |
| 3 | 新闻详情 | 未在 launcher；占位 |
| 4 | 研究卡列表 (`research_list`) | launcher tile；占位 |
| 5 | 研究卡详情 | 未在 launcher；占位 |
| 6 | K线看盘 | 未在 launcher（依赖 R-2 调研） |
| 7 | 行情列表合并 (`quote_list`) | launcher tile（`行情看盘`）；占位 |
| 8 | 收藏 (`favorites`) | launcher tile；占位 |
| 9 | 设置 (`settings`) | launcher tile；占位 |
| 10 | 免责声明 | 未在 launcher |
| 11 | 事件流 (`event_stream`) | launcher tile；占位 |
| 12 | 备用槽 (`datasource_status`) | launcher tile（`数据源状态页`）；占位 |

## 6. 测试

```text
cargo test --manifest-path native/Cargo.toml
…
test result: ok. 62 passed; 0 failed   # unit
test result: ok.  3 passed; 0 failed   # smoke
                             ─────
                             65 / 65
```

## 7. 已知问题

| 项 | 状态 |
|----|------|
| **B-0** `main.splash` `mod.lib.X.Y` dotted assign 不被 L0 方言接受 | 已修（flat `let` + property-assigned methods） |
| **B-4** OctoSense-App-Hub gate 把 `$schema` / `$id` 当外部资源拒收 | issue body 待 file（目标仓：OctoSense-org/OctoSense-App-Hub） |
| **B-7** `hub check` 偷偷改写 manifest `bundle_blake3` | issue body 待 file（同上） |
| Phase 4-6 | 阻塞中，依赖 OctoSense-org/OctoScript#56（`sys.*` helpers runtime API 未合并） |
| `bundle/listing.json` `publisher.privacy_policy_url` | 当前指向仓库 GitHub URL，未挂独立隐私政策文档 |

## 8. Git 信息

- 分支：`main`
- 相对 `origin/main`：ahead 24，未推送
- 当前 HEAD：`6b6bf36 Phase B-5: inline 10 launcher tiles + real data list rendering`

## 9. License

Apache-2.0（见 `LICENSE`）。