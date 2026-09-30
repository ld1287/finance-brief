# 财经简报 (finance-brief)

一个 OctoSense 受控脚本 App（Splash / OctoScript 编写）：把公开、免密的财经
要闻与行情聚合成五个标签页，点任意一条进入详情，可收藏并在重启后保留。

- 源码：`bundle/main.splash`
- 清单：`bundle/manifest.json`、`bundle/listing.json`
- 需求与设计：`BRIEF.md`；进度与已知问题：`TODO.md`；路线图：`MVP-TODO.md`

## Clone & run

```sh
# 1) 拉代码
git clone <this-repo> finance-brief
cd finance-brief

# 2) （可选）当作独立 App 试用 — 见 scripts/run-octosense.sh
#    或安装为 OctoSense 桌面壳里的 system app — 见 scripts/install-as-system-app.sh

# 3) 验证 bundle 满足商店门禁
<path-to-hub>/target/release/hub stamp bundle
<path-to-hub>/target/release/hub check bundle --allow-unsigned
```

详细步骤见下文 **「在 OctoSense 桌面 shell 中运行」**。

## 功能

| 标签 | 内容 |
| --- | --- |
| 要闻 | 新浪财经滚动新闻（标题 / 来源 / 时间 / 摘要 / 原文链接） |
| A股 | 上证指数、深证成指、沪深300、贵州茅台、五粮液 |
| 美股 | 苹果、微软、英伟达、特斯拉、亚马逊 |
| 加密 | Hyperliquid 永续合约：BTC、ETH、SOL 等（按 universe 顺序取前 12 个） |
| 外汇 | 美元对 CNY / EUR / JPY / GBP / HKD 参考汇率 |
| 收藏 | 用户收藏的条目，写入 `favs.json`，重启后仍在 |

顶部「刷新」重新抓取全部来源；另有 600 秒自动刷新。无网络或来源不可用时保留
上次缓存，并显示内置示例（每条都明确标注 **示例数据 · 非实时**）。

## 数据源总表

| 标签 | 服务商 | 主机 | 接口 | 方法 | 编码 | 鉴权 |
| --- | --- | --- | --- | --- | --- | --- |
| 要闻 | 新浪财经 | `feed.mix.sina.com.cn` | `/api/roll/get` | GET | UTF-8 JSON | 无 |
| A股、美股 | 腾讯财经（行情） | `qt.gtimg.cn` | `/q=<代码列表>` | GET | GBK 文本 | 无 |
| 加密 | Hyperliquid | `api.hyperliquid.xyz` | `/info` | POST | UTF-8 JSON | 无 |
| 外汇 | Frankfurter（数据源自欧洲央行） | `api.frankfurter.dev` | `/v1/latest` | GET | UTF-8 JSON | 无 |

所有主机都写在 `bundle/manifest.json` 的 `network.hosts` 里，能力仅
`storage` + `net`。App 不持有任何密钥 / token，也没有任何密码输入框。

## 各数据源详情

### 1. 要闻 —— 新浪财经滚动新闻

- 服务商：新浪财经（Sina Finance）
- 请求：`GET https://feed.mix.sina.com.cn/api/roll/get?pageid=153&lid=2516&num=20&page=1`
  （`lid=2516` 为财经要闻频道；`num` 条数；`page` 翻页）
- 字段映射（`result.data[]`）：`title` → 标题、`media_name` → 来源、
  `intro` → 摘要、`url` → 原文、`ctime` → 时间戳（秒）
- 备注：字段并非条条齐全（抽样 60 条约 1 条无 `media_name`），因此脚本对每个
  字段都用「枚举对象键」的方式读取（见下方「实现要点」）。

### 2. A股 / 美股 —— 腾讯财经行情

- 服务商：腾讯财经（行情主机）
- 请求：
  - A股：`GET https://qt.gtimg.cn/q=sh000001,sz399001,sh000300,sh600519,sz000858`
  - 美股：`GET https://qt.gtimg.cn/q=usAAPL,usMSFT,usNVDA,usTSLA,usAMZN`
- 返回形如 `v_usAAPL="200~名称~AAPL.OQ~338.40~341.07~340.37~…~-0.78~…";`，
  多个代码用 `;` 分隔、字段用 `~` 分隔。**索引 3 = 现价，索引 32 = 涨跌幅%**
  （索引 1 是名称、4 是昨收）。
- 编码：**GBK**。名称会解码成替换字符，所以 App **不用远端名称**，只用自己
  的显示名（`上证指数` / `苹果` …），只读 ASCII 数字，避免乱码问题。
- 备注：美股报价为该市场上一交易日收盘价（例如返回 `2026-09-28 16:00:01`）。

### 3. 加密 —— Hyperliquid

