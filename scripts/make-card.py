#!/usr/bin/env python3
# make-card.py — generate docs/finance-brief-card.svg (the README 名片图).
#
# Pure stdlib (no PIL / matplotlib / cairosvg required). The SVG is a single
# self-contained <svg> element using only basic shapes + text + gradients, so
# GitHub README renders it via <img src=...> and most modern viewers scale it
# cleanly. PNG export is a separate step (use any tool — e.g. browser print
# to PDF, or Inkscape, or Edge headless).
#
# Usage:
#   python scripts/make-card.py
#   python scripts/make-card.py --out docs/finance-brief-card.svg
#
# Design references (read first if editing):
#   - bundle/screens/launcher.octoscript (12 tiles, the visible product)
#   - README.md §1 (single Octoscript path) + §2 (5 data sources)
#   - bundle/kit/ (M3 palette tokens: fb_accent, fb_green_up, fb_red_down, ...)
import argparse
import sys
from pathlib import Path


W, H = 1280, 400
PAD = 60


def data_sources():
    # Order / colors match finance-brief/data sources §2 (README)
    return [
        ("新浪财经",  "Sina",      "#EF4444"),
        ("腾讯证券",  "Tencent",   "#10B981"),
        ("Stooq",     "Stooq",     "#22D3EE"),
        ("Hyperliquid","Crypto",   "#A855F7"),
        ("Frankfurter","ECB FX",   "#F59E0B"),
    ]


def chart_points():
    # Hand-tuned, illustrative trend (no real data).
    # x in [30, 500] step 50, y in [25, 145]
    return [
        (30, 130), (80, 110), (130, 125), (180, 95),
        (230, 105), (280, 75), (330, 85),  (380, 50),
        (430, 60), (480, 30), (500, 25),
    ]


def svg_header():
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" '
        'xmlns:xlink="http://www.w3.org/1999/xlink" '
        f'viewBox="0 0 {W} {H}" width="{W}" height="{H}" '
        'role="img" aria-label="finance-brief 财经简报 — OctoSense 应用">\n'
    )


def svg_defs():
    return """  <defs>
    <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#0F172A"/>
      <stop offset="55%" stop-color="#1E293B"/>
      <stop offset="100%" stop-color="#0B1220"/>
    </linearGradient>
    <linearGradient id="chartFill" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" stop-color="#22D3EE" stop-opacity="0.50"/>
      <stop offset="100%" stop-color="#22D3EE" stop-opacity="0.0"/>
    </linearGradient>
    <linearGradient id="chartLine" x1="0%" y1="0%" x2="100%" y2="0%">
      <stop offset="0%" stop-color="#10B981"/>
      <stop offset="100%" stop-color="#22D3EE"/>
    </linearGradient>
    <linearGradient id="pillHi" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.08"/>
      <stop offset="100%" stop-color="#FFFFFF" stop-opacity="0.03"/>
    </linearGradient>
    <pattern id="grid" x="0" y="0" width="40" height="40" patternUnits="userSpaceOnUse">
      <path d="M 40 0 L 0 0 0 40" fill="none" stroke="#FFFFFF" stroke-width="0.6" opacity="0.05"/>
    </pattern>
    <style>
      .ff-sc { font-family: "PingFang SC", "Microsoft YaHei", "Source Han Sans SC", "Noto Sans CJK SC", sans-serif; }
      .ff-en { font-family: "Segoe UI", "SF Pro Text", "Inter", system-ui, sans-serif; }
      .ff-mono { font-family: "JetBrains Mono", "SF Mono", "Consolas", ui-monospace, monospace; }
    </style>
  </defs>
"""


def svg_background():
    return (
        f'  <rect width="{W}" height="{H}" fill="url(#bg)"/>\n'
        f'  <rect width="{W}" height="{H}" fill="url(#grid)"/>\n'
        # Top-edge accent bar
        f'  <rect x="0" y="0" width="{W}" height="3" fill="#22D3EE"/>\n'
        # Bottom subtle bar
        f'  <rect x="0" y="{H-1}" width="{W}" height="1" fill="#22D3EE" opacity="0.25"/>\n'
    )


