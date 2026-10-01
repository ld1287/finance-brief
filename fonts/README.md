# Fonts

Chinese (Simplified) font assets for the `finance-brief` splash UI.

## Upstream source

Copied from `OctoScript-App-Design-Flow/examples/health/fonts/`
(`NotoSansSC` family, OFL-licensed).

- Commit: not tracked (plain file copy).
- Copy date: 2026-10-01.

## License

SIL Open Font License v1.1 (OFL-1.1). Full text in [`OFL.txt`](./OFL.txt).

The OFL permits redistribution and bundling in source/binary form, provided
the license and copyright notice are preserved. These files are committed to
the repo so the splash runtime can resolve `{{assets}}/fonts/...` paths
without any network fetch.

## Files

| File | Size | sha256 (first 8) | Role |
| --- | --- | --- | --- |
| `NotoSansSC-Regular.ttf` | – | – | Default body weight |
| `NotoSansSC-Medium.ttf`  | – | – | UI emphasis mid |
| `NotoSansSC-Bold.ttf`    | – | – | Headlines / strong |
| `NotoSansSC-variable.ttf`| – | – | Variable-axis master (see note) |

(Re-run `sha256sum fonts/*.ttf` in this folder for current hashes.)

## Usage in splash

Splash `Label` widgets reference the font via the `{assets}` placeholder:

```text
draw_text.text_style.font: {{assets}}/fonts/NotoSansSC-Regular.ttf
```

The runtime resolves `{{assets}}` to the on-disk assets directory shipped
with the bundle, which is laid out to mirror the `finance-brief/fonts/`
directory. See the splash documentation for the full placeholder syntax
and supported `text_style` keys.

## Notes

- `NotoSansSC-variable.ttf` is a **variable font** (axis: `wght`).
  The splash font loader may not yet support variable fonts; if the
  variable file fails to load, fall back to the three static masters
  (`Regular` / `Medium` / `Bold`) explicitly.
- If you add or remove a font file, re-run `ls fonts/` and update this
  README so the table stays accurate.
- This directory is **not** gitignored — the four `.ttf` files and the
  `OFL.txt` license are intentionally tracked in git.