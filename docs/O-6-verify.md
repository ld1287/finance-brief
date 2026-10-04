# O-6 Verify: octoscript-workflow engine

Phase 1 verification of the `octoscript-workflow` crate against the two
canonical example scripts shipped in `octoscript/examples/`.

> Re-verified 2026-10-04: content unchanged; 97/97 PASS + 2 example PASS still holds.
## Crate layout (context)

- Crate root: `octoscript/crates/octoscript-workflow/`
- Edition: workspace (2021), version 0.1.0
- `Cargo.toml` declares **no `[[bin]]` and no `[[example]]` targets** —
  `octoscript-workflow` is a pure library. The downstream CLI binary
  `octoscript-cli` (`octoscript` binary at `octoscript/crates/octoscript-cli/`)
  is the production runner that consumes this library.
- Source files:
  - `src/lib.rs` — public types: `WorkflowData`, `WorkflowDraft`,
    `WorkflowStep`, `WorkflowDraftError`, `WorkflowError`,
    `workflow_draft_json_schema()`.
  - `src/mobile.rs` — `MobileWorkflowRuntime` + `MobileWorkflowBuilder`
    (registers text / json / typed-json / capability-module tools, then
    `plan` → `approve_*` → `execute` / `execute_dataflow`).
  - `src/multiplexed_worker.rs` — external-tool worker event handling
    (`request_external_tool_cancellation`, `poll_external_tool`,
    `apply_external_tool_worker_event`).
  - `src/durable_events.rs`, `src/telemetry.rs`, `src/telemetry/*`,
    `src/bubblewrap_recovery.rs` — durability / telemetry / sandbox glue.

## Engine entry points (static)

```
pub fn plan(&mut self, steps: Vec<WorkflowStep>) -> Result<WorkflowPlan, WorkflowError>
pub fn plan_draft(&mut self, draft: WorkflowDraft) -> Result<WorkflowPlan, WorkflowError>
pub fn approve_with_step_capability_policies(...)
pub fn approve_dataflow_with_step_capability_policies(...)
pub fn approve_dataflow_with_contract_and_step_capability_policies(...)
pub fn approve_resume_with_step_capability_policies(...)
pub fn execute(&mut self, plan, approval) -> Result<(), WorkflowError>
pub fn execute_dataflow(&mut self, plan, approval) -> Result<WorkflowData, WorkflowError>
pub fn resume(&mut self, plan, checkpoint, approval) -> Result<(), WorkflowError>
pub fn resume_dataflow(&mut self, plan, checkpoint, approval) -> Result<WorkflowData, WorkflowError>
pub fn dataflow_snapshot(&self) -> Option<&WorkflowData>
pub fn take_dataflow_snapshot(&mut self) -> Option<WorkflowData>
pub fn has_suspended_execution(&self) -> bool
```

The `MobileWorkflowBuilder` exposes typed tool registration helpers
(`register_text_tool`, `register_fixed_file_catalog_tool`,
`register_http_endpoint_catalog_tool`, `register_json_tool`,
`register_typed_json_tool`, `register_capability_module`) which are what
`octoscript-cli/src/main.rs` plugs into at startup.

## Test surface (static)

`src/mobile.rs` ships an in-module `#[cfg(test)] mod tests` block. Running
`cargo test --lib -p octoscript-workflow` exercises 97 unit tests covering
plan / approval / execute / resume / dataflow contract enforcement,
external-tool leases, durable suspension, audit/event ordering, and the
LSP projection helpers in `src/lib.rs`.

## Runtime verification

The pre-built CLI binary `octoscript/target/debug/octoscript` is used as
the workflow runner (its `run` and `tool-calls` subcommands load
`octoscript-workflow::MobileWorkflowRuntime` under the hood).

### 1) `octoscript/examples/tool_workflow.octoscript`

Source:

```octoscript
use mod.tool

let task = "prepare release notes"
let result = tool.call("text.echo", task)
result
```

