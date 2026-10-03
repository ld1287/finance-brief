# Octoscript L0 sys.* catalog field reference

> 提取日期: 2026-10-03
> 来源: /home/lumina/octoOs/octoscript/crates/octoscript-ui-l0/src/lib.rs (pub mod catalog, ANSWERS const, 行 3750-3911)
> 用途: finance-brief 12 屏 sub-agent 写 .card 时引用；不直接 grep lib.rs

## 完整 catalog (32 条目)

| sys.X | fields |
|-------|--------|
| sys.geocode | lat, lon, name, country, admin1, timezone, population |
| sys.weather | temp, feels, hi, lo, cond, humidity, wind, pressure, uv, visibility, precip, dayname, days |
| sys.daylight | rise, set, now |
| sys.airquality | aqi, pm25, pm10, ozone |
| sys.moonphase | phase, illumination, name |
| sys.wiki | title, extract, description |
| sys.photo | (空) |
| sys.locale | lang, temp_unit |
| sys.convert | amount, value |
| sys.gps | lat, lon, accuracy, ok |
| sys.search | id, name, label, query, lat, lon, distance |
| sys.route | duration, distance, steps |
| sys.step | instruction, remaining, progress, eta |
| sys.places | id, name, distance, lat, lon, category |
| sys.news | id, title, author, points, comments, url |
| sys.news_digest | id, title, summary, publisher, url, published_at |
| sys.dataset | title, subtitle, summary, coverage, status, as_of, metric1_label, metric1_value, metric2_label, metric2_value, pick1_title, pick1_body, pick1_source, url1, pick2_title, pick2_body, pick2_source, url2, pick3_title, pick3_body, pick3_source, url3, evidence_title, evidence_body |
| sys.news_status | status, message, count |
| sys.quakes | id, mag, place, depth, ago, lat, lon |
| sys.news_item | id, title, author, points, comments, url |
| sys.movers | ticker, name, last, change, pct, open, high, low, prev, volume, mktcap, pe, currency, exchange |
| sys.quote | ticker, name, last, change, pct, open, high, low, prev, volume, mktcap, pe, currency, exchange |
| sys.series | min, max |
| sys.watchlist | ticker, name, last, change, pct, open, high, low, prev, volume, mktcap, pe, currency, exchange, has |
| sys.indicator | name, latest, first, change, min, max, year, title |
| sys.video | id, title, channel, length, views, age, thumb, embed |
| sys.prefs | units, range, home, work, mode |
| sys.reading | id, title, author, points, comments, url |
| sys.topics | name, top_title, top_points, top_id |
| sys.link | url |
| sys.symbol_search | ticker, name, exchange, kind |
| sys.cities | name, lat, lon, temp, feels, feels_delta, hi, lo, cond, humidity, wind |

## 屏适配提示

按 finance-brief 12 屏 ↔ catalog 适配（屏文件 sub-agent 必读）：

| # | 屏 | 推荐 sys.X | fields |
|---|---|---|---|
| 1 | launcher | 无（静态 copy） | — |
| 2 | news_list | sys.news | id, title, author, points, comments, url |
| 3 | news_detail | sys.news_item | id, title, author, points, comments, url |
| 4 | research_list | sys.topics 或 sys.reading | topics: name, top_title, top_points, top_id / reading: id, title, author, points, comments, url |
| 5 | research_detail | sys.dataset | title, subtitle, summary, coverage, status, as_of, metric1_label, metric1_value, metric2_label, metric2_value, pick1_title, pick1_body, pick1_source, url1, pick2_title, pick2_body, pick2_source, url2, pick3_title, pick3_body, pick3_source, url3, evidence_title, evidence_body |
| 6 | kline | sys.series | min, max |
| 7 | quote_list | sys.quote（单 ticker）+ sys.movers（movers 列表）| ticker, name, last, change, pct, open, high, low, prev, volume, mktcap, pe, currency, exchange |
| 8 | favorites | sys.watchlist | ticker, name, last, change, pct, open, high, low, prev, volume, mktcap, pe, currency, exchange, has |
| 9 | settings | sys.prefs | units, range, home, work, mode |
| 10 | disclaimer | 无（静态） | — |
| 11 | event_stream | sys.news_digest + sys.step | news_digest: id, title, summary, publisher, url, published_at / step: instruction, remaining, progress, eta |
| 12 | datasource_status | sys.news_status | status, message, count |

## 注意

- **不存在 sys.quotes（复数）**，只有 sys.quote（单 ticker） → 屏 #7 quote_list 不能 tab 切 4 主题，必须改单 ticker + movers 双卡设计
- sys.series 不是 candlestick，是单数值时间序列 → 屏 #6 kline 退化为 line chart 占位
- sys.dataset 不是 research detail，是 3 picks + evidence 的数据卡 → 屏 #5 research_detail 改用 dataset 形态
- sys.news_status 是 {status, message, count} → 屏 #12 datasource_status 改用 5 个独立 status 卡
- sys.news_digest 是 {id, title, summary, publisher, url, published_at} → 屏 #11 event_stream 改用 digest 流