# 财经简报 (finance-brief)

一个 OctoSense 受控脚本 App：以 **12 屏 L0 ledger `.card`** 描述 UI，桥接
Rust 侧的 **5 个公开数据源**，把免密财经要闻与行情聚合成可收藏的桌面卡组。
整个 App 走 `card-host` 路线，不再使用过期的 `main.splash` widget tree。

- 屏清单：`bundle/*.card`（12 个 L0 ledger 文件）
- 清单：`bundle/manifest.json`、`bundle/listing.json`
- 能力契约：`bundle/capabilities.toml`
- 路线图：`MVP-TODO.md`

## 1. 项目简介

- **12 屏 L0 ledger**：launcher / news_list / news_detail / research_list /
  research_detail / kline / quote_list / favorites / settings / disclaimer /
  event_stream / datasource_status。每个屏是一个独立的 `.card` 文件，遵循
  `source → state → event → copy → view` 五段顺序。
- **5 个公开数据源**：新浪财经 RSS、腾讯财经行情（A 股 / 美股，腾讯行情为
  T+1 收盘）、Stooq CSV（美股实时收盘）、Hyperliquid 永续合约行情、
  Frankfurter 外汇参考汇率（ECB 口径）。
- **无密钥 / 无 token / 无密码输入**。所有主机白名单写在
  `bundle/manifest.json` 的 `network.hosts`，越权访问会被 hub 门禁直接拒绝。
- **离线兜底**：每个数据源都有内置示例数据，断网时仍可渲染（统一标注
  `示例数据 · 非实时`）。

## 2. 数据源总表

| # | 标签 | 服务商 | 主机 | 接口 | 方法 | 编码 | 鉴权 | 缓存 TTL |
| - | ---- | ------ | ---- | ---- | ---- | ---- | ---- | -------- |
| 1 | 要闻 | 新浪财经 | `feed.mix.sina.com.cn` | `/api/roll/get` | GET | UTF-8 JSON | 无 | 300 s |
| 2 | A 股 / 美股 | 腾讯财经 | `qt.gtimg.cn` | `/q=<代码列表>` | GET | GBK 文本 | 无 | 15 s |
| 3 | 美股实时 | Stooq | `stooq.com` | `/q/l/?s=<sym>&f=sd2t2ohlcv&h&e=csv` | GET | UTF-8 CSV | 无 | 15 s |
| 4 | 加密 | Hyperliquid | `api.hyperliquid.xyz` | `/info` | POST | UTF-8 JSON | 无 | 10 s |
| 5 | 外汇 | Frankfurter (ECB) | `api.frankfurter.dev` | `/v1/latest` | GET | UTF-8 JSON | 无 | 3600 s |

## 3. 系统依赖

| 组件 | 版本 | 用途 |
| ---- | ---- | ---- |
| Rust toolchain | ≥ 1.75 | 编译 `native/` 与上游 hub / shell |
| Makepad | 随 `octoscript-makepad` 子模块 | 渲染后端 |
| OctoSense | 主分支 | 桌面 shell 与 card-host |
| octoscript | ≥ 0.3.0 | L0 ledger 解析器与 widget 白名单 |
| `tools/octo` | 来自 `OctoScript-App-Design-Flow` | 本地开发期的 HTTP 桥 |

> 工作区布局（与 OctoScript-App-Design-Flow 的 QUICKSTART 一致）：
>
> ```text
> <workspace>/
>   OctoSense-App-Hub/         hub、card-host，以及本 App (apps/finance-brief)
>   OctoScript-App-Design-Flow/ tools/octo 与设计流文档
>   makepad/  octoscript/  octoscript-makepad/    运行时（同级检出）
> ```

## 4. Clone & Run

```sh
# 1) 拉代码
git clone <this-repo> finance-brief
cd finance-brief

# 2) 构建 native（首次或 Rust 改完后）
cargo test -p finance-brief-native          # 39/39 全绿后再继续
cargo build -p octosense-card-host --release # 仅首次需要

# 3) 验证 bundle 满足商店门禁
$PATH_TO_HUB/target/release/hub stamp bundle
$PATH_TO_HUB/target/release/hub check bundle --allow-unsigned

# 4) 启动 card-host（隐藏窗口 + 本地远程桥）
python3 tools/octo run bundle --port 8141 --hidden --detach
```

`scripts/verify.sh`、`scripts/drive-test.sh` 封装了逐标签查看与交互回归，
只依赖端口参数：`sh scripts/verify.sh 8141`。结束会话务必
`curl -s localhost:8141/quit`。

