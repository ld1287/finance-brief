# bundle/listing.json publisher 字段填写指南

本文档面向 release 前的最后一步：手动填写 `bundle/listing.json` 中的发布者相关字段。
门禁 (`tools/octo check listing.json`) 会拒绝保留占位符、占位 URL 与空截图数组的内容；
本指南逐字段说明人工填写要求与反例。

适用版本: schema = 1
关联文件: `bundle/listing.json` `bundle/manifest.json` `bundle/*.card`

---

## 1. 字段总览

| 字段 | 类型 | 是否必填 | 当前值 | 验收要求 |
| --- | --- | --- | --- | --- |
| `publisher.name` | string | 是 | `"Replace with your publisher name"` | 必须替换为真实发布者名,不能含 `Replace` / `示例` / `example` |
| `publisher.support` | string (URL / mailto) | 是 | `"https://example.com/support"` | 必须替换为可访问的 issue tracker / mailto |
| `publisher.privacy_policy_url` | string (URL) | 是 | `"https://example.com/privacy"` | 必须替换为可访问的隐私政策页 |
| `platforms` | string[] | 是 | `["linux"]` | 至少 1 项,按实际运行平台填 |
| `screenshots` | string[] (URL / path) | 是 | `[]` | 至少 1 张真抓帧 PNG/JPG,不能写占位文件名 |

`schema` / `subtitle` / `description` / `category` / `keywords` / `icon` / `release_notes` / `age_rating` / `license` 不在本文档范围内,改动前请先确认是否已通过审核。

---

## 2. `publisher.name`

**用途**: 在应用市场/卡片中心列表页显示的发布者署名,也是用户首次反馈时定位来源的依据。

**填写规则**:
- 团队发布者用团队全称或约定简称 (例: `"ACME Finance"` / `"Open Octo Collective"`)
- 个人发布者用真实姓名或常用笔名 (例: `"张三"` / `"octodev"`)
- 通用但避免暴露个人身份 (身份证号、住址、内部代号不要写进 name)
- 同名项目用统一署名,后续 changelog 不要再改

**✅ 推荐写法**:
```json
"publisher": {
  "name": "ACME Finance"
}
```

**❌ 反例 (会被门禁拒绝)**:
```json
"name": "Replace with your publisher name"
"name": "示例工作室"
"name": "TODO"
"name": "your name here"
```

---

## 3. `publisher.support`

**用途**: 用户遇到问题或想反馈时的官方联系入口;门禁校验 URL/mailto 形态,而不是字面量。

**填写规则**:
- 优先使用 issue tracker URL,便于公开追踪 (例: `https://github.com/<org>/<repo>/issues`)
- 备选 `mailto:` 链接 (例: `mailto:support@example.org`)
- 不放 `example.com` 之类占位域
- 不放个人社交账号私信链接 (容易失效)

**✅ 推荐写法**:
```json
"publisher": {
  "support": "https://github.com/<org>/finance-brief/issues"
}
```

**❌ 反例 (会被门禁拒绝)**:
```json
"support": "https://example.com/support"
"support": "mailto:your-email@example.com"
"support": "TBD"
```

> 通用占位 (release 时人工填):release 前替换为该项目的真实 issue tracker 或联系邮箱。

---

## 4. `publisher.privacy_policy_url`

**用途**: 市场页和卡片详情页"数据如何被收集/使用"的入口 URL,合规必需。

**填写规则**:
- 指向仓库内的 `docs/PRIVACY.md` (例: `https://github.com/<org>/<repo>/blob/main/docs/PRIVACY.md`)
- 或单独的 privacy 子域名页面 (例: `https://<org>.example.org/privacy`)
- 必须可公开访问,不能是 `localhost` / 内网 IP / 需要登录的页面
- 与 README 隐私声明保持一致口径

**✅ 推荐写法**:
```json
"publisher": {
  "privacy_policy_url": "https://github.com/<org>/finance-brief/blob/main/docs/PRIVACY.md"
}
```