- 服务商：Hyperliquid（永续合约交易所，公开信息接口）
- 请求：`POST https://api.hyperliquid.xyz/info`，
  body：`{"type":"metaAndAssetCtxs"}`
- 返回：`[0].universe[i].name`（币种名）与 `[1][i].markPx` / `prevDayPx`
  （标记价 / 昨价，字符串），下标一一对应；涨跌幅由两者相除算出。
- 备注：`allMids` 接口也可用（只给最新价、键含大量非币种条目），
  本项目用 `metaAndAssetCtxs` 以便同时拿到涨跌幅。

### 4. 外汇 —— Frankfurter

- 服务商：Frankfurter（开源免费汇率 API，数据来自欧洲央行 ECB 参考汇率）
- 请求：`GET https://api.frankfurter.dev/v1/latest?base=USD&symbols=CNY,EUR,JPY,GBP,HKD`
- 返回：`{"amount":1.0,"base":"USD","date":"…","rates":{"CNY":…}}`
- 备注：ECB 参考汇率，**每个工作日更新一次**，不是实时撮合价；仅作参考。

## 尝试过但不可用的源（含原因）

| 源 | 主机 / 项目 | 结果 |
| --- | --- | --- |
| 新浪行情 | `hq.sinajs.cn` | `curl` 可用（需 `Referer`），但 App 的 TLS 客户端报 `SSL_read … unexpected eof while reading`（服务端未发 close_notify），故弃用 |
| 东方财富 | `push2.eastmoney.com`、`push2delay.eastmoney.com` | 本网络下时通时断（先 200 后连续连接失败），不稳定，已移除 |
| CoinGecko | `api.coingecko.com` | 本网络下无响应 |
| Yahoo Finance | `query1.finance.yahoo.com` | 返回拦截页（无法使用） |
| Stooq | `stooq.com` | CSV 接口已迁移 / 404 |
| Binance / Coinpaprika | `api.binance.com`、`api.coinpaprika.com` | 本网络下无响应 |
| 网易财经 | `api.money.126.net` | 返回空 |
| 通达信行情协议（rustdx 等） | 裸 TCP 二进制私有协议 | **受控 App 不可用**：App 只能访问 `manifest` 里声明的 `https` 主机，且 store 规则禁止裸 socket（见 SCRIPT-API「Network」） |

> 若某个源被替换，请同步更新 `manifest.json` 的 `network.hosts`——门禁会拒绝
> 源码中出现的未声明主机。

## 权限与主机白名单

```json
{
  "capabilities": ["storage", "net"],
  "network": { "hosts": [
    "feed.mix.sina.com.cn",
    "qt.gtimg.cn",
    "api.hyperliquid.xyz",
    "api.frankfurter.dev"
  ] }
}
```

- `storage`：缓存 `cache_<tab>.json` 与收藏 `favs.json`（App 私有 jail）。
- `net`：只允许访问上述四个主机；App 不含任何密钥、token 或密码字段。
- 离线示例内置于脚本（`sample_for`）：`{{assets}}` 是回环 **http** 地址，而
  `net` 策略只放行 `https`，所以随包的 `assets/*.json` 无法被 App 抓取。

## 运行

工作区布局（与 OctoScript-App-Design-Flow 的 QUICKSTART 一致）：

```text
<workspace>/
  OctoSense-App-Hub/         hub、card-host，以及本 App (apps/finance-brief)
  OctoScript-App-Design-Flow/ tools/octo 与设计流文档
  makepad/  octoscript/  octoscript-makepad/    运行时（同级检出）
```

```sh
# 1) 构建开发用容器宿主（只需一次）
cd <workspace>/OctoSense-App-Hub
cargo build --release -p octosense-card-host

# 2) 运行 App（隐藏窗口 + 本地远程桥）
cd <workspace>/OctoScript-App-Design-Flow
python3 tools/octo run <workspace>/OctoSense-App-Hub/apps/finance-brief/bundle \
  --port 8141 --hidden --detach

# 3) 通过本地 HTTP 桥驱动 / 观察（全部 GET）
curl -s 127.0.0.1:8141/snap                              # 控件与文字
curl -s "127.0.0.1:8141/m?k=click&x=101&y=139&wait=1"    # 点击（坐标为窗口布局点）
curl -s 127.0.0.1:8141/quit                              # 结束（务必）
```

`scripts/verify.sh`、`scripts/drive-test.sh` 封装了逐标签查看与交互回归，
只依赖端口参数：`sh scripts/verify.sh 8141`。

## 在 OctoSense 桌面 shell 中运行

两条路都试过，结论先说：