## 5. 在 OctoSense 桌面 shell 中运行

两条路都试过，结论先说：

- **系统 App 路径最省事**。shell 把 `os.*` 系统 App 按摘要直接打包进二进制，
  无需发布者签名，也不经过商店门禁，推荐演示 / 联调用。
- **商店路径**（`hub publish` → 本地镜像 → shell 安装）当前因
  `listing.json.screenshots` 与 bundle 实物不一致而不能过门禁；见
  `docs/PUBLISHER-GUIDE.md` 的人工补齐清单。

```sh
# 方式一：分两步（先安装系统 App，再启动 shell）
sh scripts/install-as-system-app.sh
cd "$PATH_TO_OCTOSENSE"
cargo run --release -p octosense

# 方式二：一条命令（安装 + 构建 + 启动）
sh scripts/run-octosense.sh
```

`install-as-system-app.sh` 做三件事：

1. 把本 bundle 同步到 `apps/finance-brief/bundle/`，并把 manifest id 改为
   `os.finance-brief`（系统 App 用 `os.` 前缀；`bundle_blake3` 留空，由构建
   填写）；
2. 在 bundle 根放一份 `icon.svg`（启动器图标按这个位置查找）；
3. 把 `finance-brief` 追加进 `desktop/system-apps.json` 的 `apps`（幂等，
   保留原有系统 App）。

**在 shell 里打开它**：桌面启动后，从底部 dock、左上角 **Apps** 菜单，或
`Ctrl/Cmd+Space` 搜索里选择「财经简报」。它会作为 App Hub Card runner 里的
受控 L0 ledger 程序运行，日志可确认：

```text
wm: launched finance-brief as client 1 (in-process, card)
card: os.finance-brief running under 17 capability(ies), 5 host(s), …
```

桌面壳支持 `MAKEPAD_REMOTE=<port>` 的本地远程桥，路由与 `card-host` 相同，
可用于无人值守地驱动；`MAKEPAD_HIDE_WINDOWS` 在非 macOS 上不生效。

**移除**：从 `desktop/system-apps.json` 的 `apps` 里删掉 `finance-brief`，
删除 `apps/finance-brief/`，再重新构建即可。

## 6. 数据源 & API 服务商表

### 6.1 要闻 —— 新浪财经滚动新闻

- 服务商：新浪财经（Sina Finance）
- 请求：`GET https://feed.mix.sina.com.cn/api/roll/get?pageid=153&lid=2516&num=20&page=1`
  （`lid=2516` 为财经要闻频道；`num` 条数；`page` 翻页）
- 字段映射（`result.data[]`）：`title` → 标题、`media_name` → 来源、
  `intro` → 摘要、`url` → 原文、`ctime` → 时间戳（秒）
- 备注：字段并非条条齐全（抽样 60 条约 1 条无 `media_name`），所以脚本对每个
  字段都用「枚举对象键」的方式读取（见下文「实现要点」）。

### 6.2 A 股 / 美股 —— 腾讯财经行情（T+1 收盘）

- 服务商：腾讯财经（行情主机）
- 请求：
  - A 股：`GET https://qt.gtimg.cn/q=sh000001,sz399001,sh000300,sh600519,sz000858`
  - 美 股（腾 讯）：`GET https://qt.gtimg.cn/q=usAAPL,usMSFT,usNVDA,usTSLA,usAMZN`
- 返回形如 `v_usAAPL="200~名称~AAPL.OQ~338.40~341.07~340.37~…~-0.78~…";`，
  多个代码用 `;` 分隔、字段用 `~` 分隔。**索引 3 = 现价，索引 32 = 涨跌幅%**
  （索引 1 是名称、4 是昨收）。
- 编码：**GBK**。名称会解码成替换字符，所以 App **不用远端名称**，只用自己
  的显示名（`上证指数` / `苹果` …），只读 ASCII 数字，避免乱码问题。
- 备注：美股报价为该市场上一交易日收盘价（例如返回 `2026-09-28 16:00:01`）。

### 6.3 美股实时 —— Stooq CSV

- 服务商：Stooq（公开 CSV 行情）
- 请求：`GET https://stooq.com/q/l/?s=aapl.us&f=sd2t2ohlcv&h&e=csv`
- 返回：CSV 单行，`Open,High,Low,Close,Volume,Date,Time` 等字段。
- 备注：用于美股屏实时刷新；与腾讯美股口径一致（皆为上一交易日收盘）。

