=== octoscript-capabilities tests ===

running 127 tests
....................................................................................... 87/127
........................................
test result: ok. 127 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.59s


=== octoscript-schema tests ===

running 7 tests
.......
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


=== capabilities crate examples ===

=== schema crate examples ===

=== capabilities test names (sample) ===
bounded_worker::tests::direct_discard_stops_the_worker_and_blocks_future_dispatch: test
bounded_worker::tests::error_types_are_usable_with_infallible_supervisors: test
bounded_worker::tests::passes_a_completed_response_without_terminating_the_worker: test
bounded_worker::tests::rejects_a_zero_invocation_deadline: test
bounded_worker::tests::terminates_and_poisons_after_an_inner_transport_failure: test
bounded_worker::tests::treats_a_deadline_race_as_indeterminate_and_discards_the_session: test
bounded_worker::tests::treats_a_host_force_stop_as_indeterminate_and_discards_the_session: test
bounded_worker::tests::treats_a_session_deadline_as_indeterminate_and_discards_the_session: test
fixed_file_catalog::tests::bounds_and_validates_file_content: test
fixed_file_catalog::tests::honors_the_smaller_tool_output_limit_without_truncation: test
fixed_file_catalog::tests::reads_only_registered_canonical_identifiers: test
fixed_file_catalog::tests::registers_a_redacted_text_tool_for_the_default_runtime: test
fixed_file_catalog::tests::rejects_a_non_text_or_unaddressable_catalog_tool: test
fixed_file_catalog::tests::rejects_invalid_configuration_and_catalog_growth: test
fixed_file_catalog::tests::retains_the_opened_file_when_its_path_is_replaced: test
fixed_file_catalog::tests::seals_a_fixed_file_tool_into_the_mobile_profile: test
fixed_file_catalog::tests::tool_handler_redacts_unknown_identifier_details: test
mobile::tests::derives_mobile_json_data_bounds_from_sealed_execution_limits: test
mobile::tests::exports_contiguous_audit_telemetry_after_sealing: test
mobile::tests::injects_and_extracts_bounded_mobile_json_dataflow: test
mobile::tests::preserves_canonical_source_limits_after_catalog_sealing: test
mobile::tests::preserves_explicit_catalog_limits_after_sealing: test
mobile::tests::pumps_a_static_adapter_one_event_loop_tick_at_a_time: test
mobile::tests::rejects_streaming_from_a_static_catalog: test
mobile::tests::seals_a_typed_static_catalog_for_dynamic_dataflow: test
mobile::tests::seals_direct_deferred_capability_modules_for_mobile_dataflow: test
tests::a_claimed_operation_survives_promise_gc_until_its_audit_is_recorded: test
tests::a_host_pump_deadline_prevents_the_local_handler_from_running: test
tests::a_late_external_completion_is_converted_to_a_timeout: test
tests::async_tool_calls_are_denied_before_they_can_suspend: test

=== schema test names (sample) ===
tests::compares_decimal_constraints_without_floating_point_rounding: test
tests::compares_large_integer_constraints_without_floating_point_rounding: test
tests::rejects_invalid_schema_bounds: test
tests::rejects_keywords_that_do_not_apply_to_the_declared_type: test
tests::rejects_schemas_larger_than_the_source_budget: test
tests::supports_schema_annotations_but_rejects_unknown_keywords: test
tests::validates_nested_object_array_and_scalar_constraints: test
