=== octoscript-ui-l0 crate identification ===
package: octoscript-ui-l0 v0.1.0
location: octoscript/crates/octoscript-ui-l0/
crate type: library only (no [[bin]] section in Cargo.toml)

=== public API surface used in this verify ===
lib.rs:209  pub fn check_ui_l0(source: &str) -> UiL0Report
lib.rs:214  pub fn check_ui_l0_named(_name: &str, source: &str) -> UiL0Report
lib.rs:5884 pub fn realize(source: &str, data: &serde_json::Value, limits: RealizeLimits) -> RealizeReport
lib.rs:10019 pub fn state_initials(source: &str) -> BTreeMap<String, serde_json::Value>

=== fixture inventory (octoscript/crates/octoscript-ui-l0/tests/fixtures/) ===
activity.card         chart.card            nav-excerpt.octoscript  nav.card
news.card             stock.card            trip_planner.octoscript  weather.card
youtube.card
trip_planner.octoscript size: 56694 bytes (Sep 28 18:48)

=== which octoscript-ui-l0 (binary search) ===
no binary on PATH (expected: lib crate, no bin target)

=== cargo run --bin octoscript-ui-l0 -- --help (expected: refused, no bin target) ===
[refused] cargo: no bin target named `octoscript-ui-l0` in default-run packages

=== substitute entry points: example binaries that wrap the same library fns ===
check_ui_l0  ->  examples/lint_check.rs  (calls octoscript_ui_l0::check_ui_l0_named)
realize      ->  examples/l0validate.rs   (calls octoscript_ui_l0::realize)

=== check_ui_l0 PASS / realize matrix across all 9 fixtures ===
=== activity.card ===
-- lint_check --
activity.card: 2 lint(s), valid=true
  41:19  '🌳' is a colour-font glyph, so it ignores the theme's ink and stays the colour it was drawn in — this card will render correctly in one polarity and invisibly in the other. Use an icon role, or accept that the card is single-theme.
  51:19  '☕' is a colour-font glyph, so it ignores the theme's ink and stays the colour it was drawn in — this card will render correctly in one polarity and invisibly in the other. Use an icon role, or accept that the card is single-theme.
-- l0validate --
{"diagnostics":[],"ok":true,"root":true,"state_initials":{"city":""}}

=== chart.card ===
-- lint_check --
chart.card: portable
-- l0validate --
{"diagnostics":[],"ok":true,"root":true,"state_initials":{"countries":"CHN,IND","metric":"NY.GDP.MKTP.KD.ZG","span":30.0}}

=== nav.card ===
-- lint_check --
nav.card: portable
-- l0validate --
{"diagnostics":[],"ok":true,"root":true,"state_initials":{"dest":"","editing":"none","origin":"","query":"","screen":"plan","stop":"","stop_row":"hidden","view":"tilted"}}

=== news.card ===
-- lint_check --
news.card: portable
-- l0validate --
{"diagnostics":[],"ok":true,"root":true,"state_initials":{"selected":""}}

=== stock.card ===
-- lint_check --
stock.card: portable
-- l0validate --
{"diagnostics":[],"ok":true,"root":true,"state_initials":{"editing":"none","last_act":"none","query":"","selected":""}}

=== weather.card ===
-- lint_check --
weather.card: portable
-- l0validate --
{"diagnostics":[],"ok":true,"root":true,"state_initials":{"city":"","days":7.0,"editing":"none","query":""}}

=== youtube.card ===
-- lint_check --
youtube.card: portable
-- l0validate --
{"diagnostics":[],"ok":true,"root":true,"state_initials":{"editing":"none","q":"lofi hip hop radio","typed":""}}

=== nav-excerpt.octoscript ===
-- lint_check --
nav-excerpt.octoscript: portable
-- l0validate --
{"diagnostics":["`let` is not in L0: state is declared, not bound imperatively"],"ok":false,"root":false,"state_initials":{}}

=== trip_planner.octoscript ===
-- lint_check --
trip_planner.octoscript: portable
-- l0validate --
{"diagnostics":["`let` is not in L0: state is declared, not bound imperatively"],"ok":false,"root":false,"state_initials":{}}


=== summary ===
7 of 9 fixtures: lint_check PASS, realize ok=true (clean root)
2 of 9 fixtures (nav-excerpt.octoscript, trip_planner.octoscript):
  - lint_check reports 'portable' (the let-binding is L2, not an L0 lint)
  - realize refuses with diagnostic: `let` is not in L0: state is declared, not bound imperatively
  - ok=false, root=false in JSON

trip_planner.octoscript (56694 bytes) was authored with imperative let bindings
for state initialization (q, find, orig, dest, wp1, wp2, sel, ss, go, vw, md, oq)
which the L0 profile rejects. This is a real Phase 1 finding: trip_planner.octoscript
is not a valid L0 card as currently authored and cannot be realized under the L0 profile.

verify verdict:
  PASS: 7 fixtures realize cleanly under L0 (activity/chart/nav/news/stock/weather/youtube)
  REFUSED: trip_planner.octoscript (and the sibling nav-excerpt.octoscript) - both use
          let-bindings for state init, which is L2 syntax. Realize correctly rejects them.
  The lint_check gate (check_ui_l0_named) does not catch this because the lint only flags
  theme-portability issues, not L2 lexical violations.

=== end of O-5-verify ===
