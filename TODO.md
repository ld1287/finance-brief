# TODO — 财经简报 (finance-brief)

Recorded per the project skill: requirements first, then a documented TODO.
Owner: the coding agent. Nothing outside the app directory is changed.

## Decisions (confirmed with the requester)

- A store app developed in the Hub checkout: `OctoSense-App-Hub/apps/finance-brief/`,
  id `finance-brief`, v0.1.0. `catalog.json` / `index/` / `artifacts/` untouched.
- UI in OctoScript (Splash): `bundle/main.splash`.
- Themes: 要闻 / A股 / 美股 / 加密 / 外汇 (+ 收藏). 加密 uses **Hyperliquid**.
- Data from key-free https sources only.
- Acceptance: it runs on this machine, which has a display. **No publish chain
  this round** (requester chose option A). Real screenshots were captured later
  via the shell's `--test-action capture:` GPU readback, so they now live in
  `bundle/screenshots/` and `hub check` passes the screenshot gate.

## Done

- [x] Environment: a Linux desktop session with a display; `octosense`, `hub`,
      `card-host` already built; `.sources/` present; `tools/octo doctor` all green.
- [x] OctoSense desktop shell launched (`cargo run --release -p octosense`);
      remote bridge on :8141 answered `/s`, `/snap`, `/log`.
- [x] Endpoints verified by hand with `curl` before writing the parser.
- [x] App created, `manifest.json` (storage + net, 5 hosts), `listing.json`,
      `assets/icon.svg`, `main.splash` written.
- [x] Runs in `card-host`: admitted, first frame drawn, **no script errors**
      (`grep -nE '\[E\]|splash:[0-9]+:|refused|on_render closure failed'` empty).
- [x] Live data confirmed over the bridge: 要闻 = Sina, 加密 = Hyperliquid,
      外汇 = Frankfurter, A股/美股 = Tencent (see below).
- [x] Interactions driven over the bridge and observed: tab switch, row → detail,
      返回, favourite, 收藏 tab ("已收藏 1 条").
- [x] Restart persistence: a seeded `favs.json` reloads (verified).
- [x] UI seen: an X11 window capture showed the real 412x892 app (title, tab
      bar, live news cards, disclaimer).
- [x] Independent code review by a second agent; its findings were addressed.

## Review findings addressed

- [x] **BLOCKER** `fixed()` emitted `"83745.0-16"` for BTC: `round(a * m)` with
      m = 10^4 lost the low bits of a 5-digit price. Rewritten to split the
      fraction from `floor(a)` before scaling.
- [x] `load_crypto` indexed `d[0]`/`d[1]` without checking the payload shape;
      every loader now checks the body starts with `{`/`[` before parsing.
- [x] A cached start showed the "tap refresh" prompt although real data was on
      screen; a `cached` flag now says "显示上次缓存，点「刷新」更新".
- [x] `refresh()` keeps an open detail card (stale object); it now clears it.
- [x] **A股/美股 were not live.** The first implementation used
      `push2.eastmoney.com` / `push2delay.eastmoney.com`, which flap on this
      network; when both failed the tabs showed the *sample*, and the sample
      used real index/symbol names with invented numbers (上证指数 3080.15) —
      indistinguishable from live data and simply wrong. Fixed by moving quotes
      to Tencent `qt.gtimg.cn` (verified live in the UI: 上证指数 3830.45
      +0.18%, 深证成指 12901.95, 沪深300 4345.21, 贵州茅台 1235.58, 五粮液
      68.77; 苹果 338.40, 微软 509.22, 英伟达 228.86, 特斯拉 357.45, 亚马逊
      246.15) and by rendering every `sample:` row as `示例数据 · 非实时`.
- [x] Sina `hq.sinajs.cn` was tried for quotes and had to be dropped: this
      client fails it with a TLS `unexpected eof while reading` although
      `curl` succeeds. Recorded so it is not re-attempted blindly.

## Not verified / gaps (stated, not papered over)

- [x] **Real screenshots (1..6) are in `bundle/screenshots/` and the gate
      accepts them.** `tools/octo shot` still cannot work on this machine
      (`/g?raw=1` arms the grab but never delivers pixels under software-rendered
      OpenGL), so the bundle was captured with the shell's own
      `--test-action capture:<path>` (a direct GPU readback that needs no
      backing store). See the **"截屏"** section of `README.md` for the exact
      recipe; see **"Run it and capture a real screenshot"** for the
      one-command flow.
- [x] **Running in the desktop shell: done, via the system-app path.**
      `hub publish` re-runs `hub check`, which refuses a bundle with no
      screenshot (`at least one PNG in the bundle, named in the listing, is
      required`), so no signed local catalog exists and the store rehearsal
      (PUBLISHING §4) stays open. `build/install-as-system-app.sh` instead
      syncs the bundle to `OctoSense/apps/finance-brief/bundle/` with id
      `os.finance-brief`, adds an `icon.svg`, and appends `finance-brief` to
      `OctoSense/desktop/system-apps.json`, so the shell packs it by digest.
      Verified: the rebuilt shell logged
      `wm: launched finance-brief as client 1 (in-process, card)` and
      `card: os.finance-brief running under 2 capability(ies), 5 host(s)` and
      rendered the live 要闻 list. `build/run-octosense.sh` does install + run
      in one command.
- [ ] `listing.json` publisher fields are still placeholders, and `platforms`
      says `linux` — the human owns both.
- [ ] Not run on a phone (side-loading a bundle is unsupported).

## Follow-ups (requester's call)

- [ ] Replace `assets/icon.svg` / publisher details before any submission.