### 6.4 加密 —— Hyperliquid

- 服务商：Hyperliquid（永续合约交易所，公开信息接口）
- 请求：`POST https://api.hyperliquid.xyz/info`，
  body：`{"type":"metaAndAssetCtxs"}`
- 返回：`[0].universe[i].name`（币种名）与 `[1][i].markPx` / `prevDayPx`
  （标记价 / 昨价，字符串），下标一一对应；涨跌幅由两者相除算出。
- 备注：`allMids` 接口也可用（只给最新价、键含大量非币种条目），
  本项目用 `metaAndAssetCtxs` 以便同时拿到涨跌幅。

### 6.5 外汇 —— Frankfurter

- 服务商：Frankfurter（开源免费汇率 API，数据来自欧洲央行 ECB 参考汇率）
- 请求：`GET https://api.frankfurter.dev/v1/latest?base=USD&symbols=CNY,EUR,JPY,GBP,HKD`
- 返回：`{"amount":1.0,"base":"USD","date":"…","rates":{"CNY":…}}`
- 备注：ECB 参考汇率，**每个工作日更新一次**，不是实时撮合价；仅作参考。

## 7. 尝试过但不可用的源（含原因）

| 源 | 主机 / 项目 | 结果 |
| -- | ----------- | ---- |
| 新浪行情 | `hq.sinajs.cn` | `curl` 可用（需 `Referer`），但 App 的 TLS 客户端报 `SSL_read … unexpected eof while reading`（服务端未发 close_notify），故弃用 |
| 东方财富 | `push2.eastmoney.com`、`push2delay.eastmoney.com` | 本网络下时通时断（先 200 后连续连接失败），不稳定，已移除 |
| CoinGecko | `api.coingecko.com` | 本网络下无响应 |
| Yahoo Finance | `query1.finance.yahoo.com` | 返回拦截页（无法使用） |
| Binance / Coinpaprika | `api.binance.com`、`api.coinpaprika.com` | 本网络下无响应 |
| 网易财经 | `api.money.126.net` | 返回空 |
| 通达信行情协议（rustdx 等） | 裸 TCP 二进制私有协议 | **受控 App 不可用**：App 只能访问 `manifest` 里声明的 `https` 主机，且 store 规则禁止裸 socket |

> 若某个源被替换，请同步更新 `manifest.json` 的 `network.hosts`——门禁会拒绝
> 源码中出现的未声明主机。

## 8. 项目结构

仓库根目录按职责拆成下面几块：

| 路径 | 角色 |
| ---- | ---- |
| `bundle/launcher.card` | L0 ledger：启动器（12 屏入口） |
| `bundle/news_list.card` | L0 ledger：新闻简报列表 |
| `bundle/news_detail.card` | L0 ledger：新闻详情 |
| `bundle/research_list.card` | L0 ledger：研究卡列表 |
| `bundle/research_detail.card` | L0 ledger：研究卡详情（dataset 形态） |
| `bundle/kline.card` | L0 ledger：K 线（line chart 占位） |
| `bundle/quote_list.card` | L0 ledger：行情看盘（quote + movers） |
| `bundle/favorites.card` | L0 ledger：本地收藏（watchlist） |
| `bundle/settings.card` | L0 ledger：设置（语言 / 刷新 / 关于） |
| `bundle/disclaimer.card` | L0 ledger：免责声明（静态） |
| `bundle/event_stream.card` | L0 ledger：事件流（news_digest + step） |
| `bundle/datasource_status.card` | L0 ledger：5 数据源健康监控 |
| `bundle/capabilities.toml` | 17 capability 契约（v1 14 + Q-B 新增 3） |
| `bundle/manifest.json` | App 元数据：ID、入口、版本、`network.hosts` 白名单 |
| `bundle/listing.json` | 商店展示信息：标题、简介、截图、分类 |
| `bundle/assets/` | 图标、字体等静态资源（随 bundle 一起分发） |
| `native/src/lib.rs` | CDylib 入口，导出 `run()` 符号 |
| `native/src/host.rs` | Splash ↔ Rust 桥：注册 `host.fetch` / `host.call` 服务 |
| `native/src/model.rs` | 内部数据类型（行情、新闻、收藏条目） |
| `native/src/parse.rs` | 五大数据源解析器（新浪 / 腾讯 / Stooq / Hyperliquid / Frankfurter） |
| `native/src/store.rs` | 收藏 / 设置的 JSON 持久化 |
| `native/src/synth.rs` | 无网络时的示例数据合成 |
| `native/src/adapters/` | capability 适配器：`news_sina` / `quote_tencent` / `quote_stooq` / `quote_hyperliquid` / `quote_frankfurter` / `synth_candles` / `stream_subscribe` / `stream_tick` / `datasource_status` / `builtin` |
| `scripts/run-octosense.sh` | 一键：安装系统 App + 构建 + 启动 shell |
| `scripts/install-as-system-app.sh` | 安装本 bundle 为 shell 的 `os.finance-brief` |
| `scripts/verify.sh` / `scripts/drive-test.sh` | 逐标签查看与交互回归（依赖端口） |
| `docs/CATALOG-SYS-X.md` | 32 条 `sys.X` 字段参考（屏适配用） |
| `docs/ARCH-PLATFORM-REALITY.md` | render pipeline 真相（节选） |
| `docs/PUBLISHER-GUIDE.md` | `listing.json` 发布者字段人工填写指南 |
| `MVP-TODO.md` | 路线图与剩余项 |