def svg_title_block():
    out = []
    out.append('  <!-- title block -->\n')
    out.append(f'  <g transform="translate({PAD}, 70)">\n')
    # Eyebrow pill
    out.append(
        '    <g>\n'
        '      <rect x="0" y="0" width="140" height="28" rx="14" '
        'fill="#22D3EE" fill-opacity="0.12" stroke="#22D3EE" stroke-opacity="0.55"/>\n'
        '      <circle cx="16" cy="14" r="3.5" fill="#22D3EE"/>\n'
        '      <text x="28" y="19" class="ff-en" fill="#67E8F9" '
        'font-size="12" font-weight="600" letter-spacing="1">OCTOSENSE APP</text>\n'
        '    </g>\n'
    )
    # Main title (Chinese)
    out.append(
        '    <text x="0" y="100" class="ff-sc" fill="#F1F5F9" '
        'font-size="68" font-weight="700" letter-spacing="3">财经简报</text>\n'
    )
    # English title
    out.append(
        '    <text x="0" y="142" class="ff-mono" fill="#22D3EE" '
        'font-size="22" font-weight="500" letter-spacing="2">finance-brief</text>\n'
    )
    # Tagline
    out.append(
        '    <text x="0" y="178" class="ff-sc" fill="#CBD5E1" '
        'font-size="17" font-weight="400">单一 Octoscript 路径 · 12 界面 · 5 数据源 · native widgets via makepad</text>\n'
    )
    out.append('  </g>\n')
    return "".join(out)


def svg_chart():
    out = []
    out.append('  <!-- chart -->\n')
    out.append(f'  <g transform="translate(700, 56)">\n')
    # Card backdrop
    out.append(
        '    <rect x="0" y="0" width="520" height="188" rx="10" '
        'fill="#FFFFFF" fill-opacity="0.04" stroke="#22D3EE" stroke-opacity="0.22"/>\n'
    )
    # Header bar inside card
    out.append(
        '    <rect x="0" y="0" width="520" height="22" rx="10" '
        'fill="#22D3EE" fill-opacity="0.06"/>\n'
        '    <text x="14" y="15" class="ff-en" fill="#94A3B8" '
        'font-size="10" font-weight="600" letter-spacing="2">QUOTE · sample trend (illustrative)</text>\n'
        '    <circle cx="500" cy="11" r="3" fill="#10B981"/>\n'
    )
    # Grid lines
    for y in (52, 90, 128):
        out.append(
            f'    <line x1="20" y1="{y}" x2="500" y2="{y}" '
            'stroke="#FFFFFF" stroke-opacity="0.07" stroke-width="0.6"/>\n'
        )
    # Build line path
    pts = chart_points()
    line_d = "M " + " L ".join(f"{x} {y}" for x, y in pts)
    fill_d = line_d + f" L 500 165 L 30 165 Z"
    out.append(f'    <path d="{fill_d}" fill="url(#chartFill)"/>\n')
    out.append(
        f'    <path d="{line_d}" fill="none" stroke="url(#chartLine)" '
        'stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round"/>\n'
    )
    # Candles (a few, illustrative)
    candles = [(80, 110, 105, 122, "#10B981"), (180, 95, 90, 105, "#10B981"),
               (280, 75, 70, 88, "#F87171"), (380, 50, 45, 65, "#10B981"),
               (480, 30, 22, 42, "#10B981")]
    for cx, _, top, bot, color in candles:
        out.append(
            f'    <rect x="{cx-3}" y="{top}" width="6" height="{bot-top}" fill="{color}" opacity="0.8"/>\n'
        )
    # Last-point emphasis
    last_x, last_y = pts[-1]
    out.append(f'    <circle cx="{last_x}" cy="{last_y}" r="5" fill="#22D3EE" stroke="#0F172A" stroke-width="2"/>\n')
    # Y-axis labels (decorative)
    for y, label in [(52, "+5%"), (90, "0"), (128, "-3%")]:
        out.append(
            f'    <text x="14" y="{y+3}" class="ff-mono" fill="#475569" font-size="9">{label}</text>\n'
        )
    out.append('  </g>\n')
    return "".join(out)


