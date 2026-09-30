# BRIEF — 财经简报 (finance-brief)

The app itself: `OctoSense-App-Hub/apps/finance-brief/bundle/`.

## Purpose

A read-only finance briefing: one scrollable list per theme, tap a row to read
its detail, keep items in a local favourites list. Shows a built-in sample —
clearly labelled 示例数据 · 非实时 — until the first successful fetch, then
refreshes from key-free public endpoints.

## Screens

| Screen | What it shows |
| --- | --- |
| Header | Title, subtitle, a **刷新** button |
| Tab bar (horizontal scroll) | 要闻 / A股 / 美股 / 加密 / 外汇 / 收藏 |
| Status line | 刷新中… / 来源不可用（显示缓存）/ 显示上次缓存 / 更新于 HH:MM |
| List | One card per row: title; quotes also show code, price and % change |
| Detail | Title, source + time, price + % (quotes), summary (news), URL, 返回/收藏 |
| Empty state | "暂无内容 — 点右上角「刷新」抓取" |

Rows whose id starts with `sample:` show `示例数据 · 非实时` where the source
line goes, so a fallback can never be mistaken for a live quote.

## Actions

- Tap a tab → switch theme (`pick`).
- Tap a row → open detail (`open_detail`); 返回列表 → `close_detail`.
- 收藏 / 取消 (`toggle_fav`) writes `favs.json`, kept across restarts.
- 刷新 → fetch every theme (`refresh`); a 600 s interval refreshes in the
  background.

## Data sources (all key-free, https)

| Tab | Host | Request | Fields |
| --- | --- | --- | --- |
| 要闻 | `feed.mix.sina.com.cn` | `GET /api/roll/get?pageid=153&lid=2516&num=20&page=1` | `result.data[].title/media_name/intro/url/ctime` |
| A股 | `qt.gtimg.cn` | `GET /q=sh000001,sz399001,sh000300,sh600519,sz000858` | `~`-separated: 3 current, 32 change % |
| 美股 | `qt.gtimg.cn` | `GET /q=usAAPL,usMSFT,usNVDA,usTSLA,usAMZN` | same layout |
| 加密 | `api.hyperliquid.xyz` | `POST /info` `{"type":"metaAndAssetCtxs"}` | `[0].universe[i].name`, `[1][i].markPx/prevDayPx` |
| 外汇 | `api.frankfurter.dev` | `GET /v1/latest?base=USD&symbols=CNY,EUR,JPY,GBP,HKD` | `rates{…}` |

Notes, all measured on this machine (2026-09-29):

- The quote feeds (Sina `hq.*`, Tencent `qt.gtimg.cn`) return **GBK**; names
  decode to replacement characters, so the app keeps its own display names and
  reads only the ASCII numbers.
- `hq.sinajs.cn` was tried first and **fails in this client with a TLS
  `unexpected eof`** (`SSL_read … unexpected eof while reading`) even though
  `curl` succeeds; Tencent's host does not have that problem.
- `push2.eastmoney.com` and `push2delay.eastmoney.com` were also tried: they
  are unreachable/flapping from this network (they answered at first, then
  returned connection failures). They were removed in favour of `qt.gtimg.cn`.
- `api.coingecko.com`, `query1.finance.yahoo.com`, `stooq.com`,
  `api.binance.com` and `api.coinpaprika.com` did not answer from this network.
- A 通达信/rustdx binary TCP feed is **not usable** by a contained app: the
  isolate reaches only declared `https` hosts, and raw sockets are against the
  store rules (SCRIPT-API "Network").

## Stored data (`storage`, app jail)

- `cache_<tab>.json` — last successful fetch per tab (news/a/us/crypto/fx).
- `favs.json` — favourites, reloaded on boot.
- The offline sample is built in the script (`sample_for`), not fetched: the
  `{{assets}}` origin is plain `http` on the loopback and the net policy only
  allows `https`, so a bundled JSON cannot be fetched by the app.

## Capabilities

`storage` (cache + favourites) and `net` (the four hosts above). Nothing more;
no secrets, no text-entry widget.

## States exercised

Empty, source-unavailable (keeps cache + labelled sample), cached start,
restart (favourites reload), live success per source.

## Screenshots in the bundle

`bundle/screenshots/01-news.png .. 06-favs.png` are real captures of the app
running on the OctoSense desktop shell, one per tab. They were produced by the
shell's own `--test-action capture:<path>` (a GPU texture readback that does not
need a screen), because `tools/octo shot` cannot grab frames on this
software-rendering backend. See the **"截屏"** section of `README.md` for the
exact recipe and per-tab capture flow.

## Out of scope this round (agreed with the requester)

Publishing (signing / submission) only. `hub check` already passes the
screenshot gate with these six PNGs in place; the publisher-name/support/
privacy fields and a real `hub scan` review remain for the human.