**❌ 反例 (会被门禁拒绝)**:
```json
"privacy_policy_url": "https://example.com/privacy"
"privacy_policy_url": ""
"privacy_policy_url": "see README"
```

---

## 5. `platforms`

**用途**: 标记 bundle 在哪些桌面/移动平台被验证可运行;卡片中心按平台过滤可见性。

**当前值**: `["linux"]`

**填写规则**:
- 数组元素必须取自白名单: `"linux"` / `"macos"` / `"windows"` / `"android"`
- 至少 1 项;多平台用 JSON 数组顺序书写,推荐按字母升序
- 只填已实测可运行的平台,不要把"计划支持"写进来
- schema = 1 当前不接受 `ios` / `web` / `wasm` 等非桌面平台 token

**✅ 推荐写法** (Linux 单平台):
```json
"platforms": ["linux"]
```

**✅ 推荐写法** (多平台):
```json
"platforms": ["linux", "macos", "windows"]
```

**❌ 反例**:
```json
"platforms": []
"platforms": ["Linux"]
"platforms": ["linux", "ios"]
"platforms": "linux"
```

---

## 6. `screenshots`

**用途**: 应用市场/卡片中心列表页和详情页的视觉预览;空数组或占位文件都会被门禁拒绝。

**当前值**: `[]` (空数组,验收必填 ≥ 1 张)

**填写规则**:
- 数组元素是相对 `bundle/` 的 PNG/JPG 路径,或公网可访问的图片 URL
- 至少 1 张,推荐 3–5 张覆盖主要屏 (启动屏 + 列表屏 + 详情屏 + 设置屏)
- 图片必须是真实抓帧 (用 `tools/octo shot` 在 card-host 启动后截图),不是设计稿或文字稿
- 抓帧后放到 `bundle/screenshots/` 目录,在 listing.json 中用相对路径引用
- 推荐尺寸 16:9 或 4:3,宽度 ≥ 1280 px,大小 ≤ 500 KB/张

**12 屏 `.card` 候选截图**:

| # | 源 `.card` | 推荐截图名 | 推荐场景 |
| --- | --- | --- | --- |
| 1 | `bundle/launcher.card` | `01-launcher.png` | 启动屏:5 标签导航 + 当前行情概览 |
| 2 | `bundle/news_list.card` | `02-news-list.png` | 要闻列表:实时刷新 + 收藏标识 |
| 3 | `bundle/news_detail.card` | `03-news-detail.png` | 要闻详情:正文 + 关联行情 |
| 4 | `bundle/quote_list.card` | `04-quote-list.png` | A股/美股/加密/外汇标签切换 |
| 5 | `bundle/kline.card` | `05-kline.png` | K 线:多周期切换 + 缩放 |
| 6 | `bundle/favorites.card` | `06-favorites.png` | 收藏夹:跨标签合并 + 重启后保留 |
| 7 | `bundle/research_list.card` | `07-research-list.png` | 研究/财报列表 |
| 8 | `bundle/research_detail.card` | `08-research-detail.png` | 研究/财报详情 |
| 9 | `bundle/event_stream.card` | `09-event-stream.png` | 实时事件流 |
| 10 | `bundle/datasource_status.card` | `10-datasource-status.png` | 数据源健康状态 |
| 11 | `bundle/settings.card` | `11-settings.png` | 设置:刷新频率/主题/缓存 |
| 12 | `bundle/disclaimer.card` | `12-disclaimer.png` | 免责声明 |

**抓帧命令**:
```sh
tools/octo shot --card launcher      --out bundle/screenshots/01-launcher.png
tools/octo shot --card quote_list    --out bundle/screenshots/04-quote-list.png
tools/octo shot --card news_detail   --out bundle/screenshots/03-news-detail.png
tools/octo shot --card settings      --out bundle/screenshots/11-settings.png
tools/octo shot --card favorites     --out bundle/screenshots/06-favorites.png
```

**✅ 推荐写法**:
```json
"screenshots": [
  "screenshots/01-launcher.png",
  "screenshots/04-quote-list.png",
  "screenshots/03-news-detail.png",
  "screenshots/11-settings.png",
  "screenshots/06-favorites.png"
]
```