- **商店路径（`hub publish` → 本地镜像 → shell 安装）门禁已通过**（「至少 1 张
  listing 声明的真实 PNG」一项已被 `bundle/screenshots/01-news.png .. 06-favs.png`
  满足；`hub check` 返回 `finance-brief 0.1.0 — PASSED`，仅剩发布者占位符警告
  以及 staging 后由 `hub scan` 复检的发布者签名问题，**这两项需发布者本人完成**，
  见下文「发布前：『listing.json』要人工填写的字段」）。
- **系统 App 路径仍然最省事。** shell 把自己的系统 App（`os.*`）按摘要直接
  打包进二进制，无需发布者签名，也不经过商店门禁，推荐演示 / 联调用。

安装脚本会做三件事（可重复执行）：

1. 把本 bundle 同步到 shell 仓库的 `apps/finance-brief/bundle/`，并把 manifest 的
   id 改为 `os.finance-brief`（系统 App 用 `os.` 前缀；`bundle_blake3` 留空，
   由构建填写）；
2. 在 bundle 根放一份 `icon.svg`（启动器图标按这个位置查找）；
3. 把 `finance-brief` 追加进 `desktop/system-apps.json` 的 `apps`（幂等，
   保留原有系统 App）。

```sh
# 方式一：分两步
sh scripts/install-as-system-app.sh
cd <workspace>/OctoSense
cargo run --release -p octosense

# 方式二：一条命令（安装 + 构建 + 启动）
sh scripts/run-octosense.sh
```

**在 shell 里打开它**：桌面启动后，从底部 dock、左上角 **Apps** 菜单，或
**Ctrl/Cmd+Space** 搜索里选择「财经简报」。它会作为 App Hub 的 Card runner 里的
受控 Splash 程序运行，日志可确认：

```text
wm: launched finance-brief as client 1 (in-process, card)
card: os.finance-brief running under 2 capability(ies), 5 host(s), …
```

（桌面壳支持 `MAKEPAD_REMOTE=<port>` 的本地远程桥，路由与 `card-host` 相同，
可用于无人值守地驱动；`MAKEPAD_HIDE_WINDOWS` 在非 macOS 上不生效。）

**移除**：从 `desktop/system-apps.json` 的 `apps` 里删掉 `finance-brief`，
删除 `apps/finance-brief/`，再重新构建即可。

## 截屏

某些渲染后端（如软件 OpenGL）不会把客户端的帧推回 X11 backing store，
因此 `tools/octo shot` 与 `card-host` 的 `GET /g?raw=1` 可能返回
`{"err":"grab timeout (is this backend rendering?)"}`。Makepad 自己的
指引明说不要用 OS 级截图替代（先后取到两张相同的旧图）。

旁路是 shell 自带的 **`--test-action capture:<path>`** 选项：它在每一帧动
`cx.capture_next_frame_to_file()` —— **GPU 纹理直接读回磁盘**，源码在
`OctoSense/crates/shell/src/lib.rs:3854-3902`。注释明确说
「`the GPU readback does not need one [a display]`」。

### Run it and capture a real screenshot

一次启动 + 启动后抓一帧：

```sh
# 1) 安装为系统 App（幂等，可重复跑）
sh scripts/install-as-system-app.sh

# 2) 启动桌面 shell，并在 launch-finance-brief 后把每 5 秒的帧写到 capture.png
cd <workspace>/OctoSense
nohup env DISPLAY=:0 WAYLAND_DISPLAY=wayland-0 \
  XDG_RUNTIME_DIR=/run/user/1000 \
  MAKEPAD_REMOTE=8142 \
  ./target/release/octosense \
    --test-action capture:/tmp/cap.png \
    --test-action launch-finance-brief \
  > /tmp/shell.log 2>&1 &

# 3) 等首帧（约 5–10 s），检查日志与 PNG
tail -f /tmp/shell.log   # 看 "wm: launched finance-brief as client 1" 与 "card: os.finance-brief running under ..."
ls -la /tmp/cap.png /tmp/cap.part.png   # cap.part.png 是写入中的临时文件
```

### 逐 Tab 抓图

`capture:` 路径每 5 秒被同一份覆盖；要逐 Tab 抓图就依次点击 + 间隔取名。

坐标从桌面桥实时拿（不是窗口布局点，而是**屏幕点**，记号不同）：

```sh
PORT=8142
curl -s 127.0.0.1:$PORT/d > /tmp/d.txt
# tabbar 是第 19 行起一串 Button；它们的 r=[x,y,w,h] 是屏幕坐标，中心 = (x+w/2, y+h/2)
# OctoSense 主题里当前示例位置（仅参考，重启后会变）：
#   要闻 (94,219),  A股 (155,219),  美股 (214,219),  加密 (277,219),  外汇 (340,219),  收藏 (402,219)

click() { curl -s "127.0.0.1:$PORT/click?x=$1&y=$2&wait=1" >/dev/null; }
sleep 9     # 让上一个 click 反映到下一个 capture tick
cp /tmp/cap.png <workspace>/OctoSense-App-Hub/apps/finance-brief/bundle/screenshots/01-news.png

click 155 219
sleep 9
cp /tmp/cap.png <workspace>/OctoSense-App-Hub/apps/finance-brief/bundle/screenshots/02-astocks.png

click 214 219   # ... 美股 / 加密 / 外汇 / 收藏 同样做法
```

