# Issue draft — Octoscript L0 catalog extensibility for host-registered sys.* helpers

> **Status**: draft, not yet submitted to https://github.com/OctoSense-org/Octoscript
> **Author intent**: lumina (ld1287@users.noreply.github.com)
> **Submitted-by** (you): <fill before pushing>
> **Target repo**: `OctoSense-org/Octoscript`
> **Reference commit**: `68f6a9df55692b5d8ef8873a12721e279a3f40d6` (HEAD on this checkout)

---

## Title

`octoscript-ui-l0`: allow hosts to register additional `sys.*` helpers (instead of the hard-coded 32-entry `catalog::ANSWERS` const)

## Summary

`octoscript-ui-l0::catalog::ANSWERS` is a `pub const &[(&str, &[&str])]` of 32 entries (`sys.airquality`, `sys.cities`, `sys.convert`, ..., `sys.wiki`). Every host-side data source must use one of these names AND only request fields listed there. A consumer application that wants to expose its own services (e.g. our `finance-brief` app registers 17 capabilities: `news_sina`, `quote_tencent`, `quote_stooq`, `quote_hyperliquid`, `quote_frankfurter`, `synth_candles`, `stream_subscribe`, `stream_tick`, `datasource_status`, ...) cannot do so without:

- Forking `octoscript-ui-l0` and adding entries to `ANSWERS`, OR
- Editing `lib.rs:3750-3911` in-place (which violates the upstream-first rule we follow).

Neither is sustainable for an app ecosystem that wants each app to ship its own catalog.

## Repro (minimal)

Given a host that wants to expose a `news_sina` source with fields `[id, title, source, ts, key, summary]`, writing the L0 card:

```text
source feed sys.news_sina(count: 20, fields: [title, source, ts])
```

produces:

```text
error: "sys.news_sina" is not a source capability L0 admits;
       a card may only name a catalogued helper (profile §4).
       Known: sys.airquality, sys.cities, sys.convert, ..., sys.wiki
```

The same error appears for the other 16 helpers we want to expose.

## Why this matters for downstream apps

- The `OctoSense` shell hosts multiple apps side-by-side; each app owns its own data plane (HTTP, schemas, capability list).
- Apps need to declare what `sys.*` helpers they expose without touching `octoscript-ui-l0` source.
- The current design forces every app to either be a fixture (data baked into `ANSWERS`) or fork the engine.
- This blocks the natural workflow: an app author writes `<app>/bundle/main.card`, ships it to a host, and the host wires `sys.*` calls to whatever adapter implements the contract.

## Proposed design (preferred path)

Add a runtime-registered catalog layer **on top of** the existing `ANSWERS`:

1. **Keep `ANSWERS` as the canonical "build-time" fixture catalog.** No change to its 32 entries.
2. **Add `pub fn register(name: &str, fields: &[&'static str]) -> Result<(), CatalogError>`** that hosts call at boot to layer additional helpers on top of `ANSWERS`.
3. **Add `pub fn answers(name: &str) -> Option<&'static [&'static str]>`** that the existing `validate_sources` (lib.rs:4643-4647) uses to look up helpers — change the call site from `ANSWERS.iter().find(...)` to `register_table.iter().chain(ANSWERS.iter()).find(...)`.
4. **Add `pub fn snapshot() -> Vec<String>`** for tooling (cards dump, `check_card` diagnostics, etc.).
5. **Add per-entry contracts** (mandatory fields, optional fields, return schema URI) — `register(name, fields)` becomes `register(Contract { name, required, optional, schema })`. The schema URI is opaque to L0 but lets `lower_l0` and the makepad adapter pull real type info at lower time.
6. **Document** in `docs/ui-profile-l0.md` §4 that catalogs are now layered, and add a `register_layered_example.rs` to `octoscript-ui-l0/examples/`.

This is a non-breaking extension. The existing 32 hard-coded entries stay valid; new apps opt in by calling `register(...)` at boot, exactly the same way `CapabilityRuntime` registers tools today.

### Sketch