**❌ 反例 (会被门禁拒绝)**:
```json
"screenshots": []
"screenshots": ["screenshot.png"]
"screenshots": ["TODO_placeholder.png"]
"screenshots": ["https://example.com/screen.png"]
```

> ⚠️ 占位文件陷阱:门禁会逐一打开每个 path,只要文件不存在或返回 404 即拒绝;
> 不要写 `screenshot1.png` / `placeholder.jpg` 这类无意义文件名。

---

## 7. 验证步骤

release 前请按顺序执行:

### 7.1 静态检查

```sh
tools/octo check listing.json
```

期望输出:
```
[ok] schema = 1
[ok] publisher.name 非占位
[ok] publisher.support URL/mailto 形态合法
[ok] publisher.privacy_policy_url URL 形态合法
[ok] platforms 至少 1 项
[ok] screenshots 至少 1 张,文件存在
[ok] manifest.json capability 白名单未变动
```

任意一项 `[fail]` 即中止 release,按报错回到对应章节重填。

### 7.2 JSON 形态校验

```sh
jq -e '.publisher.name | test("Replace|TODO|示例") | not' bundle/listing.json
jq -e '.publisher.support   | test("example\\.com|TBD")   | not' bundle/listing.json
jq -e '.publisher.privacy_policy_url | test("example\\.com") | not' bundle/listing.json
jq -e '.platforms  | length >= 1' bundle/listing.json
jq -e '.screenshots | length >= 1' bundle/listing.json
```

任意一条退出码非 0 即失败。

### 7.3 截图可达性

```sh
for s in $(jq -r '.screenshots[]' bundle/listing.json); do
  test -f "bundle/$s" || { echo "missing: bundle/$s"; exit 1; }
done
```

公网 URL 截图 (如有):
```sh
for s in $(jq -r '.screenshots[] | select(startswith("http"))' bundle/listing.json); do
  curl -fsSI "$s" >/dev/null || { echo "unreachable: $s"; exit 1; }
done
```

### 7.4 一致性回看

```sh
diff <(jq -S . bundle/listing.json) <(jq -S . bundle/listing.json.bak 2>/dev/null)
```

确认除 publisher / platforms / screenshots 三组外,没有意外改动。

---

## 8. 当前 `bundle/listing.json` 实际内容 (脱敏后示例)

下面是 release 前应当被替换完成的目标形态 (字段值用占位变量,实际填法见上文对应章节):

```json
{
  "schema": 1,
  "subtitle": "一屏看盘：要闻、A股、美股、加密、外汇",
  "description": "财经简报把公开免密接口的财经要闻与行情聚合到一个列表里：分「要闻 / A股 / 美股 / 加密 / 外汇」五个标签，点任意一条进入详情，可收藏并在重启后保留。无网络时显示内置示例数据。所有数据仅用于演示，不构成投资建议。",
  "category": "finance",
  "keywords": ["finance", "news", "stocks", "crypto", "forex", "行情", "财经"],
  "screenshots": [
    "screenshots/01-launcher.png",
    "screenshots/04-quote-list.png",
    "screenshots/03-news-detail.png",
    "screenshots/11-settings.png"
  ],
  "icon": "assets/icon.svg",
  "platforms": ["linux"],
  "publisher": {
    "name": "<PUBLISHER_NAME>",
    "support": "<SUPPORT_URL_OR_MAILTO>",
    "privacy_policy_url": "<PRIVACY_POLICY_URL>"
  },
  "release_notes": "首个演示版本：五个行情/要闻标签、详情、收藏与离线示例数据。",
  "age_rating": "all",
  "license": "Apache-2.0"
}
```

> 说明: `subtitle` / `description` / `category` / `keywords` / `icon` / `release_notes` / `age_rating` / `license` 不在本文档讨论范围,保持原值即可;
> `<PUBLISHER_NAME>` `<SUPPORT_URL_OR_MAILTO>` `<PRIVACY_POLICY_URL>` 三个占位符需在 release 前替换为真实值,详见 §2–§4。