装完 6 张后，盖章与门禁：

```sh
octosense/stamp --allow-unsigned <workspace>/OctoSense-App-Hub/apps/finance-brief/bundle
octosense/check --allow-unsigned <workspace>/OctoSense-App-Hub/apps/finance-brief/bundle
# 预期：finance-brief 0.1.0 — PASSED（仅 publisher 占位符警告）
```

### 为什么不用 `tools/octo shot`

| 路径 | 表现 |
| --- | --- |
| `tools/octo shot <port> out.png`（→ `card-host` 的 `/g?raw=1`） | grab arm 成功，永远 timeout |
| ImageMagick `import -window "Card host [remote]"` | 首帧是真实的，之后取到同一张旧图（GL 后端不更新 X11 backing store） |
| `--test-action capture:<path>`（shell 自己） | GPU 纹理直接读回磁盘，每 5 秒覆写一次，**可用** |

## 发布前：`listing.json` 要人工填写的字段

以下字段由发布者本人填写（脚本/agent 不代填）。规则取自 App Hub 门禁代码
（`crates/app-policy/src/listing.rs`）：

| 字段 | 门禁要求 | 怎么填 |
| --- | --- | --- |
| `publisher.name` | 非空 | 你在商店里对外的名字（个人或组织，可公开） |
| `publisher.support` | 非空，URL 或邮箱均可 | 用户求助入口，例如 `https://github.com/<你>/<仓库>/issues` 或 `support@example.com` |
| `publisher.privacy_policy_url` | **必须以 `https://` 开头** | 你的隐私政策页面；需真实可访问（门禁不查内容，评审会看） |
| `platforms` | 至少 1 个，只能是 `android` `ios` `macos` `windows` `linux` `openharmony` `web` | **只写真测过的平台**。本 App 目前只在 Linux 桌面会话上跑过 → `["linux"]` |
| `screenshots` | 1–8 个，`.png`/`.svg`，bundle 内相对路径 | `tools/octo shot` 抓的真实截图；**不能为空**，也不能放占位图 |
| `category` | 闭集之一（含 `finance`） | 本 App 用 `finance` |
| `age_rating` | `all` `12+` `16+` `18+` | 财经信息类用 `all` |
| `icon` | `.svg`/`.png` 的 bundle 内相对路径，不含 `..` | 现为 `assets/icon.svg` |

示例（把占位处换成你自己的信息）：

```json
"publisher": {
  "name": "示例工作室",
  "support": "https://github.com/example/finance-brief/issues",
  "privacy_policy_url": "https://example.com/finance-brief/privacy"
}
```

注意：`screenshots` 里写了文件名、而 bundle 里没有该文件时，门禁会直接拒绝
（`is named by the listing but is not in the bundle`）——当前就是这个状态，
需要在能产出真实截图的环境补齐后再提交。

## 已知限制

- **截图仍依赖 GPU 纹理回读。** `--test-action capture:` 在有 GPU 的环境下
  可以用；无 GPU（如某些容器 / 无显示的 sandbox）时与 `tools/octo shot` 一样
  不可用。本仓库的 6 张真图就是用这条路径出的。
- 未在手机上运行（受控 bundle 不支持侧载）。
- `listing.json` 的发布者字段仍是占位文案，平台声明为 `linux`，需人工确认。

## 实现要点（与本项目规则相关）

- 读取远端 JSON 字段一律走 `get(obj, key, fallback)`：Splash 里读取**不存在
  的属性会直接报错**（不是 nil），所以 `if o.k != nil` 拦不住，必须枚举对象键。
- 每个解析器在 `parse_json` 前检查响应首字符（`{` 或 `[`），避免非 JSON
  响应把 `refresh()` 里的 `busy` 卡住。
- `fixed(v, dec)` 先取整数部分再处理小数，避免 `round(v * 10^dec)` 在大数值上
  丢精度（曾把 BTC 显示成 `83745.0-16`）。
- 示例行以 `sample:` 开头，渲染时统一显示 `示例数据 · 非实时`，不会被误当成
  实时数据。

## 免责声明

数据来自上述公开免密接口，仅用于演示与技术验证，**不构成任何投资建议**。
各数据源的版权与使用条款归其服务商所有。