def svg_data_sources():
    out = []
    out.append('  <!-- data sources -->\n')
    out.append(f'  <g transform="translate({PAD}, 300)">\n')
    out.append(
        '    <text x="0" y="0" class="ff-en" fill="#64748B" '
        'font-size="10" font-weight="700" letter-spacing="2.5">DATA SOURCES</text>\n'
    )
    sources = data_sources()
    pill_w, pill_h = 122, 34
    gap = 8
    x = 0
    for cn, en, color in sources:
        out.append(f'    <g transform="translate({x}, 14)">\n')
        out.append(
            f'      <rect x="0" y="0" width="{pill_w}" height="{pill_h}" rx="7" '
            'fill="url(#pillHi)" stroke="#FFFFFF" stroke-opacity="0.10"/>\n'
        )
        out.append(f'      <circle cx="14" cy="17" r="4" fill="{color}"/>\n')
        out.append(
            f'      <text x="26" y="16" class="ff-sc" fill="#E2E8F0" '
            f'font-size="12" font-weight="500">{cn}</text>\n'
        )
        out.append(
            f'      <text x="26" y="28" class="ff-en" fill="#64748B" '
            f'font-size="9" letter-spacing="0.5">{en}</text>\n'
        )
        out.append('    </g>\n')
        x += pill_w + gap
    out.append('  </g>\n')
    return "".join(out)


def svg_stat_chips():
    out = []
    out.append('  <!-- stat chips -->\n')
    # Place top-right corner of the chip row
    chips = [
        ("12", "界面", "#22D3EE"),
        ("5", "数据源", "#10B981"),
        ("1", "Octoscript 路径", "#A855F7"),
        ("✓", "native widgets", "#F59E0B"),
    ]
    chip_w = 122
    gap = 8
    # Right-align to right padding
    total_w = chip_w * len(chips) + gap * (len(chips) - 1)
    x0 = W - PAD - total_w
    y0 = 290
    out.append(f'  <g transform="translate({x0}, {y0})">\n')
    x = 0
    for big, small, color in chips:
        out.append(f'    <g transform="translate({x}, 0)">\n')
        out.append(
            f'      <rect x="0" y="0" width="{chip_w}" height="60" rx="10" '
            'fill="#FFFFFF" fill-opacity="0.04" stroke="#FFFFFF" stroke-opacity="0.10"/>\n'
        )
        # left accent bar
        out.append(f'      <rect x="0" y="0" width="3" height="60" rx="1.5" fill="{color}"/>\n')
        out.append(
            f'      <text x="14" y="30" class="ff-en" fill="{color}" '
            f'font-size="22" font-weight="700">{big}</text>\n'
        )
        out.append(
            f'      <text x="14" y="48" class="ff-sc" fill="#94A3B8" '
            f'font-size="11" font-weight="500">{small}</text>\n'
        )
        out.append('    </g>\n')
        x += chip_w + gap
    out.append('  </g>\n')
    return "".join(out)


def svg_footer():
    return (
        '  <!-- footer -->\n'
        f'  <text x="{PAD}" y="378" class="ff-en" fill="#475569" '
        'font-size="11" letter-spacing="0.5">Native widgets via Octoscript DSL → makepad · Apache-2.0</text>\n'
        # Top-right small OctoSense mark
        f'  <text x="{W-PAD}" y="378" text-anchor="end" class="ff-mono" '
        'fill="#475569" font-size="11" letter-spacing="1">// OctoSense</text>\n'
    )


def build_svg():
    parts = [
        svg_header(),
        svg_defs(),
        svg_background(),
        svg_title_block(),
        svg_chart(),
        svg_data_sources(),
        svg_stat_chips(),
        svg_footer(),
        '</svg>\n',
    ]
    return "".join(parts)


def main():
    ap = argparse.ArgumentParser(prog="make-card.py",
                                 description="Generate docs/finance-brief-card.svg (the README 名片图).")
    ap.add_argument("--out", type=Path, default=Path("docs/finance-brief-card.svg"),
                    help="output SVG path (default: docs/finance-brief-card.svg)")
    args = ap.parse_args()

    args.out.parent.mkdir(parents=True, exist_ok=True)
    svg = build_svg()
    args.out.write_text(svg, encoding="utf-8")
    print(f"wrote {args.out.resolve()} ({len(svg)} bytes, {W}x{H})")


if __name__ == "__main__":
    main()