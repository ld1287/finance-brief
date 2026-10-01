# Changelog

## 0.3.0 — 2026-10-01

- **行情看盘改为独立 pane**：从主面板的 quotes tab 抽出成顶级 view（与 market-research 并列），含 "A 股 / 美股 / 链上" 3 个 tab + K 线主体（synth_candles 占位）
- 修复进入"行情看盘"时错误渲染为新闻简报的 bug（C-1：5 个 splash 函数加 tid=="quotes" 分支）
- 详情页内嵌 K 线（C-2）
- 增加快捷入口 `pick("kline")`（C-3）
- 数据源加入 NASDAQ 美股接口（`api.nasdaq.com`，仅网络白名单；运行时实现下一迭代）

## 0.2.0 — 2026-09-30

- New top-level **Launcher** view (6-tile grid: News / Quotes / Research / Favs / Settings / About)
- New **Settings** view (theme, per-host switches, push schedule placeholder)
- New **Disclaimer** view (mandatory notice shown on first run)
- **Favorites** view reorganized into typed sections (News / A-shares / US / Crypto / FX)
- Settings persisted to the app's `storage` jail (`settings.json`)
- Theme: dark / light / auto switch (auto placeholder)
- 6 bundle screenshots refreshed for the new tabs

## 0.1.0 — 2026-09-30

- Initial demo: News / A-shares / US / Crypto / FX / Favs tabs
- 4 free data sources wired: Sina news feed, Tencent quotes (intraday snapshot),
  Hyperliquid crypto (last trade + 24 h change), Frankfurter FX rates
- Detail view, favorites, offline sample data
- 6 real screenshots in the bundle