L0 ledger 与 Rust 各司其职：网络 IO、解析、持久化全部在 Rust 侧，`.card`
只负责 UI 渲染和编排调用，避免把脆弱的胶水代码塞进 DSL。

## 9. Splash ↔ Rust 通讯接口

`.card` 屏只通过 `host.fetch` / `host.call` 与 Rust 通信，没有别的桥接面。
当前注册了 **17 个 capability 名**（v1 14 + Q-B 新增 3 `stream.*`，其中
`quote.snapshot` 由 3 个 adapter 各贡献一段，故计为 3 条；`fav` / `settings`
各 2 条命名共享同一 segment）。

| Capability | 方向 | 用途 | Adapter |
| ---------- | ---- | ---- | ------- |
| `news.refresh` | fetch | 拉取并解析新浪财经滚动要闻（带缓存） | `news_sina` |
| `news.read` | fetch | 按 ID 读取单条新闻 | `news_sina` |
| `quote.snapshot` (A 股 / 美股) | fetch | 腾讯行情快照（含美股 Stooq 兜底） | `quote_tencent`, `quote_stooq` |
| `quote.snapshot` (加密) | fetch | Hyperliquid 永续行情 | `quote_hyperliquid` |
| `quote.snapshot` (外汇) | fetch | Frankfurter ECB 汇率 | `quote_frankfurter` |
| `quote.candles` | fetch | K 线（`synth_candles` 合成示例作兜底） | `synth_candles` |
| `research.list` | fetch | 研究卡列表 | `builtin` |
| `research.read` | fetch | 研究卡详情（dataset 形态） | `builtin` |
| `stream.subscribe` | call | 订阅事件流频道 | `stream_subscribe` |
| `stream.unsubscribe` | call | 退订事件流频道 | `stream_subscribe` |
| `stream.frequency.set` | call | 调整事件流推送频率 | `stream_subscribe` |
| `stream.tick` | fetch | 拉取事件流最新一帧 | `stream_tick` |
| `datasource.status` | fetch | 5 数据源健康监控 | `datasource_status` |
| `fav.list` | call | 列出当前全部收藏 | `builtin` |
| `fav.toggle` | call | 切换某条目的收藏状态 | `builtin` |
| `settings.load` | call | 读取持久化设置 | `builtin` |
| `settings.save` | call | 写入持久化设置 | `builtin` |

约束：

- `host.fetch` 走 `manifest.json` 的 `network.hosts` 白名单，越权会被 hub
  拦截。
- `host.call` 参数 / 返回值都是 JSON；Rust 侧统一用 `serde_json` 收口。
- `.card` 里所有调用必须显式 `await`，DSL 不提供并发原语。

## 10. L0 `.card` ledger 硬约束

`.card` 是受限 L0 ledger DSL，写起来有 **5 条核心硬约束**，踩到就编译错
或运行时崩（语法权威参考：
`octoscript/crates/octoscript-ui-l0/tests/fixtures/*.card`）：

1. **5 段顺序固定**：`source → state → event → copy → view`。顺序颠倒或
   缺段都会被 L0 解析器拒编。`launcher.card` 是最小可工作样例，其余 11 屏
   均沿用此结构。
