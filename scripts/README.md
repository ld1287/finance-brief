# `finance-brief/scripts` — 脚本使用手册

> 本目录 8 个 Python 脚本覆盖 finance-brief 在 OctoSense 桌面端的全生命周期:注册、启动、远程桥(headless)验证、shell 污染诊断与清理。每个脚本 `python foo.py --help` 拿完整参数。

> 这是根目录 `README.md §4` 的详细补充版。`§4` 适合 30 秒扫一眼,本文适合实际使用时的参考手册。

## 目录

| § | 脚本 | 一句话定位 |
|---|------|-----------|
| 1 | [`clean-shell-pollution.py`](#1-clean-shell-pollutionpy) | 清理 finance-brief 在 OctoSense shell 留下的 3 处污染 |
| 2 | [`diagnose-shell-state.py`](#2-diagnose-shell-statepy) | 只读诊断:扫 5 处路径,输出 JSON,不写任何文件 |
| 3 | [`drive-test.py`](#3-drive-testpy) | remote bridge 自动点 launcher 5 个 tab,验证 label 序列 |
| 4 | [`install-as-makepad-app.py`](#4-install-as-makepad-apppy) | **推荐路径**:build + 写 `~/.octosense/apps.json` |
| 5 | [`install-as-system-app.py`](#5-install-as-system-apppy) | legacy Page-format 脚本(可选 `--uninstall` 清理) |
| 6 | [`run-octosense.py`](#6-run-octosensepy) | 调 `install-as-makepad-app.py` + `cargo run -p octosense` |
| 7 | [`verify.py`](#7-verifypy) | remote bridge 验证 5 个行情 tab + 刷新按钮的 label |
| 8 | [`register-with-shell.py`](#8-register-with-shellpy) | 把 finance-brief 注册到 OctoSense shell 用户级 catalog(不 rebuild) |

另见:根目录 [`README.md` §4](../README.md#4-脚本-scriptspy)(30 秒速览)。

## 共同前置

- **Python**:3.8+ 即可跑通(用 stdlib only);`dict | None` 注解写在 docstring / 类型提示里,所以严格说需要 **3.10+** 才不会被 linter 标红。
- **工作目录**:从仓库根 `finance-brief/` 跑 `python scripts/<name>.py`。脚本内部用 `Path(__file__).resolve().parent` 推工作目录,所以**不强制要求 cd 到 scripts/**。
- **环境变量 `OCTOSENSE_HOME`**:可选。设了之后,用户级 catalog 与缓存路径切到 `$OCTOSENSE_HOME/.octosense/...` 而不是 `~/`。Windows / macOS / Linux 都生效。诊断类、清理类脚本读取此变量。
- **跨平台**:所有脚本仅依赖 stdlib(`argparse` / `subprocess` / `urllib.request` / `pathlib` / `shutil` / `json`),Windows 不需要 Git Bash / WSL,直接 `python foo.py`。
- **OctoSense checkout**:`install-as-system-app.py` 强制要求同级存在 `OctoSense/` 仓库(写 `system-apps.json` 与 `apps/finance-brief/`)。`install-as-makepad-app.py` 不需要。

## 退出码约定

| 退出码 | 语义 | 出现于 |
|--------|------|--------|
| `0` | 成功 | 全部 |
| `2` | argparse 参数错 | 全部带 argparse 的脚本 |
| `7` | remote bridge 不可达(`URLError`) | `drive-test.py` / `verify.py` |
| `10` | pollution detected | `diagnose-shell-state.py` / `clean-shell-pollution.py --diagnose-only` |
| `1` | 内部 error / 部分清理失败 | `clean-shell-pollution.py`(清理后仍有污染)/ `drive-test.py`(其它异常) |
| `3` | makepad rev 不对齐 | `install-as-makepad-app.py` |
| `4` | 缺 binary / 不可执行 | `install-as-makepad-app.py` |

其它退出码:`install-as-makepad-app.py` 会把 cargo 的退出码原样透传;`run-octosense.py` 把子进程失败透传。

## `.sh` vs `.py`

`scripts/` 同时挂 5 个 `.sh` legacy 文件:

| `.sh`(legacy) | `.py`(推荐) |
|--------------|-------------|
| `install-as-makepad-app.sh` | `install-as-makepad-app.py` |
| `install-as-system-app.sh` | `install-as-system-app.py` |
| `run-octosense.sh` | `run-octosense.py` |
| `drive-test.sh` | `drive-test.py` |
| `verify.sh` | `verify.py` |

`.sh` 是 2026-10-05 重写之前的版本,**没有删除**(怕破坏老 caller),但根目录 `README.md §4` 已声明"旧 `.sh` 文件已删除"。新代码请用 `.py`:

- `.py` 用 stdlib only,Windows 原生支持,不用 Git Bash / WSL。
- `.sh` 在 Windows 需 Git Bash / WSL 才能跑,且依赖 bash 4+ 特性。
- `.py` 加了 `clean-shell-pollution.py` / `diagnose-shell-state.py` 两个新脚本,`.sh` 版本没有对应物。

---

## 1. `clean-shell-pollution.py`

**用途:** 诊断并清理 finance-brief 在 OctoSense shell 留下的 3 处污染(legacy Page-format 脚本 `install-as-system-app.py` 在没加 `--uninstall` 的情况下误跑后会产生)。

**前置依赖**:

- Python 3.10+(用了 `dict | None` 注解)
- 可选:环境变量 `OCTOSENSE_HOME`(决定用户级路径基址)

**用法**:

```sh
python scripts/clean-shell-pollution.py [选项]
```

**参数**(来自 `--help`):

- `--diagnose-only`(flag):打印诊断 JSON 后直接退出,**不写任何文件**;有污染时 exit `10`,干净时 exit `0`。
- `--dry-run`(flag):打印会做什么(写路径 + status = "would-clean"),不实际执行。
- `--user-cache-only`(flag,与 `--octosense-only` 互斥):只清 `~/.octosense/apps/.system/os.finance-brief/`。
- `--octosense-only`(flag,与 `--user-cache-only` 互斥):只清 `OctoSense/desktop/system-apps.json` 与 `OctoSense/apps/finance-brief/`。

**3 处污染源**:

1. `OctoSense/desktop/system-apps.json` 里 `apps` 数组含 `finance-brief` 字符串项)。
2. `OctoSense/apps/finance-brief/` 整个目录(legacy 安装留下的 partial bundle 拷贝)。
3. `~/.octosense/apps/.system/os.finance-brief/<hash>/` shell 启动时打成的 cached pack。

**退出码**:

- `0`:清理完成且最终诊断干净。
- `1`:清理完了,但最终诊断仍报 pollution(部分路径被锁 / 只读 / 文件被占用)。
- `2`:argparse 参数错。
- `10`:带 `--diagnose-only` 且发现污染(此时没改任何文件)。

**示例**:

```sh
# 只看不改
python scripts/clean-shell-pollution.py --diagnose-only
echo $?     # 10 = 有污染, 0 = 干净

# 看会做什么但不写
python scripts/clean-shell-pollution.py --dry-run

# 全清
python scripts/clean-shell-pollution.py

# 只清 shell 缓存,不动 OctoSense 仓库里的 system-apps.json
python scripts/clean-shell-pollution.py --user-cache-only

# 只清 OctoSense 仓库里的污染,不动 shell 缓存(罕见)
python scripts/clean-shell-pollution.py --octosense-only
```

**典型错误**:

- `JSONDecodeError`(捕获,转 warning):`system-apps.json` 损坏。脚本会把它视作 None,直接走 catalog "missing (no-op)" 分支。
- `PermissionError` / `OSError`(透传到 `1` 退出码):`OctoSense/apps/finance-brief/` 被锁或只读,关掉引用它的进程再跑。
- `FileNotFoundError`(不会出现):此脚本不依赖 finance-brief 已 build,不需要 `apps/desktop/target/release/finance-brief(.exe)`。

**相关脚本**:

- 配 [`diagnose-shell-state.py`](#2-diagnose-shell-statepy):诊断后看详细 JSON,再决定用 `--user-cache-only` 还是 `--octosense-only`。
- 替代品 [`install-as-system-app.py --uninstall`](#5-install-as-system-apppy):也能清 3 处,但语义更"取消一次安装"而不是"清理污染";如果 shell 缓存还在,优先用这个脚本。

---

## 2. `diagnose-shell-state.py`

**用途:** 只读诊断。扫 5 处与 finance-brief 相关的路径,输出 JSON,不写任何文件,不改任何文件。

**前置依赖**:

- Python 3.10+(用了 `dict | None` 注解)
- 可选:环境变量 `OCTOSENSE_HOME`(决定用户级路径基址)

**用法**:

```sh
python scripts/diagnose-shell-state.py
```

**参数**:**无**。此脚本不调用 argparse,`--help` 也会执行扫描(返回 exit `0` 或 `10`)。要看"是否干净"只看返回码:`0` = 干净,`10` = 有污染。

**5 处扫描路径**:

| Key | 路径 | 检查内容 |
|-----|------|---------|
| `octosense_system_apps` | `OctoSense/desktop/system-apps.json` | JSON 数组是否含 `id="finance-brief"` |
| `octosense_apps_json` | `OctoSense/desktop/config/apps.json` | JSON 数组是否含 `id="finance-brief"` |
| `octosense_bundle` | `OctoSense/apps/finance-brief/` | 目录是否存在 |
| `user_cache` | `~/.octosense/apps/.system/os.finance-brief/` | 目录是否存在,若有列 hash 子目录 |
| `user_catalog` | `~/.octosense/apps.json` | JSON 数组是否含 `id="finance_brief"`(下划线) |

**退出码**:

- `0`:任何污染源都不存在(任何 `octosense_system_apps.found` / `octosense_bundle.exists` / `user_cache.exists` 全 false)。
- `10`:发现污染。
- `1`:扫的过程中抛了未捕获异常(JSON 读不出来、IO 错等),stderr 输出 `{"error": "..."}`。

**示例**:

```sh
# 装完之后验证一下
python scripts/install-as-makepad-app.py && python scripts/diagnose-shell-state.py
echo $?     # 0 = 干净, 10 = 有污染

# 退出码配合 shell 自动化
if python scripts/diagnose-shell-state.py > /dev/null; then
    echo "shell state clean"
else
    case $? in
        10) python scripts/clean-shell-pollution.py ;;
        *)  echo "diagnostic error" ;;
    esac
fi
```

**典型错误**:

- 路径不存在 → JSON 字段 `exists: false`,不算污染。
- JSON 损坏 → 字段 `exists: true, error: "<details>"`,不算 pollution,也不影响退出码判断(只看 `octosense_system_apps.found` / `octosense_bundle.exists` / `user_cache.exists`)。
- 注意 `octosense_system_apps` 期望顶层是 JSON **数组**;如果顶层是 dict,会写 `schema: "dict"` 并算 `found: false`。

**相关脚本**:

- 是 [`clean-shell-pollution.py`](#1-clean-shell-pollutionpy) 的"安全版前置":先 `diagnose-shell-state.py` 看有什么、再 `clean-shell-pollution.py` 清。
- 比 `install-as-system-app.py --uninstall` 扫得更广(多了 `octosense_apps_json` 与 `user_catalog`)。

---

## 3. `drive-test.py`

**用途:** 通过 card-host remote bridge 自动点击 launcher 5 个 tab,每步后打印当前屏幕可见文本序列,验证 widget tree 与 on_click 路由。

**前置依赖**:

- finance-brief 已 build:`cd apps/desktop && cargo build --release`(或 `cargo run --release` 一次也行)。
- finance-brief 用 `--remote=0` 启动(绑 ephemeral port,从 log 提端口)。
- remote bridge 在该端口 listen `/snap`、`/m?k=click&...`。

**用法**:

```sh
python scripts/drive-test.py --port <port>
python scripts/drive-test.py <port>           # 位置参数(等价)
```

**参数**(来自 `--help`):

- `--port PORT`(int,与位置参数 `port_pos` 互斥,二选一必填):remote bridge 端口。
- `port_pos`(位置,int,可选):同上,--flag 优先。

**5 步操作流程**:

1. `snap`:打印初始屏幕 label。
2. `click_text("加密")` → snap。
3. `click_text("BTC")`(第一行)→ snap(进入 detail)。
4. `click_text("收藏 / 取消")` → snap(收藏)。
5. `click_text("返回列表")`(无失败保护)→ `click_text("收藏")` → snap。
6. `click_text("要闻")` → snap(live news)。

每步如果 `rect(name)` 找不到对应 widget,打印 `[no widget] <name>` 并跳到下一步,不中断。

**退出码**:

- `0`:全部跑完。
- `2`:argparse 参数错(没传 port)。
- `7`:`urllib.error.URLError`,bridge 不可达(端口错 / bridge 没起 / 进程崩了)。
- `1`:其它异常(写 stderr)。

**示例**:

```sh
# 后台起 finance-brief,绑 remote bridge
nohup ./apps/desktop/target/release/finance-brief --remote=0 > /tmp/fb.log 2>&1 &
disown
sleep 5
PORT=$(grep -oE 'listening on 127.0.0.1:[0-9]+' /tmp/fb.log | grep -oE '[0-9]+$' | head -1)

# 自动点 launcher 5 tab
python scripts/drive-test.py --port "$PORT"
```

**典型错误**:

- `URLError` (exit 7):bridge 端口没起 / 进程崩 / log 里没 `listening on 127.0.0.1:PORT`。先 `curl http://127.0.0.1:$PORT/status` 验证。
- 打印 `[no widget] 加密`:widget label 改了。脚本用硬编码字符串定位,需同步更新源码。
- 卡在某步不返回:click 没触发 UI 更新。检查 splash on_click 是否接入。

**相关脚本**:

- 兄弟脚本 [`verify.py`](#7-verifypy):坐标硬编码,验证 5 个**行情 tab**;本脚本走 widget label,验证 **launcher 5 tab** 路由。

---

## 4. `install-as-makepad-app.py`

**用途:** **当前推荐路径**。Build finance-brief + 把二进制写到 `~/.octosense/apps.json`(用户级 catalog,不动 dep 仓库),shell 启动器就能看到 "财经简报" tile。包含 makepad rev 对齐检查、UTF-8 写盘、跨平台 binary 后缀(`.exe` / 没有)。

**前置依赖**:

- Python 3.10+(`dict | None`、`from __future__ import annotations` 不在这里但用了 PEP 604 语法)。
- `cargo` 在 PATH 上(`cargo build --release`)。
- 可选:同级 `../makepad` git 仓库(用来 rev-align 检查)。不存在就跳过对齐检查。
- 可选:环境变量 `OCTOSENSE_HOME`(决定 `user_apps` 写到哪)。

**用法**:

```sh
python scripts/install-as-makepad-app.py [--uninstall | --dry-run]
```

**参数**(来自 `--help`):

- `--uninstall`(flag,与 `--dry-run` 互斥):从 `~/.octosense/apps.json` 删 `finance_brief` 项,**不 build、不 verify rev**。
- `--dry-run`(flag,与 `--uninstall` 互斥):只 echo 会做什么(打印 `cargo build` 命令 + 打印会写到的 JSON 行),不 build、不写。

**4 步流程**:

1. `cargo build --release --manifest-path apps/desktop/Cargo.toml`(`--uninstall` / `--dry-run` 跳过)。
2. 验证产生的 binary 存在且可执行(Windows 只验存在,POSIX 验 `-x`)。失败 exit `4`。
3. 验证 `../makepad` HEAD 与 `apps/desktop/Cargo.toml` 中 `makepad-widgets` pin 的 rev 一致(存在 `../makepad` 才做)。失败 exit `3`。
4. 写 `~/.octosense/apps.json`(idempotent):删旧 `finance_brief` 项,然后追加新项(除非取消 `--uninstall`)。

**退出码**:

- `0`:成功。
- `2`:argparse 参数错。
- `3`:makepad rev 不对齐(`OctoSense/AGENTS.md §2 "One revision per external dependency"`)。
- `4`:binary 不存在 / 不可执行。
- cargo 自己的退出码:透传(`subprocess.run(..., check=True)` → `SystemExit(e.returncode)`)。

**示例**:

```sh
# 推荐路径:注册 finance-brief 到 OctoSense 启动器
python scripts/install-as-makepad-app.py
cd ../OctoSense && cargo run --release -p octosense
# → 启动器 tile "财经简报" 点击后拉起 finance-brief 独立窗口

# 看会做什么
python scripts/install-as-makepad-app.py --dry-run

# 撤掉注册
python scripts/install-as-makepad-app.py --uninstall

# 完整 v8 流程(README §3.2 摘要)
python scripts/install-as-makepad-app.py \
    && python scripts/diagnose-shell-state.py \
    && python scripts/drive-test.py --port <PORT>
```

**典型错误**:

- `subprocess.CalledProcessError`(cargo 失败):cargo 脚本带常见 exit code 1 / 101。先 `cargo build --release --manifest-path apps/desktop/Cargo.toml` 单独跑定位。
- exit `3`(rev mismatch):拉 `../makepad` 到 `apps/desktop/Cargo.toml` pin 的那行 `Target-URLrev=` 指定的 commit,然后再跑 install。
- exit `4`(missing binary):build 没成功产物(磁盘满 / 编译错误)。看 cargo 输出。
- `FileNotFoundError`:`apps/desktop/Cargo.toml` 不存在(目录结构错了)。

**相关脚本**:

- 配 [`run-octosense.py`](#6-run-octosensepy):**不要** 用 `run-octosense.py`,因为它调的是 `install-as-makepad-app.py`(默认 rebuild 太重,不是 "只想刷新 catalog" 的场景)。推荐直接 `cargo run -p octosense`(在 `../OctoSense/` 仓库里),因为 shell 启动时会自动读 `~/.octosense/apps.json`。如果你只想刷新 catalog 不 rebuild,用 [`register-with-shell.py`](#8-register-with-shellpy)。
- 清理误装:看 [`clean-shell-pollution.py`](#1-clean-shell-pollutionpy) / [`install-as-system-app.py --uninstall`](#5-install-as-system-apppy)。

---

## 5. `install-as-system-app.py`

**用途:** **legacy**。把 finance-brief bundle 拷到 `OctoSense/apps/finance-brief/bundle/`(写 dep 仓库),为老 Page-format bundle(`*.card` 文件、`workflow.octoscript` / `kit/` / `assets/`)设计。当前仓库用 Path-1 bundle(`bundle/screens/*.octoscript`),**运行此脚本会装出空 bundle**,然后 WARNING 提示改用 `install-as-makepad-app.py`。

**前置依赖**:

- Python 3.10+。
- 同级 `OctoSense/` 仓库必须存在且含 `Cargo.toml`(否则 exit `1`)。
- `bundle/launcher.card`、`bundle/manifest.json`、`bundle/listing.json` 等老格式文件(Path-1 仓库这些都没有,只会 warning 跳过)。

**用法**:

```sh
python scripts/install-as-system-app.py [--dry-run | --uninstall]
```

**参数**(来自 `--help`):

- `--dry-run`(flag):打印每个文件操作(JSON diff / copy / mkdir),不实际写。
- `--uninstall`(flag,与 `--dry-run` 互斥):删 `system-apps.json` 里 `finance-brief` 项、删 `apps/finance-brief/` 整个目录、清 `~/.octosense/apps/.system/os.finance-brief/`。

**5 处改造**:

1. `bundle/listing.json` → 重写 `icon` 为 `"icon.svg"`(系统在系统应用打包时不会 honor `assets/` 子目录)。
2. `bundle/manifest.json` → 重写 `id` 为 `"os." + <short>`(系统包 prefix 为 `os.`),清空 `integrity.bundle_blake3`(build 时盖戳)。
3. `OctoSense/desktop/system-apps.json` → `apps` 数组加 `"finance-brief"` 字符串。
4. 拷 `*.card` / `launcher.card → page.card` / `schema/` / `workflow.octoscript` / `kit/` / `assets/icon.svg` / `screenshots/*.png` 到 `OctoSense/apps/finance-brief/bundle/`。
5. 写 `OctoSense/apps/finance-brief/bundle/page.data.json` = `{}`。

每一步缺源文件就 warning + 跳过,不崩(防御式)。

**退出码**:

- `0`:成功(或 `--dry-run` 打印完毕 / `--uninstall` 全部清完)。
- `1`:同级无 `OctoSense/Cargo.toml`(验 dep 仓库)。
- `2`:argparse 参数错。

**示例**:

```sh
# 看会做什么(强烈推荐先跑)
python scripts/install-as-system-app.py --dry-run

# 实际装(对 Path-1 bundle 会 warning 提示用 install-as-makepad-app.py)
python scripts/install-as-system-app.py

# 清理一次污染(legacy scripts 误跑后)
python scripts/install-as-system-app.py --uninstall
```

**典型错误**:

- exit `1`:同级没 `OctoSense/` 仓库。检查目录结构。
- 装完报 `page.card 系统找不到指定文件 (os error:2)` (从 shell log):bundle 没 `launcher.card`(Path-1 没有),所以 `page.card` 也没创建。**正确路径**:用 `install-as-makepad-app.py` + `~/.octosense/apps.json`,不要用这个脚本。
- `--uninstall` 不报错但 `system-apps.json` 没变:之前没用这个脚本装过(装的是 makepad path)。正常 no-op。

**相关脚本**:

- 替代品 [`install-as-makepad-app.py`](#4-install-as-makepad-apppy):当前 Path-1 bundle 下唯一正确的注册路径。
- 清理:`--uninstall` = [`clean-shell-pollution.py`](#1-clean-shell-pollutionpy) 的功能子集。`clean-shell-pollution.py` 扫得更广,但 `install-as-system-app.py --uninstall` 是"取消一次安装"的语义更清晰。

---

## 6. `run-octosense.py`

**用途:** 一键装 + 跑 OctoSense shell。内部就是 `pip install-as-makepad-app.py` + `cargo run --release -p octosense`。

**前置依赖**:

- Python 3.10+。
- `cargo` 在 PATH 上(`cargo run`)。
- 同级 `OctoSense/` 仓库。

**用法**:

```sh
python scripts/run-octosense.py
```

**参数**:**无**。脚本内有 `argparse.ArgumentParser` 但没注册任何 flag(`-h, --help` 是自动的)。

**行为**:

1. `subprocess.run([python, install-as-makepad-app.py], check=True)`。
2. `subprocess.run(["cargo", "run", "--release", "-p", "octosense"], check=True, cwd=WS / "OctoSense")`。

**退出码**:

- `0`:成功。
- `非 0`(cargo 的或子进程的):透传。

**示例**:

```sh
# 一键装 + 跑(Path-1 bundle 下)
python scripts/run-octosense.py
```

**典型错误**:

- cargo 编译失败:看 cargo 输出。
- `install-as-makepad-app.py` 报错(rev 不对齐 / binary 缺失):exit 码透传。

**相关脚本**:

- **当前 Path-1 bundle 下推荐**(README §3.2 明确建议改用 `install-as-makepad-app.py` + `cargo run -p octosense`,`run-octosense.py` 就是这两个步骤的快捷方式)。
- 等价手动流程:
  ```sh
  python scripts/install-as-makepad-app.py
  cd ../OctoSense && cargo run --release -p octosense
  ```

---

## 7. `verify.py`

**用途:** remote bridge 验证 5 个行情 tab(`要闻` / `A股` / `美股` / `加密` / `外汇`)+ `收藏` tab + `刷新` 按钮的 label。每步点固定坐标 → 抓 `/snap` → 打印 `Label` widget 文本序列。

**前置依赖**:

- finance-brief 已 build 且 `--remote=0` 启动(bind ephemeral port)。
- 行情 tab 用 `quote_list` 屏幕布局(本仓库 Path-1 的 `bundle/screens/quote_list.octoscript`)。

**用法**:

```sh
python scripts/verify.py <port>
python scripts/verify.py --port <port>
```

**参数**(来自 `--help`):

- `port`(位置,int,可选):server port(位置形式)。
- `--port PORT_FLAG`(flag,int,可选):server port(flag 形式,与位置互斥)。flag 优先。

**坐标硬编码(对应当前 `quote_list` 布局)**:

| Tab | x |
|-----|---|
| 要闻 | `41` |
| A股 | `101` |
| 美股 | `161` |
| 加密 | `223` |
| 外汇 | `287` |
| 收藏 | `348` |
| 刷新按钮 | `(367, 87)` |

每次 click 后 sleep 1s,刷新后 sleep 6s。

**退出码**:

- `0`:跑完。
- `2`:`argparse.error("the following arguments are required: port")`(没传 port)。
- `7`:`urllib.error.URLError`,bridge 不通。

**示例**:

```sh
PORT=$(grep -oE 'listening on 127.0.0.1:[0-9]+' /tmp/fb.log | grep -oE '[0-9]+$' | head -1)
python scripts/verify.py --port "$PORT"

# 或者位置形式
python scripts/verify.py "$PORT"
```

**典型错误**:

- `URLError` (exit 7):bridge 不通。先 `curl http://127.0.0.1:$PORT/status` 验。
- 点错坐标 / 没反应:布局改了。改 `apps/desktop/src/app.rs` 或 `bundle/screens/quote_list.octoscript` 后同步更新本脚本里的坐标。
- 没输出 label:`/snap` JSON 里没有 `ty == "Label"` 字段,或 layout 全 0×0。已知 v8 bug 已修(见 `apps/desktop/src/app.rs:60` 的 `View{height: Fit, {ui}}` 修复)。

**相关脚本**:

- 兄弟 [`drive-test.py`](#3-drive-testpy):按 widget label 走 launcher 5 tab 路由;本脚本按坐标走行情 tab label 序列。两者互补,不重叠。
- 修根因:`.todo-finance-brief-splash-zero-rect-2026-10-05.md`(Splash 子树 rect 0×0 v8 修复)。

---

## 8. `register-with-shell.py`

**用途:** 轻量只注册脚本。写 `~/.octosense/apps.json`,**不 rebuild**。区别于 [`install-as-makepad-app.py`](#4-install-as-makepad-apppy)(默认会跑 `cargo build`)。

**用法:**

```bash
python scripts/register-with-shell.py             # 检查 binary + 写 entry
python scripts/register-with-shell.py --dry-run   # 输出 entry,不写
python scripts/register-with-shell.py --uninstall # 移除 entry
python scripts/register-with-shell.py --help
```

**行为:**

- 二进制不存在 → 退出码 **4**(与 [`install-as-makepad-app.py`](#4-install-as-makepad-apppy) 对齐)
- catalog 不存在 → 自动创建父目录 + 空数组
- catalog 不是 JSON 数组 → 退出码 `1`,stderr 拒绝覆盖
- `--uninstall` 互斥 `--dry-run`;没有 entry → no-op
- 跨平台:Windows / macOS / Linux / WSL;`OCTOSENSE_HOME` env 优先于 `Path.home()`;Windows 自动加 `.exe` 后缀

**为什么这个脚本存在:** 用户原话 *"apps.json 加 finance-brief 条目 这里不能直接写到依赖项目里"*。直接写 `OctoSense/desktop/config/apps.json` 是改 dep 仓;[`install-as-makepad-app.py`](#4-install-as-makepad-apppy) 默认 rebuild 太重。`register-with-shell.py` 是中间档:写 user-level catalog,不 rebuild。

---

## 9. 跨平台 / Windows 注意事项

所有 `scripts/*.py` 用 stdlib only,`pathlib` 跨平台写法:

| 场景 | 做法 |
| --- | --- |
| `/tmp/foo` 不可用(Windows Python) | 用 `tempfile.gettempdir()` 或 `Path(os.environ.get('TEMP') or Path.home()) / 'foo'` |
| 二进制后缀 | `sys.platform.startswith('win')` → `.exe` |
| 用户 home | `OCTOSENSE_HOME` env 优先于 `Path.home()` |
| PowerShell vs Git Bash | 都直接 `python scripts/foo.py`;不需要 `bash foo.sh` 也不需要 `powershell foo.ps1` |

---

## 附录 A:典型 v8 完整流程

```sh
cd finance-brief/

# 1. 注册(推荐路径)
python scripts/install-as-makepad-app.py

# 2. 验状态(应 exit 0)
python scripts/diagnose-shell-state.py

# 3. 后台起 finance-brief,绑 remote bridge
nohup ./apps/desktop/target/release/finance-brief --remote=0 > /tmp/fb.log 2>&1 &
disown
sleep 5
PORT=$(grep -oE 'listening on 127.0.0.1:[0-9]+' /tmp/fb.log | grep -oE '[0-9]+$' | head -1)

# 4. headless 验证
python scripts/drive-test.py --port "$PORT"     # launcher 5 tab 路由
python scripts/verify.py --port "$PORT"         # 行情 5 tab + 刷新按钮 label

# 5. (可选)启动 shell 看 GUI
cd ../OctoSense && cargo run --release -p octosense
```

## 附录 B:污染恢复流程(shell 报 `page.card 系统找不到指定文件`)

```sh
# 1. 先看是什么污染
python scripts/diagnose-shell-state.py
# 2. 清 shell 缓存 + system-apps.json + apps/finance-brief/
python scripts/clean-shell-pollution.py
# 3. 改走正确路径
python scripts/install-as-makepad-app.py
# 4. 重启 shell
cd ../OctoSense && cargo run --release -p octosense
```

详见 `README.md §3.2` "如果 shell 仍报 page card" 段。