```rust
// octoscript-ui-l0/src/lib.rs (new module)
pub mod catalog {
    const ANSWERS: &[(&str, &[&str])] = &[ /* unchanged 32 */ ];

    static REGISTERED: OnceLock<RwLock<Vec<Contract>>> = OnceLock::new();

    pub fn register(contract: Contract) -> Result<(), String> {
        // reject if name collides with build-time ANSWERS
        if ANSWERS.iter().any(|(n, _)| *n == contract.name) {
            return Err(format!("name {:?} collides with build-time catalog", contract.name));
        }
        REGISTERED.get_or_init(Default::default).write().unwrap().push(contract);
        Ok(())
    }

    pub fn answers(name: &str) -> Option<&'static [&'static str]> {
        // 1. build-time
        if let Some((_, fields)) = ANSWERS.iter().find(|(n, _)| *n == name) { return Some(*fields); }
        // 2. runtime registered
        REGISTERED.get().and_then(|t| t.read().unwrap().iter().find(|c| c.name == name).map(|c| c.fields))
    }
}
```

### Verification

- Add a unit test `octoscript-ui-l0/tests/catalog_register.rs` that:
  - Registers `sys.fake_helper(fields: ["a", "b"])`.
  - Parses a `feed.sys.fake_helper(fields: [a, b])` card via `check_ui_l0_named` and asserts `valid = true`.
  - Re-registers the same name and asserts `Err(_)` (collision).
- Add `octoscript-ui-l0/examples/register_layered.rs` mirroring the `finance-brief` style (17 capabilities at once).

## Alternative considered (rejected)

- **Pre-baked entries** for `news_sina` / `quote_tencent` / etc. in `ANSWERS`: rejected. This would (a) lock the engine to one app's vocabulary, (b) make every consumer fork on rename, (c) require re-releasing `octoscript-ui-l0` for every new app.
- **Make `ANSWERS` `pub static mut`**: rejected. Mutable statics + unsafe is exactly the wrong shape for an app ecosystem; concurrent hosts would race.
- **Bypass L0 entirely and feed raw Makepad DSL**: rejected. The whole point of `lower_l0` is to keep the data binding confined; bypassing it reintroduces `{{state.x}}` placeholders, ad-hoc widget composition, and the `view_quotes` bug class we've been chasing.

## Willing to do

We (the `finance-brief` team) can:

- Open the PR.
- Provide the `register` / `Contract` types with full unit tests.
- Migrate the 4 entry points that read `ANSWERS` (`validate_sources`, `answers`, `lower_kit`, `lower_l0`'s source-resolution path) to the layered lookup.
- Update `docs/ui-profile-l0.md` §4 with the layered-catalog contract.

What we need from the maintainers:

- A maintainer-side review of the `Contract` shape (whether `schema: Option<String>` for the runtime schema URI is the right escape hatch).
- Guidance on whether `register` should be infallible (panic on collision) or return `Result` (we prefer `Result` for app boot).
- A release-train commitment so we can depend on the layered API within the next `OctoSense` app-store cycle.

## Reproduction context

- `octoscript` checkout: `68f6a9df55692b5d8ef8873a12721e279a3f40d6`
- Card file: `/home/lumina/octoOs/finance-brief/bundle/news_list.card` line 5: `source feed sys.news_sina(count: 20, fields: [title, source, ts])`
- Error output: `error: "sys.news_sina" is not a source capability L0 admits; a card may only name a catalogued helper (profile §4).`
- Internal references: `octoscript-ui-l0/src/lib.rs:3656-3935` (`pub mod catalog`), `lib.rs:4643-4647` (`validate_sources`).

## Checklist

- [x] I've searched existing issues and PRs in `OctoSense-org/Octoscript` — no existing proposal.
- [x] I've read `docs/ui-profile-l0.md` §4 (catalog).
- [x] I understand the layered approach keeps all existing fixture entries valid.
- [x] I'm willing to open the PR with tests + migration.
- [ ] I have NOT modified `octoscript-ui-l0` locally to work around this (our local copy was rolled back to `68f6a9df` clean on 2026-10-03).