2. **Widget 白名单**：`Surface(pad: .page)` 是顶层容器，行 / 列容器是
   `Col { ... }` 与 `Row(gap: N) { ... }`，文本 widget 只有
   `TextTitle` / `TextBody` / `TextCaption`，可点击卡片用
   `Card(on_tap: <event>, value: <v>)`。**禁用** `View` / `Label` /
   `flow:` / `width: Fill` / `{{state.x}}` 占位符 / `|| nav.push(...)`
   闭包等旧 widget-tree 写法。
3. **保留字不能当标识符**：`if` / `for` / `fn` / `on_render` / `host` 等
   都是关键字，重名直接拒编。命名 event 时也要避开，例如 `open_news_list`
   是合法 event 名，而 `if_open_news` 不是。
4. **十六进制颜色必须带 `#x` 前缀**（如 `#xff5500`），裸 `#ff5500` 不会被
   识别为颜色字面量，会按普通标识符报错。
5. **没有 `substr` / `min` / `sin` 等内建函数**：字符串切片、数值最小值、
   三角函数这些都得在 Rust 侧实现后通过 `host.call` 暴露，DSL 自己没有。
   同样地，**不支持 `import` / 多文件**：每个屏必须装在自己那一个 `.card`
   文件里，想拆模块只能在 Rust 侧做。

另外几条软的、踩过的坑（开发时常被绊到）：

- 读不存在的属性会**直接报错**，不是返回 `nil`，`if o.k != nil` 拦不住，
  必须用 `get(o, "k", fallback)`。
- 字符串拼接用 `..`，不是 `+`；`+` 只对数值生效。
- 列表字面量是 `[a, b, c]`，没有 `array(…)` 构造函数。
- 加载 / 错误状态必须显式表达，例如 `when source.$state == .pending { ... }`，
  L0 不会替你推断。
- `:=` 只能在 `on_render` 闭包外用；渲染闭包内赋值会被判定为有副作用而拒绝
  （避免重渲染时反复触发），需要可变状态就提到外层 `state` 段。

## 11. 数据刷新频率设置

- **快讯（`event_stream` 屏）默认 1 s/帧**，可在 `#9 settings` 屏切换为
  2 s / 5 s / 10 s，对应 `stream.frequency.set` 调用。
- **行情快照（`quote_list` 屏）默认 500 ms/symbol**，可在 `#9 settings`
  屏切换为 1 s / 2 s / 5 s（更长间隔减小网络压力）。
- 切换会立即下发 `stream.frequency.set`，无需重启屏。
- 离线或来源不可用时保留上次缓存，并显示内置示例（每条都明确标注
  **示例数据 · 非实时**）。

## 12. 已知限制 + 免责声明

**已知限制**

- 当前 `bundle/listing.json.screenshots` 字段引用的真实截图需发布者在有
  GPU 纹理回读的环境下补齐；本仓库不附带截图二进制。详细补齐流程见
  `docs/PUBLISHER-GUIDE.md`。
- `listing.json` 的发布者字段仍是占位文案，平台声明为 `linux`，需人工
  确认（见 `docs/PUBLISHER-GUIDE.md`）。
- 未在手机上运行（受控 bundle 不支持侧载）。
- 美股两个数据源（腾讯 + Stooq）均为上一交易日收盘，不是盘中实时撮合价。

**免责声明**

数据来自上述公开免密接口，仅用于演示与技术验证，**不构成任何投资建议**。
各数据源的版权与使用条款归其服务商所有。

## 13. 提交与分支管理

本仓库由多个 AI agent 协作开发，提交走统一流程：

- **主 AI 自己不 commit**：负责实现 / 改代码 / 改文档的 agent 一律不直接
  `git commit`，改完即停下。
- **由 commit agent 统一 commit**：专门的 commit agent 收集变更、按
  Conventional Commits 风格生成 message、统一落到分支上。
- **不动全局 `user.name` / `user.email`**：所有 commit 都走仓库级
  `git config user.name` / `user.email`，绝不去碰 `~/.gitconfig`。
- **不 push**：agent 不执行 `git push`，是否推送由人在本地或服务端触发。
- **并发上限**：同时在跑的 agent ≤ 2，避免在同一文件上互相覆盖；超出的
  任务走队列。
- **分支**：功能 / 修复在各自的 `feature/*` / `fix/*` 分支上推进，合并走
  PR / 评审，不直接往主分支 force-push。

这样能让多 agent 并行开发时，提交历史仍可追溯、不会互相覆盖。