Static analysis (`tool-calls`):

```json
{"diagnostics":[],"diagnostics_truncated":false,"direct_module_calls":[],
 "tool_calls":[{"callee":{"column":14,"end_byte":71,"line":4,"start_byte":62},
                "kind":"call","name":{"end_byte":83,"kind":"literal",
                "start_byte":72,"value":"text.echo"}}],
 "tool_calls_truncated":false,"valid":true}
```

Execution (`run --allow-echo`):

```
$ octoscript run --allow-echo examples/tool_workflow.octoscript
tool event_sequence=1 sequence=0 name=text.echo outcome=Allowed input_bytes=21 output_bytes=21
```

Exit code: 0. result: PASS — engine allowed the `text.echo` capability and
echoed the 21-byte input back to the host-owned result binding.

### 2) `octoscript/examples/json_tool_workflow.octoscript`

Source:

```octoscript
use mod.tool
use mod.std.assert

let response_json = tool.start_json("math.add", {left: 20, right: 22}).await()
let response = response_json.parse_json()
assert(response.total == 42)
```

Static analysis (`tool-calls`):

```json
{"diagnostics":[],"diagnostics_truncated":false,"direct_module_calls":[],
 "tool_calls":[{"callee":{"column":21,"end_byte":68,"line":4,"start_byte":53},
                "kind":"start_json","name":{"end_byte":79,"kind":"literal",
                "start_byte":69,"value":"math.add"}}],
 "tool_calls_truncated":false,"valid":true}
```

Execution (`run --allow-json-add`):

```
$ octoscript run --allow-json-add examples/json_tool_workflow.octoscript
tool event_sequence=1 sequence=0 name=math.add outcome=Allowed input_bytes=22 output_bytes=12
```

Exit code: 0. result: PASS — engine dispatched the typed JSON tool
`math.add` (22-byte input), received 12-byte output `{total: 42}`, the
script asserted `response.total == 42` without throwing.

### Negative control

Without explicit grants the engine correctly denies tool calls (proves the
policy layer is wired):

```
$ octoscript run examples/tool_workflow.octoscript
diagnostic: inline.octoscript:3:24: tool call denied: no capability grants text.echo
tool event_sequence=1 sequence=0 name=text.echo outcome=Denied input_bytes=21 output_bytes=0
error: script evaluation failed
```

## `cargo test --lib -p octoscript-workflow`

```
test result: ok. 97 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
```

Representative passing tests:

```
test tests::step_capability_policies_issue_ordered_host_authority ... ok
test tests::step_capability_leases_retain_the_current_authority_across_external_await ... ok
test tests::workflow_operation_dispatch_requires_the_persisted_canonical_input ... ok
test tests::workflow_plan_rejects_excess_steps_and_aggregate_source ... ok
test tests::workflow_draft_round_trips_and_reviews_without_authority ... ok
test tests::workflow_data_contract_enforces_an_aggregate_schema_bound ... ok
test tests::suspended_external_workflow_operation_is_durable_before_exact_dispatch ... ok
test tests::workflow_cancels_unawaited_external_work_before_the_next_step ... ok
test tests::workflow_pumps_nonawaited_local_work_before_its_waiting_promise ... ok
test tests::workflow_drains_unawaited_local_work_before_the_next_step ... ok
test tests::suspended_dataflow_lsp_projection_tracks_the_engine_prefix ... ok
test tests::workflow_review_bounds_aggregate_tool_call_hints ... ok
```

## Verdict

- `tool_workflow.octoscript` → PASS (text.echo allowed, output echoed).
- `json_tool_workflow.octoscript` → PASS (math.add allowed, total=42
  asserted).
- `cargo test --lib -p octoscript-workflow` → 97/97 PASS.
- Capability-denial path → correctly rejected when no grant flag passed.

The `octoscript-workflow` engine is the canonical execution surface for
Octoscript scripts and behaves correctly on both bundled examples plus
its full library unit-test matrix.
