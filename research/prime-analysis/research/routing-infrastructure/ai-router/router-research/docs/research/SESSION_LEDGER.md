# Session Ledger

Purpose: preserve a detailed externalized working trace so recovery does not
depend on memory.

What goes here:
- files read
- commands run
- edits made
- conclusions reached
- next actions queued
- explicit recovery checkpoints

What does not go here:
- hidden chain-of-thought
- private internal reasoning that never became an external action

Use the helper:

```bash
python tools/session_ledger.py --kind read --note "Read PROJECT_CONTEXT and ROUTE_MATRIX"
python tools/session_ledger.py --kind decision --note "RR-061 still active; do not continue from INC-0065 yet"
python tools/session_ledger.py --kind next --note "Resume RR-061 with stronger angular-law diagnostics"
```

## 2026-04-05
- 23:00:00 EDT [decision] Froze the router-memory branch as ready for a larger router-native systems prototype and defined the next bounded shared-workspace prototype with the current read path unchanged.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_router_memory_readiness.md, /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_router_memory_systems_prototype.md
- 22:45:00 EDT [edit] Added the first bounded multi-entity router-memory experiment and task-family note using the current read path and promotion logic unchanged.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_multi_entity_router_memory_experiment.py, /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_multi_entity_router_memory_family.md
- 22:45:00 EDT [run] Ran the bounded multi-entity router-memory experiment and saved the coordination summary artifact.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_multi_entity_router_memory_experiment.csv
  Cmd: `python ai-router/router-research/tools/prime_transport/run_multi_entity_router_memory_experiment.py`
- 22:45:00 EDT [decision] The unchanged router-memory architecture remains coherent on bounded multi-entity state coordination and is now strong enough for a future larger router-native systems prototype.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_multi_entity_router_memory_experiment.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 22:25:00 EDT [edit] Added the first larger router-memory experiment on a richer structured-record task family using the current read path unchanged.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_larger_router_memory_experiment.py, /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_router_memory_record_family.md
- 22:25:00 EDT [run] Ran the larger structured-record router-memory experiment and saved the bounded results summary.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_larger_router_memory_experiment.csv
  Cmd: `python ai-router/router-research/tools/prime_transport/run_larger_router_memory_experiment.py`
- 22:25:00 EDT [decision] The unchanged router-memory read path remains coherent on the first richer structured-record task family and is now strong enough to justify a larger router-native systems experiment.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_larger_router_memory_experiment.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 22:10:00 EDT [decision] Froze the router-memory branch status: keep the current read path unchanged, treat promoted-query burden as a real deeper-family architectural cost, and use that as the starting point for the next larger router-native decision.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_router_memory_status.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 22:00:00 EDT [edit] Added a bounded promoted-query reduction comparison for the router-memory task loop using one tiny read-side `phi0` hint refinement.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/router_memory_layer.py, /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_router_memory_promoted_query_reduction.py
- 22:00:00 EDT [run] Ran the promoted-query reduction comparison and saved the baseline-versus-refined memory-task summary.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_router_memory_promoted_query_reduction.csv
  Cmd: `python ai-router/router-research/tools/prime_transport/run_router_memory_promoted_query_reduction.py`
- 22:00:00 EDT [decision] Keep the current router-memory read path unchanged: the tested `phi0` hint lowers promoted-query burden, but it degrades task retrieval accuracy too much to be a worthwhile replacement.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_router_memory_promoted_query_reduction.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 21:45:00 EDT [edit] Added the first bounded stateful task-loop driver on top of the router-memory layer, turning the routing state into an explicit symbolic read/write memory process.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_router_memory_task_loop.py, /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/router_memory_layer.py
- 21:45:00 EDT [run] Ran the bounded router-memory task loop and saved the multi-step state-carry summary artifact.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_router_memory_task_loop.csv
  Cmd: `python ai-router/router-research/tools/prime_transport/run_router_memory_task_loop.py`
- 21:45:00 EDT [decision] The router-memory layer now supports real carried-state retrieval in a bounded task loop strongly enough to justify a larger router-native experiment, with promoted-query burden on deeper lifts as the main remaining architectural cost.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_router_memory_task_loop.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 21:30:00 EDT [edit] Added the first structured read/write router-memory layer on top of the guarded prime-transport policy, with explicit query and write operations over reusable `base_gap` memory cells.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/router_memory_layer.py, /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_router_memory_layer_demo.py
- 21:30:00 EDT [run] Ran the bounded router-memory demo on the existing exact trace family and saved the memory-layer summary artifact.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_router_memory_layer_demo.csv
  Cmd: `python ai-router/router-research/tools/prime_transport/run_router_memory_layer_demo.py`
- 21:30:00 EDT [decision] The router-memory path is strong enough to continue architecturally: the minimal read/write layer is stable and high-resolution after warmup, but exact branch retrieval remains partial and should be the main caution before any larger router-native experiment.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_router_memory_layer.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 21:05:00 EDT [decision] Froze the final real-signal status: keep `base_gap` as the first guarded real-signal policy, stop replay-family micro-refinement, and treat fallback efficiency rather than route reuse as the currently validated win.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_real_signal_status.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 20:45:00 EDT [edit] Added the larger guarded real-signal trial runner by extending the existing real-signal wrapper to accept an explicit paired replay slice without changing the wrapper boundary or policy.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/real_signal_benchmark_wrapper.py, /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_prime_transport_larger_real_signal_trial.py
- 20:45:00 EDT [run] Ran the paired larger real-signal benchmark on the selected `timing_mode_eval_v1` replay artifacts and saved the guarded comparison summary.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_larger_real_signal_trial.csv
  Cmd: `python ai-router/router-research/tools/prime_transport/run_prime_transport_larger_real_signal_trial.py`
- 20:45:00 EDT [decision] The paired larger replay slice still does not produce natural route reuse for `base_gap`, but it preserves the prime-transport fallback advantage; keep `base_gap` unchanged as the first guarded real-signal policy.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_larger_real_signal_trial.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 18:50:00 EDT [edit] Implemented the first offline mock router module plus a tiny bounded demo runner under the research tools tree.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/mock_router_module.py, /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_mock_router_module_demo.py
- 18:50:00 EDT [run] Ran the mock router demo and saved the minimal demo artifact showing initialization, update, route decision, and selective promotion behavior.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_mock_router_demo.csv
  Cmd: `python ai-router/router-research/tools/prime_transport/run_mock_router_module_demo.py`
- 18:50:00 EDT [edit] Added a short implementation note documenting what was built and the bounded demo assumptions.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_mock_router_module_implementation.md
- 18:35:00 EDT [edit] Added an implementation-facing mock router module spec and compact interface table for the exact-layer routing abstraction.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_mock_router_module_spec.md, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_mock_router_interface_table.csv
- 18:35:00 EDT [decision] The smallest non-runtime prototype step is an offline module that defaults to `R_min`, exposes explicit promotion checks, and escalates only unresolved classes toward `R_full`.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_mock_router_module_spec.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 18:20:00 EDT [edit] Added a bounded offline evaluation harness for the three exact-layer routing-state candidates and a short note interpreting the resulting prototype tradeoff.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_rmin_offline_eval.py, /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_rmin_offline_eval.md
- 18:20:00 EDT [run] Ran the offline `R_static` vs `R_min` vs `R_full` evaluation and saved the summary table.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_rmin_offline_eval_summary.csv
  Cmd: `python ai-router/router-research/tools/prime_transport/run_rmin_offline_eval.py`
- 18:20:00 EDT [decision] Keep `R_min = (b, phi, r, next_return_gap)` as the first routing prototype target: it is strong enough to justify a prototype, but it should be paired with selective promotion because it remains materially weaker than full spin.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_rmin_offline_eval.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 18:05:00 EDT [edit] Added an implementation-ready exact-layer prototype spec for the recommended minimal routing state `(b, phi, r, next_return_gap)` and a compact state/update/refinement table.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_rmin_prototype_spec.md, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_rmin_prototype_table.csv
- 18:05:00 EDT [decision] Treat `R_min` as the first routing prototype target: route coarsely on chart plus immediate return-memory, and promote only unresolved classes that continue collective first-splitting.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_rmin_prototype_spec.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 17:55:00 EDT [edit] Added a routing-abstraction note that translates the current exact-layer results into three candidate routing states and a minimal routing principle based on coarse state, delayed refinement, and first-splitting promotion.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_routing_abstraction.md
- 17:55:00 EDT [edit] Added a compact three-state comparison table for static chart, minimal hybrid predictive state, and full predictive state.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_routing_state_candidates.csv
- 17:55:00 EDT [decision] Recommend the minimal hybrid state `(b, phi, r, next_return_gap)` as the first routing prototype target because it is the smallest currently justified predictive refinement of the exact chart that remains genuinely smaller than spin.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_routing_abstraction.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 17:35:00 EDT [edit] Added a minimal predictive-state note that selects the smallest currently viable exact routing state from the validated residual candidates and ties it back to the canonical threshold-law reading.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_minimal_predictive_state.md
- 17:35:00 EDT [run] Built a compact comparison table placing the chart exact address, the selected minimal candidate, stronger noncompressed return-memory candidates, and full spin on the same bounded exact rows.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_minimal_predictive_state_comparison.csv
- 17:35:00 EDT [decision] Record `next_return_gap` as the smallest currently viable exact predictive residual for routing use: it stays smaller than spin and captures substantial structure, but it remains materially weaker than full spin.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_minimal_predictive_state.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 17:15:00 EDT [edit] Added a bounded return-grammar compression runner that compares a very small exact dynamical candidate family against the spin-side target on the existing tightly matched first-splitting rows.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/analyze_return_grammar_compression.py
- 17:15:00 EDT [run] Ran the return-grammar compression comparison and saved the per-row candidate table plus the candidate score summary.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_return_grammar_comparison.csv, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_return_grammar_scores.csv
  Cmd: `python ai-router/router-research/tools/prime_transport/analyze_return_grammar_compression.py`
- 17:15:00 EDT [result] The strongest raw-capture candidates are short return-gap sequences, but they overexpand the label space beyond spin and therefore do not qualify as genuine compressions; record the residual as partially compressible but still significantly weaker than spin.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_return_grammar_compression.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 16:50:00 EDT [edit] Added a final consolidated exact-layer note summarizing the current prime transport picture across static phase-fiber-scale structure, predictive spin, visible-threshold mechanism, and the current residual beyond the static chart.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_exact_layer_consolidated.md
- 16:50:00 EDT [decision] Treat the consolidated exact-layer note as the compact landing reference for this branch: exact static chart `(b, phi, r)`, stronger predictive state `(b, spin_H)`, canonical law class `density + first-splitting event`, and residual candidate class centered on return-memory rather than local suffix memory.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_exact_layer_consolidated.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md, /Users/adminamn/AI-Research/ai-router/router-research/results/INDEX.md
- 16:35:00 EDT [edit] Added a bounded residual-analysis runner for the exact-layer question “spin minus phase-fiber-scale,” using tightly matched first-splitting rows and a small family of exact dynamical candidate labels.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/analyze_spin_phase_fiber_residual.py
- 16:35:00 EDT [run] Ran the spin-minus-phase-fiber residual analysis on the two existing tightly density-matched families and saved the residual, summary, and score tables.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_spin_minus_phase_fiber_residuals.csv, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_spin_minus_phase_fiber_summary.csv, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_spin_minus_phase_fiber_scores.csv
  Cmd: `python ai-router/router-research/tools/prime_transport/analyze_spin_phase_fiber_residual.py`
- 16:35:00 EDT [result] The strongest small exact residual object tested so far is a previous/next return-gap pair, which materially improves predictive purity over `b` alone but still captures only part of the spin partition, especially on the deeper `30030 -> 510510` family; record the residual as partially compressible but not yet reducible to a small exact object.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_spin_minus_phase_fiber_residual.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 16:10:00 EDT [edit] Added a compact canonical exact-layer note summarizing the current relationship between the static phase-fiber chart `(b, phi, r)`, finite-horizon spin, and visible first-splitting events.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_phase_fiber_vs_spin_canonical.md
- 16:10:00 EDT [decision] Record the stable exact-layer reading that phase-fiber-scale coordinates carry real first-splitting signal but do not yet replace spin as the stronger exact predictor; visible first splitting should currently be described as partially geometric and partially genuinely dynamical.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_phase_fiber_vs_spin_canonical.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md, /Users/adminamn/AI-Research/ai-router/router-research/results/INDEX.md
- 15:55:00 EDT [edit] Added a bounded exact-layer phase-fiber readability runner that maps known first-splitting parent predictive classes back into the exact phase-fiber chart `(b, phi, r)` and scores simple phase-fiber concentration features against visible threshold.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/analyze_phase_fiber_first_split.py
- 15:55:00 EDT [run] Ran the phase-fiber readability analysis on the two existing tightly density-matched families and saved the mapped class table, row-level readability summary, and predictor score table.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_phase_fiber_mapped_first_split_classes.csv, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_phase_fiber_readability_summary.csv, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_phase_fiber_readability_scores.csv
  Cmd: `python ai-router/router-research/tools/prime_transport/analyze_phase_fiber_first_split.py`
- 15:55:00 EDT [result] First visible splitting is partially readable in exact phase-fiber-scale coordinates: concentration of first splitters in dominant `phi` tuples and base phases is strong on the bounded matched-family table, but full spin-side first-splitting multiplicity remains the stronger exact predictor and no simple local phase-fiber rule is yet earned.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_phase_fiber_first_split_readability.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 04:40:00 EDT [edit] Added a fixed-family arrangement-isolation runner that holds the lift prime and local forbidden-count fixed while comparing residual visible-threshold variation against arrangement-sensitive stencil statistics.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/analyze_arrangement_isolation.py
- 04:40:00 EDT [run] Ran the exact fixed-family arrangement-isolation analysis on the `p = 13, nu_p(A) = 4` and `p = 17, nu_p(A) = 4` families and saved the focused row and score tables.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_arrangement_isolation_rows.csv, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_arrangement_isolation_scores.csv
  Cmd: `python ai-router/router-research/tools/prime_transport/analyze_arrangement_isolation.py`
- 04:40:00 EDT [result] Arrangement is now isolated as a genuine secondary driver because visible-threshold variation persists when both `p` and `nu_p(A)` are fixed, but parent admissible density still outperforms the tested arrangement statistics; `gap_entropy` and `gap_max` are the best current arrangement candidates.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_arrangement_isolation.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 04:25:00 EDT [edit] Extended the visible-threshold predictor analysis with a small arrangement-sensitive statistic family built from the forbidden-residue gap profile at the lift prime.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/analyze_visible_threshold_predictors.py, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_arrangement_stats.csv
- 04:25:00 EDT [result] Arrangement information now clearly improves on forbidden-count alone, with `gap_max` and `gap_span` the strongest tested arrangement features, but no arrangement statistic beats the lift prime as the leading coarse organizer.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_arrangement_statistics.md, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_predictor_scores.csv
- 04:10:00 EDT [edit] Extended the exact threshold family with three additional admissible size-4 tuplets chosen to collapse `nu_p(A)` at targeted tested primes while keeping raw tuplet size fixed.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_threshold_law_table.py, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/threshold_law_summary.csv
- 04:10:00 EDT [run] Computed the exact threshold extension rows for `double_twins_p13`, `double_twins_p17`, and `double_twins_p19`, merged them into the main threshold table, and reran the visible-threshold predictor analysis.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/threshold_law_summary.csv, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/threshold_law_detail.csv, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_predictor_scores.csv
  Cmd: `python ai-router/router-research/tools/prime_transport/analyze_visible_threshold_predictors.py`
- 04:10:00 EDT [result] The extension broke the earlier `|A|` versus `nu_p(A)` degeneracy: size alone is now more clearly ruled out, `p * |A|` is no longer the leading coarse organizer, and the remaining plausible law class depends on lift prime plus finer local stencil arrangement beyond forbidden-count alone.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_visible_threshold_extension.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 03:40:00 EDT [edit] Added an exact-layer visible-threshold predictor analysis runner that augments the threshold table with local stencil counts, parent admissible density, and coarse mixed-scale candidate variables.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/analyze_visible_threshold_predictors.py
- 03:40:00 EDT [run] Ran the visible-threshold predictor analysis on the current exact threshold table and saved row-level predictors, score summaries, and monotonicity checks.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_predictor_rows.csv, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_predictor_scores.csv, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/visible_threshold_monotonicity.csv
  Cmd: `python ai-router/router-research/tools/prime_transport/analyze_visible_threshold_predictors.py`
- 03:40:00 EDT [result] Current exact-table analysis rules out laws in `p` alone, `|A|` alone, or small lag, while the strongest tested coarse candidate class is a mixed prime-by-stencil scale such as `p * |A|`; record this only as a plausible law class anchor, not a formula.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_visible_threshold_predictors.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 03:25:00 EDT [edit] Added a threshold-law runner that measures first internal split, first visible split, and lag across successive wheel lifts and established tuplet patterns entirely at the exact `(b, spin_H)` layer.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_threshold_law_table.py
- 03:25:00 EDT [run] Ran the exact threshold-law table across lifts `210 -> 2310 -> 30030 -> 510510 -> 9699690` for tuplets `twins`, `triplet`, and `quadruplet`, with horizons checked up to `H = 60`.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/threshold_law_summary.csv, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/threshold_law_detail.csv
  Cmd: `python ai-router/router-research/tools/prime_transport/run_threshold_law_table.py`
- 03:25:00 EDT [result] The current exact table shows `first_internal_split_H = 1` in every tested lift/tuplet, while visible split ranges from immediate to strongly delayed; the quadruplet visible lag is `0, 20, 50, >59` along the tested lifts, so no small-lag law is supported.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_threshold_law.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 20:25:00 EDT [edit] Added a durable canonical framework note that makes the exact recursive admissibility system the primary project object and explicitly places quotient geometry in the downstream layer.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_canonical_framework.md
- 20:25:00 EDT [edit] Added a short canonical-vs-downstream boundary note to reduce future drift back toward treating quotient work as primary.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_canonical_vs_downstream.md
- 20:25:00 EDT [decision] Record the next preferred step strictly at the exact recursive-system layer: formalize and test the law for first internal split horizon, first visible split horizon, and their lag as functions of wheel lift and tuplet pattern.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_canonical_framework.md, /Users/adminamn/AI-Research/ai-router/router-research/results/INDEX.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 20:05:00 EDT [read] Re-centered the prime-transport line on the canonical recursive-system summary and explicitly dropped quotient geometry as the primary object for the current task.
  Files: /Users/adminamn/AI-Research/prime_research_summary.md
- 20:05:00 EDT [edit] Added a stable exact-system research reference covering the transport orbit, recursive affine lift, exact `(b, phi, r)` state, finite-horizon spin, compressed predictive state `(b, spin_H)`, and delayed visibility.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_exact_recursive_system.md
- 20:05:00 EDT [edit] Added a bounded exact-state experiment that measures delayed visibility and predictive-state refinement under wheel lifts entirely at the `(b, spin_H)` layer, with no quotient target.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_recursive_state_visibility.py
- 20:05:00 EDT [run] Ran the exact recursive-system visibility experiment on the `30030 -> 510510` and `510510 -> 9699690` wheel lifts for horizons `H = 1 .. 60`.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/recursive_state_visibility_detail.csv, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/recursive_state_visibility_summary.csv
  Cmd: `python ai-router/router-research/tools/prime_transport/run_recursive_state_visibility.py`
- 20:05:00 EDT [result] Confirmed that internal predictive refinement is active from `H = 1` on both tested wheel lifts, while visible predictive-state count growth is delayed; the `17`-layer lift first becomes visibly distinct at `H = 51`, and the `19`-layer lift showed no visible predictive-state count increase through `H = 60`.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_exact_recursive_system.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 19:15:00 EDT [read] Re-read the canonical prime summary and the current grouped-packet note/index/evidence files before resetting the research line around the recursive dynamical system.
  Files: /Users/adminamn/AI-Research/prime_research_summary.md, /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_grouped_packet_recovery_stage2.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md, /Users/adminamn/AI-Research/ai-router/router-research/results/INDEX.md
- 19:15:00 EDT [edit] Added a corrective spin-first research note that reorders the project around exact recursive lift, exact `(b, phi, r)` state, finite-horizon spin, and only then downstream low-dimensional quotient attempts.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_spin_first_reset.md
- 19:15:00 EDT [edit] Added a bounded spin-based bridge runner that constructs exact finite-horizon future-word states `spin_H` from the aligned datasets and compares `spin_H -> z` recovery against the saved grouped-packet baseline family results.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_spin_h_c2_bridge.py
- 19:15:00 EDT [run] Ran the first spin-first bridge comparison on the checked-in `W = 30030` and `W = 510510` aligned datasets.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_spin_bridge/spin_h_c2_bridge_detail.csv, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_spin_bridge/spin_h_c2_bridge_summary.csv
  Cmd: `python ai-router/router-research/tools/prime_transport/run_spin_h_c2_bridge.py`
- 19:15:00 EDT [result] First-pass exact `spin_H` keyed recovery slightly improved over the weakest packet baselines only marginally, remained weak in absolute terms, and did not beat the stronger within-scale packet result; record spin-first as the better-motivated bridge candidate, but not yet an empirical explanation of the `C^2` backbone.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_spin_first_reset.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 18:25:00 EDT [run] Ran the second-stage grouped-packet family comparison on the aligned `W = 30030` and `W = 510510` datasets.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_grouped_packet_family_recovery.py, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_grouped_packets/grouped_packet_family_recovery_detail.csv, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_grouped_packets/grouped_packet_family_recovery_summary.csv
  Cmd: `python ai-router/router-research/tools/prime_transport/run_grouped_packet_family_recovery.py`
- 18:25:00 EDT [result] The best tested family was `fourier4_adjacent_interactions`, which improved within-scale recovery slightly versus the baseline but still left absolute recovery poor and cross-scale transfer weak; record the grouped-packet picture as weak / currently unsupported at this simple composition level.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_grouped_packet_recovery_stage2.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 18:10:00 EDT [edit] Added a second-stage grouped-packet recovery runner comparing a small controlled family of encoders and composition rules against the aligned `C^2` target across `W = 30030` and `W = 510510`.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_grouped_packet_family_recovery.py
- 18:10:00 EDT [edit] Added a bounded stage-2 interpretation note and indexed the recovery experiment artifacts.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_grouped_packet_recovery_stage2.md, /Users/adminamn/AI-Research/ai-router/router-research/results/INDEX.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md
- 17:10:00 EDT [read] Inspected the existing local prime-transport artifacts to identify the current quotient trajectory source and the available phase/fiber summaries before building an aligned packet dataset.
  Files: /Users/adminamn/AI-Research/complex_C2_quotient_trajectory.csv, /Users/adminamn/AI-Research/phase_scale_fiber_admissible_states.csv, /Users/adminamn/AI-Research/phase_scale_fiber_summary.csv, /Users/adminamn/AI-Research/phase_fiber_generalization_summary.csv, /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/c2_shared_astar_unseen_wheel_test.py
- 17:10:00 EDT [edit] Added a dataset builder that reconstructs the current `C^2` quotient coordinates and aligns them with transport index, base phase, admissibility, and finite-depth layer phases derived from the fiber index.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/build_layer_packet_dataset.py
- 17:10:00 EDT [edit] Updated the grouped-packet scaffold to consume the aligned dataset schema directly via `phi_m_angle` and `z1/z2` columns.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/layer_packet_c2_scaffold.py
- 17:10:00 EDT [edit] Added a short schema/source-of-truth note for the aligned packet datasets and indexed the new builder.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_grouped_packets/README.md, /Users/adminamn/AI-Research/ai-router/router-research/results/INDEX.md
- 16:30:00 EDT [read] Re-read the durable prime transport note, results index, evidence summary, and prime-transport tool location before recording the next grouped-packet step.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_c2_backbone.md, /Users/adminamn/AI-Research/ai-router/router-research/results/INDEX.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md, /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/c2_shared_astar_unseen_wheel_test.py
- 16:30:00 EDT [edit] Added a bounded research note formalizing the grouped complex-packet hypothesis as a follow-on explanation test for the empirical `C^2` backbone.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_grouped_packet_hypothesis.md
- 16:30:00 EDT [edit] Added a small scaffold experiment defining a candidate layer packet, a normalized composition rule, and a simple recovery test against reference `C^2` coordinates.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/layer_packet_c2_scaffold.py
- 16:30:00 EDT [decision] Keep the grouped-packet step explicitly downstream of the layered torus-state and empirical `C^2` backbone results: packet composition is a candidate explanatory formalism, not a proved symmetry or closed-form recursion.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_grouped_packet_hypothesis.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md, /Users/adminamn/AI-Research/ai-router/router-research/results/INDEX.md
- 13:00:00 EDT [read] Located durable research-note, index, and integration-note destinations for the prime admissibility transport result inside the merged monorepo.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/SESSION_LEDGER.md, /Users/adminamn/AI-Research/ai-router/router-research/results/INDEX.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md, /Users/adminamn/AI-Research/MUDBench/docs/GEOMETRIC_ROUTED_PROMPT_ENGINE.md
- 13:00:00 EDT [edit] Added a durable router-side research note for the layered torus-bundle interpretation, the `C^2` quotient, and the shared canonical transport law `A_*`.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_c2_backbone.md
- 13:00:00 EDT [edit] Copied the unseen-wheel script and result artifacts into stable `router-research` experiment locations.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/c2_shared_astar_unseen_wheel_test.py, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_c2_backbone/c2_shared_astar_unseen_9699690_summary.csv, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_c2_backbone/c2_shared_astar_unseen_9699690_eigenvalues.csv, /Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_c2_backbone/c2_shared_astar_unseen_9699690_note.md
- 13:00:00 EDT [edit] Added a short MUDBench integration note framing the result as a low-dimensional backbone-plus-residual hint for hierarchical routing and reasoning compression, not as a production routing law.
  Files: /Users/adminamn/AI-Research/MUDBench/docs/PRIME_TRANSPORT_BACKBONE_NOTE.md, /Users/adminamn/AI-Research/MUDBench/docs/GEOMETRIC_ROUTED_PROMPT_ENGINE.md
- 13:00:00 EDT [decision] Record the prime admissibility transport result as durable supporting evidence for reusable compressed transport structure while keeping the non-claims explicit: no exact closure, no prime oracle, no production routing claim.
  Files: /Users/adminamn/AI-Research/ai-router/router-research/docs/research/prime_transport_c2_backbone.md, /Users/adminamn/AI-Research/ai-router/router-research/EVIDENCE_SUMMARY.md, /Users/adminamn/AI-Research/ai-router/router-research/results/INDEX.md

Recovery rule:
- on resume after compaction or interruption, read this file after
  `docs/SESSION_BOOTSTRAP.md`
- trust the most recent entries only if they are consistent with
  `docs/PROJECT_CONTEXT.md` and `docs/routes/ROUTE_MATRIX.md`

## 2026-03-11
- 09:42:06 EDT [checkpoint] Recovery work started after context drift and stale increment anchoring
- 09:42:06 EDT [next] Resume from RR-061 with stronger angular-law-sensitive diagnostics instead of continuing INC-0065 immediately
- 09:42:06 EDT [read] Re-read startup source-of-truth docs and branch-control docs
  Files: /Users/adminamn/ai-router/router-research/docs/SESSION_BOOTSTRAP.md, /Users/adminamn/ai-router/router-research/docs/PROJECT_CONTEXT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/research/MECHANISM_FIRST_PLAN.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0060_h4_hopf_measure_diagnostics.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0061_measure_consistent_route_law.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0062_hopf_base_angular_law.md, /Users/adminamn/ai-router/router-research/docs/program/PROJECT_BOARD.md
- 09:42:06 EDT [decision] RR-061 still appears to be the correct active gate; INC-0062 did not close Hopf-angular correctness, so later phase-first work was likely off-sequence
- 09:43:38 EDT [decision] User requested that INC-0062, INC-0063, and INC-0064 be treated as falsified by bad data and redone before returning to INC-0065
- 09:53:20 EDT [checkpoint] Confirmed RR-061 remains active on the project board; inspecting partial transport-complex code before redoing INC-0062/0063/0064.
- 10:03:49 EDT [edit] Completed router/proxy plumbing recovery for INC-0063/0064: live alpha bins, complex-coupled transport metrics, route-coordinate diagnostics, and stale audit-test cleanup.
  Files: /Users/adminamn/ai-router/router-research/hyperbolic_router_so8.py, /Users/adminamn/ai-router/router-research/tasks/router_proxy_eval.py, /Users/adminamn/ai-router/router-research/tests/test_cli_contract.py, /Users/adminamn/ai-router/router-research/tests/test_sector_modes.py, /Users/adminamn/ai-router/router-research/tests/test_measure_diagnostics.py
- 10:08:54 EDT [edit] Added corrected INC-0062/0063/0064 sweep configs and a generic proxy address-diff helper for route-key comparison on the eval subset.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0062_hopf_base_screen_corrected.json, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0062_hopf_base_confirm_corrected.json, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0063_phase_transport_screen_corrected.json, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0064_coupled_complex_phase_screen.json, /Users/adminamn/ai-router/router-research/tools/proxy_address_diff.py
- 10:10:23 EDT [result] Completed corrected INC-0062 screen sweep. Reviewing route stats and new Hopf-sector diagnostics before deciding whether to launch corrected confirm.
- 10:12:22 EDT [result] Completed corrected INC-0062 confirm sweep. The reopened angular gate now has explicit within-sector Hopf diagnostics across four seeds.
- 10:13:53 EDT [result] Completed corrected INC-0063 screen sweep; now checking whether transported phase is still inert after restoring live alpha bins.
- 10:14:09 EDT [edit] Fixed proxy_address_diff import path so the route-key audit can run from tools/ without manual PYTHONPATH setup.
  Files: /Users/adminamn/ai-router/router-research/tools/proxy_address_diff.py
- 10:16:18 EDT [result] Completed corrected INC-0064 screen sweep; running route-key diff against HOPF_BASE_K25_PHI to measure explicit complex-field coupling movement.
- 10:19:52 EDT [read] Reloaded canonical context before editing dynamic research docs.
  Files: /Users/adminamn/ai-router/router-research/docs/SESSION_BOOTSTRAP.md, /Users/adminamn/ai-router/router-research/docs/PROJECT_CONTEXT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md
- 10:20:06 EDT [read] Checked program board and issue registry before rewriting live research docs so corrected 62/63/64 conclusions stay consistent with the higher-level queue.
  Files: /Users/adminamn/ai-router/router-research/docs/program/PROJECT_BOARD.md, /Users/adminamn/ai-router/router-research/docs/program/ISSUE_REGISTRY.md, /Users/adminamn/ai-router/router-research/docs/PLAN.md
- 10:20:48 EDT [checkpoint] Found queue drift between route matrix and program board; checking RR/INC numbering before reconciling corrected 62/63/64 docs.
  Files: /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/program/PROJECT_BOARD.md, /Users/adminamn/ai-router/router-research/docs/program/ISSUE_REGISTRY.md
- 10:21:17 EDT [read] Checked worktree before doc reconciliation to keep the corrected rerun narrative in one bounded edit pass.
  Cmd: `git -C /Users/adminamn/ai-router/router-research status --short`
- 10:28:04 EDT [edit] Reconciled canonical increment, handoff, route-matrix, board, checkpoint, and results-index docs to the corrected INC-0062/63/64 reruns and queued INC-0065 as the next branch.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0062_hopf_base_angular_law.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0063_phase_transport_necessity.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0064_coupled_complex_phase_transport.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0065_product_phase_field.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/research/RECOVERY_CHECKPOINT_20260311.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/program/PROJECT_BOARD.md, /Users/adminamn/ai-router/router-research/docs/program/ISSUE_REGISTRY.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 10:29:06 EDT [decision] Corrected 62/63/64 reruns are now canonical across the bootstrap, theory, handoff, and queue docs; INC-0065/RR-065 is the next branch.
  Files: /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/research/LEARNED_KNOWLEDGE.md, /Users/adminamn/ai-router/router-research/docs/research/OPEN_QUESTIONS.md, /Users/adminamn/ai-router/router-research/docs/research/PROGRESS_TRACE.md, /Users/adminamn/ai-router/router-research/docs/research/MATH_CONTRACT_H4XH4_PHASE_SPECTRAL_20260310.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/program/PROJECT_BOARD.md, /Users/adminamn/ai-router/router-research/docs/program/ISSUE_REGISTRY.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 10:29:30 EDT [checkpoint] Readback and stale-context sweep completed after reconciling the corrected 62/63/64 reruns into the canonical docs.
  Files: /Users/adminamn/ai-router/router-research/docs/research/PROGRESS_TRACE.md, /Users/adminamn/ai-router/router-research/docs/research/LEARNED_KNOWLEDGE.md, /Users/adminamn/ai-router/router-research/docs/research/OPEN_QUESTIONS.md, /Users/adminamn/ai-router/router-research/docs/research/MATH_CONTRACT_H4XH4_PHASE_SPECTRAL_20260310.md
- 10:29:50 EDT [result] Verified doc reconciliation pass for corrected INC-0062/63/64 and queued INC-0065 with git diff --check on the touched docs.
  Cmd: `git diff --check on corrected increment, handoff, theory, queue, and index docs`
- 10:33:04 EDT [read] Reloaded INC-0065 branch doc and inspected router/task/test/tool surfaces for existing product phase-field support before making code changes.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0065_product_phase_field.md, /Users/adminamn/ai-router/router-research/hyperbolic_router_so8.py, /Users/adminamn/ai-router/router-research/tasks/router_proxy_eval.py, /Users/adminamn/ai-router/router-research/tests
- 10:55:37 EDT [result] Completed INC-0065 product phase-field screen from tracked config; all product variants passed health gate, field-shift metrics were nonzero, H4XH4_FIELD_A150 had the best screen MSE, H4XH4_FIELD_A100 was the sweep recommendation, and seed-0 address audit showed about 98.4%-98.6% sector difference vs HOPF_BASE_K25_PHI.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0065_product_phase_field_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0065_product_phase_field_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0065_product_phase_field_address_diff.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_105034.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 10:58:01 EDT [next] Run the tracked INC-0065 confirm slice with H4XH4_FIELD_A100 and H4XH4_FIELD_A150 against the routed controls and R0, then evaluate whether the positive product screen survives 4 seeds.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0065_product_phase_field_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0065_product_phase_field_screen.json
- 11:14:11 EDT [read] Re-read project context, route matrix, confirm handoff docs, and spectral theory files before starting INC-0066.
  Files: /Users/adminamn/ai-router/router-research/docs/PROJECT_CONTEXT.md, /Users/adminamn/ai-router/router-research/docs/PLAN.md, /Users/adminamn/ai-router/router-research/docs/IMPLEMENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0065_product_phase_field.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0066_spectral_route_operator.md, /Users/adminamn/ai-router/router-research/spectral_emergence_moonshot.md, /Users/adminamn/ai-router/router-research/minimal_theorem_for_spectral_emergence.md, /Users/adminamn/ai-router/router-research/GEOMETRIC_COMPUTATION_HYPOTHESIS.md
- 11:14:11 EDT [result] Spectral sweep tooling smoke checks passed.
  Files: /Users/adminamn/ai-router/router-research/tools/spectral_route_sweep.py, /Users/adminamn/ai-router/router-research/tests/test_spectral_route_sweep.py, /Users/adminamn/ai-router/router-research/tests/test_spectral_route_audit.py
  Cmd: `python -m py_compile tools/spectral_route_sweep.py tests/test_spectral_route_sweep.py && python -m unittest -v tests/test_spectral_route_sweep.py tests/test_spectral_route_audit.py`
- 11:14:11 EDT [edit] Added reproducible INC-0066 spectral sweep tooling, config, and regression tests.
  Files: /Users/adminamn/ai-router/router-research/tools/spectral_route_sweep.py, /Users/adminamn/ai-router/router-research/tests/test_spectral_route_sweep.py, /Users/adminamn/ai-router/router-research/configs/spectral_route_inc0066_screen.json
- 11:26:25 EDT [result] INC-0066 confirm closed positive: the spectral distinction survived four seeds, with product routes retaining higher participation and lower sector lowfreq concentration than the control set.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0066_spectral_route_operator_confirm.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_112215.md
- 11:26:25 EDT [result] INC-0066 screen closed positive: all graphs stayed connected and the product routes showed higher low-mode participation with lower sector lowfreq concentration than the control set.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0066_spectral_route_operator_screen.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_112010.md
- 11:26:25 EDT [run] Ran the tracked INC-0066 spectral screen on the confirmed route set.
  Files: /Users/adminamn/ai-router/router-research/configs/spectral_route_inc0066_screen.json, /Users/adminamn/ai-router/router-research/tools/spectral_route_sweep.py
  Cmd: `python tools/spectral_route_sweep.py --config configs/spectral_route_inc0066_screen.json`
- 11:26:25 EDT [run] Ran the tracked INC-0066 spectral confirm on the fixed confirmed route set.
  Files: /Users/adminamn/ai-router/router-research/configs/spectral_route_inc0066_confirm.json, /Users/adminamn/ai-router/router-research/tools/spectral_route_sweep.py
  Cmd: `python tools/spectral_route_sweep.py --config configs/spectral_route_inc0066_confirm.json`
- 11:26:25 EDT [edit] Updated canonical docs and queued INC-0067 after closing INC-0066 positive.
  Files: /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0066_spectral_route_operator.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0067_spectral_signal_probes.md, /Users/adminamn/ai-router/router-research/docs/research/LEARNED_KNOWLEDGE.md, /Users/adminamn/ai-router/router-research/docs/research/PROGRESS_TRACE.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 11:26:25 EDT [result] Final verification passed for the INC-0066 pass: full unit suite green and targeted diff check clean on the touched files.
  Files: /Users/adminamn/ai-router/router-research/tools/spectral_route_sweep.py, /Users/adminamn/ai-router/router-research/tests/test_spectral_route_sweep.py, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
  Cmd: `python -m unittest discover -s tests -v && git diff --check -- [touched files]`
- 11:35:54 EDT [edit] Added INC-0067 spectral signal-probe tooling, config, and tests on top of the confirmed operator branch.
  Files: /Users/adminamn/ai-router/router-research/tools/spectral_signal_probe.py, /Users/adminamn/ai-router/router-research/tools/spectral_signal_sweep.py, /Users/adminamn/ai-router/router-research/configs/spectral_signal_inc0067_screen.json, /Users/adminamn/ai-router/router-research/tests/test_spectral_signal_sweep.py, /Users/adminamn/ai-router/router-research/tests/test_spectral_route_audit.py
- 11:35:54 EDT [result] Targeted spectral-signal tests passed before the tracked INC-0067 run.
  Files: /Users/adminamn/ai-router/router-research/tools/spectral_signal_probe.py, /Users/adminamn/ai-router/router-research/tools/spectral_signal_sweep.py, /Users/adminamn/ai-router/router-research/tests/test_spectral_signal_sweep.py, /Users/adminamn/ai-router/router-research/tests/test_spectral_route_audit.py
  Cmd: `python -m py_compile tools/spectral_signal_probe.py tools/spectral_signal_sweep.py tests/test_spectral_signal_sweep.py tests/test_spectral_route_audit.py && python -m unittest -v tests/test_spectral_route_audit.py tests/test_spectral_signal_sweep.py`
- 11:45:21 EDT [run] Ran the tracked INC-0067 direct label-probe screen on the fixed confirmed operator set.
  Files: /Users/adminamn/ai-router/router-research/configs/spectral_signal_inc0067_screen.json, /Users/adminamn/ai-router/router-research/tools/spectral_signal_sweep.py
  Cmd: `python tools/spectral_signal_sweep.py --config configs/spectral_signal_inc0067_screen.json`
- 11:45:21 EDT [result] INC-0067 screen stayed connected but direct one-hot label probes were slightly negative versus the Hopf controls, so the branch was not closed positive from the screen.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0067_spectral_signal_probes_screen.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_113729.md
- 11:45:21 EDT [run] Ran the tracked INC-0067 direct label-probe confirm on the fixed confirmed operator set.
  Files: /Users/adminamn/ai-router/router-research/configs/spectral_signal_inc0067_confirm.json, /Users/adminamn/ai-router/router-research/tools/spectral_signal_sweep.py
  Cmd: `python tools/spectral_signal_sweep.py --config configs/spectral_signal_inc0067_confirm.json`
- 11:45:21 EDT [result] INC-0067 confirm stayed connected but remained slightly negative on raw label lowfreq and Dirichlet gaps versus the Hopf controls, so the branch was closed inconclusive/negative for direct label probes.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0067_spectral_signal_probes_confirm.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_114258.md
- 11:45:21 EDT [result] Final verification passed for the INC-0067 pass: full unit suite green and targeted diff check clean on the touched files.
  Files: /Users/adminamn/ai-router/router-research/tools/spectral_signal_probe.py, /Users/adminamn/ai-router/router-research/tools/spectral_signal_sweep.py, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
  Cmd: `python -m unittest discover -s tests -v && git diff --check -- [touched files]`
- 11:45:21 EDT [edit] Updated canonical docs to close INC-0067 inconclusive/negative and queued INC-0068 for residual/task-error probes.
  Files: /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0067_spectral_signal_probes.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0068_spectral_residual_task_signals.md, /Users/adminamn/ai-router/router-research/docs/research/LEARNED_KNOWLEDGE.md, /Users/adminamn/ai-router/router-research/docs/research/PROGRESS_TRACE.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 12:27:48 EDT [edit] Added INC-0068 routed residual/task-error probe tooling, config, and regression tests on top of the confirmed operator branch.
  Files: /Users/adminamn/ai-router/router-research/tools/spectral_route_audit.py, /Users/adminamn/ai-router/router-research/tools/spectral_residual_probe.py, /Users/adminamn/ai-router/router-research/tools/spectral_residual_sweep.py, /Users/adminamn/ai-router/router-research/configs/spectral_residual_inc0068_screen.json, /Users/adminamn/ai-router/router-research/configs/spectral_residual_inc0068_confirm.json, /Users/adminamn/ai-router/router-research/tests/test_spectral_residual_probe.py, /Users/adminamn/ai-router/router-research/tests/test_spectral_residual_sweep.py
- 12:27:53 EDT [run] Ran the tracked INC-0068 residual/task-error screen on the fixed confirmed operator set.
  Files: /Users/adminamn/ai-router/router-research/configs/spectral_residual_inc0068_screen.json, /Users/adminamn/ai-router/router-research/tools/spectral_residual_sweep.py, /Users/adminamn/ai-router/router-research/results/analysis/inc0068_spectral_residual_task_signals_screen.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_122236.md
  Cmd: `python tools/spectral_residual_sweep.py --config configs/spectral_residual_inc0068_screen.json`
- 12:27:58 EDT [decision] Closed INC-0068 inconclusive/negative at screen stage because residual norm, error-indicator, and true-margin probes all stayed negative versus the Hopf controls despite the confirmed operator remaining connected and distinct.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0068_spectral_residual_task_signals_screen.json, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0068_spectral_residual_task_signals.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 12:28:02 EDT [next] Queue INC-0069 next: translate the confirmed product phase-field branch into the routed retrieval harness instead of replaying more proxy-spectral signal probes.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0069_product_phase_translation_eval.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/program/ISSUE_REGISTRY.md, /Users/adminamn/ai-router/router-research/docs/program/PROJECT_BOARD.md
- 12:29:33 EDT [read] Reloaded the INC-0069 increment doc plus retrieval harness, old translation configs, and retrieval tests before touching code.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0069_product_phase_translation_eval.md, /Users/adminamn/ai-router/router-research/tasks/router_retrieval_eval.py, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0056_product_complex_translation_screen.json, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0052_retrieval_amortization_confirm.json, /Users/adminamn/ai-router/router-research/tests/test_router_retrieval_eval.py, /Users/adminamn/ai-router/router-research/tests/test_cli_contract.py
- 12:31:47 EDT [edit] Brought the routed retrieval harness to phase-field parity and added CLI regression coverage.
  Files: /Users/adminamn/ai-router/router-research/tasks/router_retrieval_eval.py, /Users/adminamn/ai-router/router-research/tests/test_router_retrieval_eval.py, /Users/adminamn/ai-router/router-research/tests/test_cli_contract.py
- 12:32:48 EDT [result] Targeted retrieval parity tests passed before the INC-0069 screen run.
  Cmd: `python -m unittest tests.test_router_retrieval_eval tests.test_cli_contract -v`
- 12:40:11 EDT [run] Ran the tracked INC-0069 translated retrieval screen on the fixed confirmed product route set.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0069_product_phase_translation_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0069_product_phase_translation_screen.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_124011.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0069_product_phase_translation_screen.json`
- 12:40:11 EDT [decision] INC-0069 screen was positive/narrow enough to justify confirm: the product routes preserved translated pruning signal and one route improved top-1 vs HOPF_K25 while the other improved runtime.
- 12:46:16 EDT [run] Ran the tracked INC-0069 translated retrieval confirm on the fixed confirmed product route set.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0069_product_phase_translation_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0069_product_phase_translation_confirm.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_124616.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0069_product_phase_translation_confirm.json`
- 12:46:16 EDT [decision] Closed INC-0069 confirm positive/narrow as translated retrieval evidence and queued INC-0070 next to refine only the retrieval layer on top of the fixed product routes.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0069_product_phase_translation_eval.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0070_product_phase_translation_rescue.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 12:55:18 EDT [edit] Added low-margin selective translated reranking on the fixed product routes and regression coverage for the new retrieval surface.
  Files: /Users/adminamn/ai-router/router-research/tasks/router_retrieval_eval.py, /Users/adminamn/ai-router/router-research/tests/test_router_retrieval_eval.py, /Users/adminamn/ai-router/router-research/tests/test_cli_contract.py, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0070_product_phase_translation_rescue_screen.json
- 12:55:49 EDT [result] Targeted retrieval tests passed after adding the INC-0070 low-margin rerank surface.
  Cmd: `python -m unittest tests.test_router_retrieval_eval tests.test_cli_contract -v`
- 13:02:26 EDT [run] Ran the tracked INC-0070 translated retrieval rescue screen on the fixed confirmed product route set.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0070_product_phase_translation_rescue_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0070_product_phase_translation_rescue_screen.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_130226.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0070_product_phase_translation_rescue_screen.json`
- 13:02:26 EDT [decision] Closed INC-0070 negative at screen stage because low-margin reranking did not beat the fixed product baselines and gave back too much translated runtime.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0070_product_phase_translation_rescue_screen.json, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0070_product_phase_translation_rescue.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 13:05:27 EDT [result] Full unit suite passed after the INC-0070 retrieval-surface change.
  Cmd: `python -m unittest discover -s /Users/adminamn/ai-router/router-research/tests -v`
- 13:05:27 EDT [next] Queued INC-0071 next: translated secondary-key screening on the fixed product routes using the existing `hopf_plus_complex` harness support.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0071_product_phase_translation_secondary_keys.md, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0071_product_phase_translation_secondary_keys_screen.json
- 13:14:50 EDT [run] Ran the tracked INC-0071 translated secondary-key screen on the fixed confirmed product route set.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0071_product_phase_translation_secondary_keys_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0071_product_phase_translation_secondary_keys_screen.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_131450.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0071_product_phase_translation_secondary_keys_screen.json`
- 13:15:36 EDT [decision] INC-0071 screen was positive enough to justify confirm because H4XH4_FIELD_A150_CPX8 materially improved top-1 and pruning on the fixed product route law while staying healthy.
- 13:16:35 EDT [edit] Added the tracked INC-0071 confirm config carrying only the fixed references plus H4XH4_FIELD_A150_CPX8.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0071_product_phase_translation_secondary_keys_confirm.json
- 13:21:41 EDT [run] Ran the tracked INC-0071 translated secondary-key confirm on the fixed confirmed product route set.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0071_product_phase_translation_secondary_keys_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0071_product_phase_translation_secondary_keys_confirm.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_132141.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0071_product_phase_translation_secondary_keys_confirm.json`
- 13:21:41 EDT [decision] Closed INC-0071 confirm positive/narrow: the second H^4 held up as a translated discrete addressing field, but the confirmed pruning gain still did not cash out into a runtime win against the main Hopf translated control.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0071_product_phase_translation_secondary_keys.md, /Users/adminamn/ai-router/router-research/results/analysis/inc0071_product_phase_translation_secondary_keys_confirm.json, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 13:21:41 EDT [next] Queued INC-0072 next: keep the confirmed route/key law fixed and rescue only the translated secondary-key systems cost path.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0072_product_phase_translation_secondary_key_cost_rescue.md
- 13:32:15 EDT [edit] Implemented the INC-0072 systems-only rescue in the translated retrieval harness: prepared same-bucket retrieval plans and eval-route reuse across identical query repeats.
  Files: /Users/adminamn/ai-router/router-research/tasks/router_retrieval_eval.py, /Users/adminamn/ai-router/router-research/tests/test_router_retrieval_eval.py
- 13:32:38 EDT [result] Retrieval regression tests passed after the INC-0072 systems rescue change.
  Cmd: `python -m unittest /Users/adminamn/ai-router/router-research/tests/test_router_retrieval_eval.py /Users/adminamn/ai-router/router-research/tests/test_cli_contract.py -v`
- 13:33:56 EDT [run] Ran the tracked INC-0072 translated secondary-key cost-rescue screen on the fixed confirmed product route set.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0072_product_phase_translation_secondary_key_cost_rescue_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0072_product_phase_translation_secondary_key_cost_rescue_screen.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_133356.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0072_product_phase_translation_secondary_key_cost_rescue_screen.json`
- 13:34:14 EDT [decision] INC-0072 screen was positive enough to justify confirm because H4XH4_FIELD_A150_CPX8 now beat HOPF_K25_BASE_PHI on top-1, pruning, online time, and amortized time.
- 13:34:36 EDT [edit] Added the tracked INC-0072 confirm config carrying the fixed translated references and H4XH4_FIELD_A150_CPX8.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0072_product_phase_translation_secondary_key_cost_rescue_confirm.json
- 13:37:31 EDT [run] Ran the tracked INC-0072 translated secondary-key cost-rescue confirm on the fixed confirmed product route set.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0072_product_phase_translation_secondary_key_cost_rescue_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0072_product_phase_translation_secondary_key_cost_rescue_confirm.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_133731.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0072_product_phase_translation_secondary_key_cost_rescue_confirm.json`
- 13:37:31 EDT [decision] Closed INC-0072 confirm positive: the fixed product route/key law is now also a translated systems win, with H4XH4_FIELD_A150_CPX8 beating HOPF_K25_BASE_PHI on top-1, pruning, and translated cost.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0072_product_phase_translation_secondary_key_cost_rescue.md, /Users/adminamn/ai-router/router-research/results/analysis/inc0072_product_phase_translation_secondary_key_cost_rescue_confirm.json, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 13:38:12 EDT [result] Full unit suite passed after the INC-0072 systems rescue.
  Cmd: `python -m unittest discover -s /Users/adminamn/ai-router/router-research/tests -v`
- 13:38:12 EDT [next] Queued INC-0073 next: harden the confirmed translated systems lead under larger load before making stronger operational claims.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0073_product_phase_translation_secondary_key_large_load.md
- 14:20:57 EDT [run] Ran the tracked INC-0074 dense-frontier screen on the fixed INC-0071/72/73 law.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0074_product_phase_translation_dense_frontier_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0074_product_phase_translation_dense_frontier_screen.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_140824.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0074_product_phase_translation_dense_frontier_screen.json`
- 14:20:57 EDT [decision] INC-0074 screen was positive enough to justify confirm because H4XH4_FIELD_A150_CPX8 beat dense on candidate fraction plus online/amortized cost and screened slightly above dense on top-1.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0074_product_phase_translation_dense_frontier_screen.json, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0074_product_phase_translation_dense_frontier.md
- 14:20:57 EDT [run] Ran the tracked INC-0074 dense-frontier confirm on the fixed INC-0071/72/73 law.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0074_product_phase_translation_dense_frontier_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0074_product_phase_translation_dense_frontier_confirm.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_141705.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0074_product_phase_translation_dense_frontier_confirm.json`
- 14:20:57 EDT [decision] Closed INC-0074 confirm positive/narrow: the fixed product phase-field branch is now directly positive against dense exact retrieval, with H4XH4_FIELD_A150_CPX8 as the dense-frontier systems lead and INC-0075 queued for bounded quality recovery.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0074_product_phase_translation_dense_frontier.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0075_product_phase_translation_dense_quality_recovery.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 14:20:57 EDT [result] Full unit suite passed after the retrieval-aware proxy-sweep recommendation fix and INC-0074 doc reconciliation.
  Cmd: `python -m unittest discover -s /Users/adminamn/ai-router/router-research/tests -v`
- 15:18:33 EDT [run] Ran the tracked INC-0075 dense-frontier quality-recovery screen on the fixed dense-frontier law.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0075_product_phase_translation_dense_quality_recovery_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0075_product_phase_translation_dense_quality_recovery_screen.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_142843.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0075_product_phase_translation_dense_quality_recovery_screen.json`
- 15:18:33 EDT [decision] Closed INC-0075 confirm negative: bounded rerank quality rescue did not improve the fixed dense-frontier systems lead, so the next branch should be break-even mapping instead of more retrieval rescue.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0075_product_phase_translation_dense_quality_recovery.md, /Users/adminamn/ai-router/router-research/results/analysis/inc0075_product_phase_translation_dense_quality_recovery_confirm.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_144445.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 15:18:33 EDT [next] Queued INC-0076 next: keep the fixed product frontier unchanged and measure repeated-query break-even against dense exact retrieval.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0076_product_phase_translation_break_even.md, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0076_product_phase_translation_break_even_screen.json
- 15:18:33 EDT [run] Ran the tracked INC-0076 break-even screen on repeat counts Q01/Q08/Q16/Q24/Q32 for dense, the quality-matched routed point, and the secondary-key systems lead.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0076_product_phase_translation_break_even_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0076_product_phase_translation_break_even_screen.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_145722.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0076_product_phase_translation_break_even_screen.json`
- 15:18:33 EDT [decision] INC-0076 screen justified confirm because no routed crossover survived at Q08, while both routed points crossed dense on amortized cost by Q16 and widened again at Q24/Q32.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0076_product_phase_translation_break_even_screen.json, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0076_product_phase_translation_break_even.md
- 15:18:33 EDT [run] Ran the tracked INC-0076 break-even confirm on the fixed Q08/Q16/Q24 crossover bracket.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0076_product_phase_translation_break_even_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0076_product_phase_translation_break_even_confirm.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_151013.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0076_product_phase_translation_break_even_confirm.json`
- 15:18:33 EDT [decision] Closed INC-0076 confirm positive/narrow: the fixed product branch now has a confirmed amortized crossover against dense exact retrieval by Q16, with H4XH4_FIELD_A150_Q16 as the quality-matched break-even point and H4XH4_FIELD_A150_CPX8_Q16 as the first stronger-pruning systems crossover point.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0076_product_phase_translation_break_even.md, /Users/adminamn/ai-router/router-research/results/analysis/inc0076_product_phase_translation_break_even_confirm.json, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 15:18:33 EDT [next] Queued INC-0077 next: profile hardware-facing cost ratios and workload-scale behavior on the fixed Q16/Q24 crossover points instead of reopening geometry or retrieval rescue.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0077_product_phase_translation_hardware_profile.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/reports/REAL_TASK_COMPARISON.md
- 15:26:32 EDT [edit] Added the translated hardware-profile summarizer, tracked `INC-0077` configs, and regression coverage for hardware-facing derived metrics on the fixed crossover family.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_hardware_profile.py, /Users/adminamn/ai-router/router-research/tests/test_translated_hardware_profile.py, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0077_product_phase_translation_hardware_profile_screen.json, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0077_product_phase_translation_hardware_profile_confirm.json
- 15:26:32 EDT [result] Targeted `INC-0077` profile tests passed.
  Cmd: `python -m py_compile tools/translated_hardware_profile.py tests/test_translated_hardware_profile.py && python -m unittest tests.test_translated_hardware_profile tests.test_proxy_sweep -v`
- 15:26:32 EDT [run] Ran the tracked `INC-0077` smaller-bank hardware-profile screen on the fixed `Q16/Q24` crossover set.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0077_product_phase_translation_hardware_profile_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0077_product_phase_translation_hardware_profile_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0077_product_phase_translation_hardware_profile_screen_profile.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_152632.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0077_product_phase_translation_hardware_profile_screen.json && python tools/translated_hardware_profile.py --baseline results/analysis/inc0076_product_phase_translation_break_even_confirm.json --analysis results/analysis/inc0077_product_phase_translation_hardware_profile_screen.json --output results/analysis/inc0077_product_phase_translation_hardware_profile_screen_profile.json --report docs/reports/INC0077_PRODUCT_PHASE_TRANSLATION_HARDWARE_PROFILE_SCREEN.md`
- 15:26:32 EDT [decision] `INC-0077` screen justified confirm because the smaller-bank profile preserved stable work-ratio reduction, lost crossover at `Q16`, and retained crossover at `Q24`.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0077_product_phase_translation_hardware_profile_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0077_product_phase_translation_hardware_profile_screen_profile.json, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0077_product_phase_translation_hardware_profile.md
- 15:30:47 EDT [run] Ran the tracked `INC-0077` smaller-bank hardware-profile confirm on the fixed `Q16/Q24` crossover set.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0077_product_phase_translation_hardware_profile_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0077_product_phase_translation_hardware_profile_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0077_product_phase_translation_hardware_profile_confirm_profile.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_153047.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0077_product_phase_translation_hardware_profile_confirm.json && python tools/translated_hardware_profile.py --baseline results/analysis/inc0076_product_phase_translation_break_even_confirm.json --analysis results/analysis/inc0077_product_phase_translation_hardware_profile_confirm.json --output results/analysis/inc0077_product_phase_translation_hardware_profile_confirm_profile.json --report docs/reports/INC0077_PRODUCT_PHASE_TRANSLATION_HARDWARE_PROFILE_CONFIRM.md`
- 15:30:47 EDT [decision] Closed `INC-0077` confirm positive/narrow: the first explicit hardware-cost profile survived, the search-work ratio stayed stable across bank size, and `H4XH4_FIELD_A150_CPX8_Q24_T6000` became the first confirmed smaller-bank crossover point.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0077_product_phase_translation_hardware_profile.md, /Users/adminamn/ai-router/router-research/results/analysis/inc0077_product_phase_translation_hardware_profile_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0077_product_phase_translation_hardware_profile_confirm_profile.json, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 15:30:47 EDT [next] Queued `INC-0078` next: map the crossover boundary over bank size and repeat count on the fixed route/key law instead of reopening geometry or retrieval rescue.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0078_product_phase_translation_crossover_map.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 15:38:17 EDT [edit] Extended the translated hardware-profile tool with bank-level crossover summaries and added regression coverage for first-crossover and ratio-stability reporting.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_hardware_profile.py, /Users/adminamn/ai-router/router-research/tests/test_translated_hardware_profile.py
- 15:38:17 EDT [result] Targeted translated-hardware-profile tests passed after the bank-summary extension.
  Cmd: `python -m py_compile tools/translated_hardware_profile.py tests/test_translated_hardware_profile.py && python -m unittest tests.test_translated_hardware_profile -v`
- 15:56:44 EDT [run] Ran the tracked `INC-0078` crossover-map screen on the fixed translated product family across `max_train={3000,6000,12000}` and `Q={12,16,20,24}`.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0078_product_phase_translation_crossover_map_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0078_product_phase_translation_crossover_map_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0078_product_phase_translation_crossover_map_screen_profile.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_155644.md, /Users/adminamn/ai-router/router-research/docs/reports/INC0078_PRODUCT_PHASE_TRANSLATION_CROSSOVER_MAP_SCREEN.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0078_product_phase_translation_crossover_map_screen.json && python tools/translated_hardware_profile.py --analysis results/analysis/inc0078_product_phase_translation_crossover_map_screen.json --output-json results/analysis/inc0078_product_phase_translation_crossover_map_screen_profile.json --output-md docs/reports/INC0078_PRODUCT_PHASE_TRANSLATION_CROSSOVER_MAP_SCREEN.md`
- 15:57:03 EDT [decision] `INC-0078` screen justified confirm because the boundary was already coherent: no crossover through `Q24` at `3000`, onset at `Q24` for `6000`, and onset at `Q12` for `12000` with stable secondary-key work ratio.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0078_product_phase_translation_crossover_map_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0078_product_phase_translation_crossover_map_screen_profile.json, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0078_product_phase_translation_crossover_map.md, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0078_product_phase_translation_crossover_map_confirm.json
- 16:11:19 EDT [run] Ran the tracked `INC-0078` crossover-map confirm on the narrowed bank-by-bank boundary bracket.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0078_product_phase_translation_crossover_map_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0078_product_phase_translation_crossover_map_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0078_product_phase_translation_crossover_map_confirm_profile.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_161119.md, /Users/adminamn/ai-router/router-research/docs/reports/INC0078_PRODUCT_PHASE_TRANSLATION_CROSSOVER_MAP_CONFIRM.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0078_product_phase_translation_crossover_map_confirm.json && python tools/translated_hardware_profile.py --analysis results/analysis/inc0078_product_phase_translation_crossover_map_confirm.json --output-json results/analysis/inc0078_product_phase_translation_crossover_map_confirm_profile.json --output-md docs/reports/INC0078_PRODUCT_PHASE_TRANSLATION_CROSSOVER_MAP_CONFIRM.md`
- 16:11:19 EDT [decision] Closed `INC-0078` confirm positive/narrow: the translated product systems branch now has a confirmed monotone crossover boundary with no crossover through `Q24` at `3000`, first crossover at `Q24` for `6000`, and first crossover at `Q12` for `12000`.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0078_product_phase_translation_crossover_map.md, /Users/adminamn/ai-router/router-research/results/analysis/inc0078_product_phase_translation_crossover_map_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0078_product_phase_translation_crossover_map_confirm_profile.json, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 16:11:19 EDT [next] Queued `INC-0079` next: extend the fixed crossover-boundary map upward to larger banks rather than reopening geometry or retrieval rescue.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0079_product_phase_translation_large_bank_boundary_extension.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 22:25:01 EDT [run] Ran the tracked `INC-0079` larger-bank boundary screen on the fixed translated systems family at `max_train={12000,18000}` and `Q={8,12,16}`.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0079_product_phase_translation_large_bank_boundary_extension_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_screen_profile.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_222501.md, /Users/adminamn/ai-router/router-research/docs/reports/INC0079_PRODUCT_PHASE_TRANSLATION_LARGE_BANK_BOUNDARY_EXTENSION_SCREEN.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0079_product_phase_translation_large_bank_boundary_extension_screen.json && python tools/translated_hardware_profile.py --analysis results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_screen.json --output-json results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_screen_profile.json --output-md docs/reports/INC0079_PRODUCT_PHASE_TRANSLATION_LARGE_BANK_BOUNDARY_EXTENSION_SCREEN.md`
- 22:25:32 EDT [decision] `INC-0079` screen justified confirm because the onset map was already coherent: `T12000` stayed at `Q12` while `T18000` crossed by `Q08`, with a stable secondary-key work ratio near `0.19`.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_screen_profile.json, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0079_product_phase_translation_large_bank_boundary_extension.md, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0079_product_phase_translation_large_bank_boundary_extension_confirm.json
- 22:38:41 EDT [run] Ran the tracked `INC-0079` larger-bank boundary confirm on the narrowed `Q08/Q12` onset bracket for `12000` and `18000`.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0079_product_phase_translation_large_bank_boundary_extension_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_confirm_profile.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_223841.md, /Users/adminamn/ai-router/router-research/docs/reports/INC0079_PRODUCT_PHASE_TRANSLATION_LARGE_BANK_BOUNDARY_EXTENSION_CONFIRM.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0079_product_phase_translation_large_bank_boundary_extension_confirm.json && python tools/translated_hardware_profile.py --analysis results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_confirm.json --output-json results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_confirm_profile.json --output-md docs/reports/INC0079_PRODUCT_PHASE_TRANSLATION_LARGE_BANK_BOUNDARY_EXTENSION_CONFIRM.md`
- 22:38:41 EDT [decision] Closed `INC-0079` confirm positive/narrow: the fixed translated product systems branch now confirms an even earlier onset at `Q08` for `max_train=18000`, while `max_train=12000` still begins at `Q12` and the search-work ratio stays pinned near `0.19`.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0079_product_phase_translation_large_bank_boundary_extension.md, /Users/adminamn/ai-router/router-research/results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0079_product_phase_translation_large_bank_boundary_extension_confirm_profile.json, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 22:38:41 EDT [next] Queued `INC-0080` next: extend the fixed onset map to a second larger bank rather than reopening geometry or retrieval rescue.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0080_product_phase_translation_second_large_bank_boundary_extension.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 23:05:36 EDT [run] Ran the tracked `INC-0080` second larger-bank boundary screen on the fixed translated systems family at `max_train={24000,30000}` and `Q={4,8,12}`.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0080_product_phase_translation_second_large_bank_boundary_extension_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_screen_profile.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_230536.md, /Users/adminamn/ai-router/router-research/docs/reports/INC0080_PRODUCT_PHASE_TRANSLATION_SECOND_LARGE_BANK_BOUNDARY_EXTENSION_SCREEN.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0080_product_phase_translation_second_large_bank_boundary_extension_screen.json && python tools/translated_hardware_profile.py --analysis results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_screen.json --output-json results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_screen_profile.json --output-md docs/reports/INC0080_PRODUCT_PHASE_TRANSLATION_SECOND_LARGE_BANK_BOUNDARY_EXTENSION_SCREEN.md`
- 23:05:36 EDT [decision] `INC-0080` screen justified confirm because the onset held coherently at `Q08` for both `T24000` and `T30000`, while `Q04` still did not cross and the secondary-key work ratio stayed near `0.19`.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_screen_profile.json, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0080_product_phase_translation_second_large_bank_boundary_extension.md, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm.json
- 23:26:57 EDT [run] Ran the tracked `INC-0080` second larger-bank boundary confirm on the narrowed `Q04/Q08` threshold bracket for `24000` and `30000`.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm_profile.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_232657.md, /Users/adminamn/ai-router/router-research/docs/reports/INC0080_PRODUCT_PHASE_TRANSLATION_SECOND_LARGE_BANK_BOUNDARY_EXTENSION_CONFIRM.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm.json && python tools/translated_hardware_profile.py --analysis results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm.json --output-json results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm_profile.json --output-md docs/reports/INC0080_PRODUCT_PHASE_TRANSLATION_SECOND_LARGE_BANK_BOUNDARY_EXTENSION_CONFIRM.md`
- 23:26:57 EDT [decision] Closed `INC-0080` confirm positive/narrow: the fixed translated product systems branch now holds its `Q08` onset through `max_train=30000`, `Q04` still does not cross, and the search-work ratio remains stable near `0.19`.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0080_product_phase_translation_second_large_bank_boundary_extension.md, /Users/adminamn/ai-router/router-research/results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0080_product_phase_translation_second_large_bank_boundary_extension_confirm_profile.json, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 23:26:57 EDT [next] Queued `INC-0081` next: search explicitly for the first real `Q04` crossover above `30000` rather than claiming a new earlier onset without evidence.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0081_product_phase_translation_q04_threshold_search.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 23:49:46 EDT [run] Ran the tracked `INC-0081` `Q04` threshold screen on the fixed translated systems family at `max_train={36000,40000}` and `Q={4,8}`.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0081_product_phase_translation_q04_threshold_search_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0081_product_phase_translation_q04_threshold_search_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0081_product_phase_translation_q04_threshold_search_screen_profile.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260311_234946.md, /Users/adminamn/ai-router/router-research/docs/reports/INC0081_PRODUCT_PHASE_TRANSLATION_Q04_THRESHOLD_SEARCH_SCREEN.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0081_product_phase_translation_q04_threshold_search_screen.json && python tools/translated_hardware_profile.py --analysis results/analysis/inc0081_product_phase_translation_q04_threshold_search_screen.json --output-json results/analysis/inc0081_product_phase_translation_q04_threshold_search_screen_profile.json --output-md docs/reports/INC0081_PRODUCT_PHASE_TRANSLATION_Q04_THRESHOLD_SEARCH_SCREEN.md`
- 23:49:46 EDT [decision] `INC-0081` screen justified confirm because the first apparent `Q04` crossover at `T36000` was real enough to carry, while `T40000` still began at `Q08`, making the threshold search clearly non-monotone.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0081_product_phase_translation_q04_threshold_search_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0081_product_phase_translation_q04_threshold_search_screen_profile.json, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0081_product_phase_translation_q04_threshold_search.md, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0081_product_phase_translation_q04_threshold_search_confirm.json
- 00:20:00 EDT [run] Ran the tracked `INC-0081` `Q04` threshold confirm on the fixed `Q04/Q08` bracket for `36000` and `40000`.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0081_product_phase_translation_q04_threshold_search_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0081_product_phase_translation_q04_threshold_search_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0081_product_phase_translation_q04_threshold_search_confirm_profile.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260312_002000.md, /Users/adminamn/ai-router/router-research/docs/reports/INC0081_PRODUCT_PHASE_TRANSLATION_Q04_THRESHOLD_SEARCH_CONFIRM.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0081_product_phase_translation_q04_threshold_search_confirm.json && python tools/translated_hardware_profile.py --analysis results/analysis/inc0081_product_phase_translation_q04_threshold_search_confirm.json --output-json results/analysis/inc0081_product_phase_translation_q04_threshold_search_confirm_profile.json --output-md docs/reports/INC0081_PRODUCT_PHASE_TRANSLATION_Q04_THRESHOLD_SEARCH_CONFIRM.md`
- 00:20:00 EDT [decision] Closed `INC-0081` confirm positive/narrow: the fixed translated product systems branch now has a first real confirmed `Q04` crossover at `T36000`, but the threshold is not monotone because `T40000` still begins at `Q08`.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0081_product_phase_translation_q04_threshold_search.md, /Users/adminamn/ai-router/router-research/results/analysis/inc0081_product_phase_translation_q04_threshold_search_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0081_product_phase_translation_q04_threshold_search_confirm_profile.json, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 00:20:00 EDT [next] Queued `INC-0082` next: explain the `36000` versus `40000` threshold split with direct cost-accounting / memory-traffic analysis instead of extending the bank map again.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0082_product_phase_translation_cost_accounting_audit.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 00:40:55 EDT [run] Implemented and ran the fixed translated cost-accounting audit on the `INC-0081` confirm bracket after adding the missing script-entry import path shim.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_cost_accounting.py, /Users/adminamn/ai-router/router-research/tests/test_translated_cost_accounting.py, /Users/adminamn/ai-router/router-research/results/analysis/inc0082_product_phase_translation_cost_accounting_audit.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0082_PRODUCT_PHASE_TRANSLATION_COST_ACCOUNTING_AUDIT.md
  Cmd: `python tools/translated_cost_accounting.py --analysis results/analysis/inc0081_product_phase_translation_q04_threshold_search_confirm.json --meta data/wikitext2_proxy/wikitext2_proxy_meta.json --route-id DENSE_Q04_T36000 --route-id H4XH4_FIELD_A150_CPX8_Q04_T36000 --route-id DENSE_Q04_T40000 --route-id H4XH4_FIELD_A150_CPX8_Q04_T40000 --route-id DENSE_Q08_T40000 --route-id H4XH4_FIELD_A150_CPX8_Q08_T40000 --output-json results/analysis/inc0082_product_phase_translation_cost_accounting_audit.json --output-md docs/reports/INC0082_PRODUCT_PHASE_TRANSLATION_COST_ACCOUNTING_AUDIT.md`
- 00:40:55 EDT [verify] Re-ran the full repo unit suite after the new audit tool landed.
  Cmd: `python -m unittest discover -s /Users/adminamn/ai-router/router-research/tests -v`
- 00:40:55 EDT [decision] Closed `INC-0082` positive/explanatory: the non-monotone `Q04/Q08` threshold split is explained by static offline route-build cost composition on the fixed translated stack, while the pruning/search signal itself stays stable.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0082_product_phase_translation_cost_accounting_audit.md, /Users/adminamn/ai-router/router-research/results/analysis/inc0082_product_phase_translation_cost_accounting_audit.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0082_PRODUCT_PHASE_TRANSLATION_COST_ACCOUNTING_AUDIT.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 00:40:55 EDT [next] Queued `INC-0083` next: activate persistent route-cache reuse on the fixed translated product stack to reduce the static offline build cost without changing the route law or search signal.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0083_product_phase_translation_persistent_route_cache.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md

## 2026-03-12
- 01:51:56 EDT [run] Ran INC-0083 cold and warm confirm sweeps plus compare on the fixed T40000 translated stack.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0083_product_phase_translation_persistent_route_cache_confirm_cold.json, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0083_product_phase_translation_persistent_route_cache_confirm_warm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0083_product_phase_translation_persistent_route_cache_confirm_cold.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0083_product_phase_translation_persistent_route_cache_confirm_warm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0083_product_phase_translation_persistent_route_cache_confirm_compare.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0083_PRODUCT_PHASE_TRANSLATION_PERSISTENT_ROUTE_CACHE_CONFIRM_COMPARE.md
- 01:51:56 EDT [decision] Closed INC-0083 positive/narrow: persistent cache reuse removes almost all static offline build cost on the fixed translated product stack, rescuing Q04 T40000 without changing top1 or candidate fraction.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0083_product_phase_translation_persistent_route_cache.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 01:51:56 EDT [next] Queued INC-0084 next: map warm-cache onset on the fixed T40000 translated stack across Q01/Q02/Q04/Q08 without changing the route law, secondary-key law, bank, or cache implementation.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0084_product_phase_translation_warm_cache_onset_map.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md
- 02:16:55 EDT [decision] Closed INC-0084 positive/narrow: the fixed translated product stack now has a confirmed warm-cache single-query crossover at Q01 T40000, with routed top1/candidate fraction unchanged and both caches hitting on every routed run.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0084_product_phase_translation_warm_cache_onset_map.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 02:16:55 EDT [run] Ran INC-0084 warm-cache onset screen and confirm on the fixed T40000 translated stack across Q01/Q02/Q04/Q08.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0084_product_phase_translation_warm_cache_onset_map_screen.json, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0084_product_phase_translation_warm_cache_onset_map_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0084_product_phase_translation_warm_cache_onset_map_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0084_product_phase_translation_warm_cache_onset_map_confirm.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0084_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_ONSET_MAP.md
- 02:16:55 EDT [next] Queued INC-0085 next: search for the earliest bank where warm-cache Q01 crossover holds on the fixed translated product stack, using only the Q01/Q02 bracket under explicit persisted-bank conditions.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0085_product_phase_translation_warm_cache_q01_bank_boundary.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md
- 05:04:20 EDT [decision] Closed INC-0085 positive/narrow and promoted H4XH4_FIELD_A150_CPX8_Q01_T3000 as the earliest tracked confirmed warm-cache single-query crossover point.
  Files: docs/research/increments/INC_0085_product_phase_translation_warm_cache_q01_bank_boundary.md, results/analysis/inc0085_product_phase_translation_warm_cache_q01_bank_boundary_confirm.json, results/analysis/inc0085_product_phase_translation_warm_cache_q01_bank_boundary_confirm_profile.json
- 05:04:20 EDT [run] Ran tracked INC-0085 prewarm screen/screen/prewarm confirm/confirm on the fixed warm-cache Q01 bank-boundary bracket.
  Files: configs/proxy_transfer_inc0085_product_phase_translation_warm_cache_q01_bank_boundary_prewarm_screen.json, configs/proxy_transfer_inc0085_product_phase_translation_warm_cache_q01_bank_boundary_screen.json, configs/proxy_transfer_inc0085_product_phase_translation_warm_cache_q01_bank_boundary_prewarm_confirm.json, configs/proxy_transfer_inc0085_product_phase_translation_warm_cache_q01_bank_boundary_confirm.json, results/analysis/inc0085_product_phase_translation_warm_cache_q01_bank_boundary_screen.json, results/analysis/inc0085_product_phase_translation_warm_cache_q01_bank_boundary_confirm.json
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0085_product_phase_translation_warm_cache_q01_bank_boundary_prewarm_screen.json && python tools/proxy_sweep.py --config configs/proxy_transfer_inc0085_product_phase_translation_warm_cache_q01_bank_boundary_screen.json && python tools/proxy_sweep.py --config configs/proxy_transfer_inc0085_product_phase_translation_warm_cache_q01_bank_boundary_prewarm_confirm.json && python tools/proxy_sweep.py --config configs/proxy_transfer_inc0085_product_phase_translation_warm_cache_q01_bank_boundary_confirm.json`
- 05:04:20 EDT [next] Queue INC-0086 next: refine the lower bank boundary below T3000 while holding the fixed route law, secondary-key law, and warm-cache implementation constant.
  Files: docs/research/increments/INC_0086_product_phase_translation_warm_cache_q01_lower_boundary_refine.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/routes/ROUTE_MATRIX.md
- 05:39:11 EDT [run] Ran tracked INC-0086 prewarm screen/screen/prewarm confirm/confirm on the fixed warm-cache lower-bound bracket T2500/T2750/T3000 with Q01/Q02.
  Files: configs/proxy_transfer_inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_prewarm_screen.json, configs/proxy_transfer_inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_screen.json, configs/proxy_transfer_inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_prewarm_confirm.json, configs/proxy_transfer_inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_confirm.json, results/analysis/inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_confirm.json, results/analysis/inc0086_product_phase_translation_warm_cache_q01_lower_boundary_refine_confirm_profile.json, docs/reports/INC0086_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_LOWER_BOUNDARY_REFINE_CONFIRM.md
- 05:39:11 EDT [decision] Closed INC-0086 positive/narrow: Q01 crosses by T2750, T2500 still misses at Q01 but crosses at Q02, and the next honest branch is a threshold refine inside 2500-2750.
  Files: docs/research/increments/INC_0086_product_phase_translation_warm_cache_q01_lower_boundary_refine.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md
- 05:39:11 EDT [run] Ran tracked INC-0087 prewarm screen/screen/prewarm confirm/confirm on the fixed warm-cache threshold bracket T2600/T2650/T2700 with Q01/Q02, then audited the same confirm bracket with translated_cost_accounting.
  Files: configs/proxy_transfer_inc0087_product_phase_translation_warm_cache_q01_threshold_refine_prewarm_screen.json, configs/proxy_transfer_inc0087_product_phase_translation_warm_cache_q01_threshold_refine_screen.json, configs/proxy_transfer_inc0087_product_phase_translation_warm_cache_q01_threshold_refine_prewarm_confirm.json, configs/proxy_transfer_inc0087_product_phase_translation_warm_cache_q01_threshold_refine_confirm.json, results/analysis/inc0087_product_phase_translation_warm_cache_q01_threshold_refine_confirm.json, results/analysis/inc0087_product_phase_translation_warm_cache_q01_threshold_refine_confirm_profile.json, results/analysis/inc0087_product_phase_translation_warm_cache_q01_threshold_refine_cost_audit.json, docs/reports/INC0087_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_THRESHOLD_REFINE_CONFIRM.md, docs/reports/INC0087_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_THRESHOLD_REFINE_COST_AUDIT.md
- 05:39:11 EDT [decision] Closed INC-0087 positive/narrow and INC-0088 positive/explanatory: T2600 is now the earliest tracked confirmed warm-cache Q01 point, T2650 is a local cost pocket rather than route failure, and the next branch is a final refine inside 2500-2600.
  Files: docs/research/increments/INC_0087_product_phase_translation_warm_cache_q01_threshold_refine.md, docs/research/increments/INC_0088_product_phase_translation_warm_cache_q01_local_cost_audit.md, docs/research/increments/INC_0089_product_phase_translation_warm_cache_q01_2500_2600_refine.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md, docs/DECISIONS.md, results/INDEX.md
- 05:46:55 EDT [run] Ran tracked INC-0089 prewarm screen/screen/prewarm confirm/confirm on the fixed warm-cache 2500-2600 bracket T2525/T2550/T2575 with Q01/Q02.
  Files: configs/proxy_transfer_inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_prewarm_screen.json, configs/proxy_transfer_inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_screen.json, configs/proxy_transfer_inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_prewarm_confirm.json, configs/proxy_transfer_inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_confirm.json, results/analysis/inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_confirm.json, results/analysis/inc0089_product_phase_translation_warm_cache_q01_2500_2600_refine_confirm_profile.json, docs/reports/INC0089_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_2500_2600_REFINE_CONFIRM.md
- 05:46:55 EDT [decision] Closed INC-0089 positive/narrow: T2525 is now the earliest tracked confirmed warm-cache Q01 point, Q02 T2500 remains the earliest any-repeat point, and the next branch is a final refine inside 2500-2525.
  Files: docs/research/increments/INC_0089_product_phase_translation_warm_cache_q01_2500_2600_refine.md, docs/research/increments/INC_0090_product_phase_translation_warm_cache_q01_2500_2525_refine.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md, docs/DECISIONS.md, results/INDEX.md
- 06:09:31 EDT [run] Ran tracked INC-0090 prewarm screen/screen/prewarm confirm/confirm on the fixed warm-cache 2500-2525 bracket T2505/T2510/T2515/T2520 with Q01/Q02.
  Files: configs/proxy_transfer_inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_prewarm_screen.json, configs/proxy_transfer_inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_screen.json, configs/proxy_transfer_inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_prewarm_confirm.json, configs/proxy_transfer_inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_confirm.json, results/analysis/inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_screen.json, results/analysis/inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_confirm.json, results/analysis/inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_screen_profile.json, results/analysis/inc0090_product_phase_translation_warm_cache_q01_2500_2525_refine_confirm_profile.json, docs/reports/INC0090_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_2500_2525_REFINE_SCREEN.md, docs/reports/INC0090_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_2500_2525_REFINE_CONFIRM.md
- 06:09:31 EDT [decision] Closed INC-0090 positive/narrow: T2505 is now the earliest tracked confirmed warm-cache Q01 point, Q02 T2500 remains the earliest any-repeat point, and the next branch is a final refine inside 2500-2505.
  Files: docs/research/increments/INC_0090_product_phase_translation_warm_cache_q01_2500_2525_refine.md, docs/research/increments/INC_0091_product_phase_translation_warm_cache_q01_2500_2505_refine.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md, docs/DECISIONS.md, results/INDEX.md
- 06:09:31 EDT [next] Queued INC-0091 next: refine the last warm-cache Q01 threshold gap inside T2500-T2505 while holding the fixed translated route law, secondary-key law, and persistent-cache implementation constant.
  Files: docs/research/increments/INC_0091_product_phase_translation_warm_cache_q01_2500_2505_refine.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md
- 06:32:30 EDT [run] Ran tracked INC-0091 prewarm screen/screen/prewarm confirm/confirm on the fixed warm-cache 2500-2505 bracket T2501/T2502/T2503/T2504 with Q01/Q02.
  Files: configs/proxy_transfer_inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_prewarm_screen.json, configs/proxy_transfer_inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_screen.json, configs/proxy_transfer_inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_prewarm_confirm.json, configs/proxy_transfer_inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_confirm.json, results/analysis/inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_screen.json, results/analysis/inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_confirm.json, results/analysis/inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_screen_profile.json, results/analysis/inc0091_product_phase_translation_warm_cache_q01_2500_2505_refine_confirm_profile.json, docs/reports/INC0091_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_2500_2505_REFINE_SCREEN.md, docs/reports/INC0091_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_2500_2505_REFINE_CONFIRM.md
- 06:32:30 EDT [decision] Closed INC-0091 positive/narrow: T2501 is now the earliest tracked confirmed warm-cache Q01 point, Q02 T2500 remains the earliest any-repeat point, and the lower-bound search is closed at integer resolution.
  Files: docs/research/increments/INC_0091_product_phase_translation_warm_cache_q01_2500_2505_refine.md, docs/research/increments/INC_0092_product_phase_translation_warm_cache_q01_floor_hardening.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md, docs/DECISIONS.md, results/INDEX.md
- 06:32:30 EDT [next] Queued INC-0092 next: harden the exact warm-cache Q01 floor on the T2500/T2501 bracket rather than refining T further.
  Files: docs/research/increments/INC_0092_product_phase_translation_warm_cache_q01_floor_hardening.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md
- 06:45:31 EDT [run] Ran tracked INC-0092 prewarm screen/screen/prewarm confirm/confirm on the exact warm-cache floor bracket T2500/T2501 with Q01/Q02, using an expanded 8-seed confirm.
  Files: configs/proxy_transfer_inc0092_product_phase_translation_warm_cache_q01_floor_hardening_prewarm_screen.json, configs/proxy_transfer_inc0092_product_phase_translation_warm_cache_q01_floor_hardening_screen.json, configs/proxy_transfer_inc0092_product_phase_translation_warm_cache_q01_floor_hardening_prewarm_confirm.json, configs/proxy_transfer_inc0092_product_phase_translation_warm_cache_q01_floor_hardening_confirm.json, results/analysis/inc0092_product_phase_translation_warm_cache_q01_floor_hardening_screen.json, results/analysis/inc0092_product_phase_translation_warm_cache_q01_floor_hardening_confirm.json, results/analysis/inc0092_product_phase_translation_warm_cache_q01_floor_hardening_screen_profile.json, results/analysis/inc0092_product_phase_translation_warm_cache_q01_floor_hardening_confirm_profile.json, docs/reports/INC0092_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_FLOOR_HARDENING_SCREEN.md, docs/reports/INC0092_PRODUCT_PHASE_TRANSLATION_WARM_CACHE_Q01_FLOOR_HARDENING_CONFIRM.md
- 06:45:31 EDT [decision] Closed INC-0092 positive/explanatory: the old exact T2500 miss / T2501 hit story does not survive hardening, T2500 now also crosses at Q01, and the lower-bound T/Q search is retired.
  Files: docs/research/increments/INC_0092_product_phase_translation_warm_cache_q01_floor_hardening.md, docs/research/increments/INC_0093_product_phase_translation_cache_residency_mix.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md, docs/DECISIONS.md, results/INDEX.md
- 06:45:31 EDT [next] Queued INC-0093 next: test cache-residency robustness on the fixed T2500 Q01 lower-bank floor and T40000 Q01 stabilized warm-cache point rather than refining bank thresholds again.
  Files: docs/research/increments/INC_0093_product_phase_translation_cache_residency_mix.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md
- 07:41:11 EDT [run] Ran tracked INC-0093 cold/prewarm/warm screen and confirm on the fixed translated product stack, decomposing cache residency into chart-only, route-only, and full warm across `T2500 Q01` and `T40000 Q01`.
  Files: configs/proxy_transfer_inc0093_product_phase_translation_cache_residency_mix_screen_cold.json, configs/proxy_transfer_inc0093_product_phase_translation_cache_residency_mix_screen_prewarm.json, configs/proxy_transfer_inc0093_product_phase_translation_cache_residency_mix_screen_warm.json, configs/proxy_transfer_inc0093_product_phase_translation_cache_residency_mix_confirm_cold.json, configs/proxy_transfer_inc0093_product_phase_translation_cache_residency_mix_confirm_prewarm.json, configs/proxy_transfer_inc0093_product_phase_translation_cache_residency_mix_confirm_warm.json, results/analysis/inc0093_product_phase_translation_cache_residency_mix_confirm_cold.json, results/analysis/inc0093_product_phase_translation_cache_residency_mix_confirm_warm.json, results/analysis/inc0093_product_phase_translation_cache_residency_mix_confirm_compare_chart.json, results/analysis/inc0093_product_phase_translation_cache_residency_mix_confirm_compare_route.json, results/analysis/inc0093_product_phase_translation_cache_residency_mix_confirm_compare_full.json, docs/reports/INC0093_PRODUCT_PHASE_TRANSLATION_CACHE_RESIDENCY_MIX_CONFIRM.md
- 07:41:11 EDT [decision] Closed INC-0093 positive/explanatory: chart residency carries almost all of the operational rescue, route-only residency stays negative at both anchors, the upper-bank `T40000 Q01` win survives under chart-only residency, and the exact lower-bank `T2500 Q01` floor remains a full-warm claim.
  Files: docs/research/increments/INC_0093_product_phase_translation_cache_residency_mix.md, docs/research/increments/INC_0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md, docs/DECISIONS.md, results/INDEX.md
- 07:41:11 EDT [next] Queued INC-0094 next: map the minimal repeat reuse needed under chart-resident / route-ephemeral operation on the same fixed `T2500` and `T40000` anchor points.
  Files: docs/research/increments/INC_0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md
- 08:23:31 EDT [run] Ran tracked INC-0094 prewarm screen/screen/prewarm confirm/confirm on the fixed chart-resident / route-ephemeral repeat ladder for `T2500` and `T40000` with `Q01/Q02/Q04`.
  Files: configs/proxy_transfer_inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_prewarm_screen.json, configs/proxy_transfer_inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_screen.json, configs/proxy_transfer_inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_prewarm_confirm.json, configs/proxy_transfer_inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_confirm.json, results/analysis/inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_screen.json, results/analysis/inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_screen_profile.json, results/analysis/inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_confirm.json, results/analysis/inc0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map_confirm_profile.json, docs/reports/INC0094_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_ROUTE_EPHEMERAL_REPEAT_MAP_CONFIRM.md
- 08:23:31 EDT [decision] Closed INC-0094 positive/narrow: chart-resident sessions now survive at `T2500` by `Q02` and at `T40000` already by `Q01`, while chart-only `T2500 Q01` still misses slightly.
  Files: docs/research/increments/INC_0094_product_phase_translation_chart_resident_route_ephemeral_repeat_map.md, docs/research/increments/INC_0095_product_phase_translation_chart_resident_q01_bank_boundary.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md, docs/DECISIONS.md, results/INDEX.md
- 08:23:31 EDT [next] Queued INC-0095 next: search for the earliest bank where chart-resident `Q01` survives on the fixed translated stack.
  Files: docs/research/increments/INC_0095_product_phase_translation_chart_resident_q01_bank_boundary.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md
- 09:11:38 EDT [run] Ran tracked INC-0095 prewarm screen/screen/prewarm confirm/confirm on the fixed chart-resident Q01 bank ladder, then profiled the focused lower-bank confirm slice T2500/T2750/T3000/T4000.
  Files: configs/proxy_transfer_inc0095_product_phase_translation_chart_resident_q01_bank_boundary_prewarm_screen.json, configs/proxy_transfer_inc0095_product_phase_translation_chart_resident_q01_bank_boundary_screen.json, configs/proxy_transfer_inc0095_product_phase_translation_chart_resident_q01_bank_boundary_prewarm_confirm.json, configs/proxy_transfer_inc0095_product_phase_translation_chart_resident_q01_bank_boundary_confirm.json, results/analysis/inc0095_product_phase_translation_chart_resident_q01_bank_boundary_screen.json, results/analysis/inc0095_product_phase_translation_chart_resident_q01_bank_boundary_screen_profile.json, results/analysis/inc0095_product_phase_translation_chart_resident_q01_bank_boundary_confirm.json, results/analysis/inc0095_product_phase_translation_chart_resident_q01_bank_boundary_confirm_profile.json, docs/reports/INC0095_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_Q01_BANK_BOUNDARY_CONFIRM.md
- 09:11:38 EDT [decision] Closed INC-0095 positive/explanatory: the focused chart-resident Q01 packet survives already at T2500, while the earlier INC-0094 mixed-repeat miss becomes a packet-scope sensitivity issue rather than a bank-location issue.
  Files: docs/research/increments/INC_0095_product_phase_translation_chart_resident_q01_bank_boundary.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md, docs/DECISIONS.md, results/INDEX.md
- 09:11:38 EDT [next] Queued INC-0096 next: audit packet-scope stability of the lower-bank chart-resident Q01 point by comparing the focused Q01 packet against the older mixed-repeat packet on the fixed translated stack.
  Files: docs/research/increments/INC_0096_product_phase_translation_chart_resident_q01_packet_scope_audit.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md
- 09:34:22 EDT [next] Queued INC-0097 next: reopen sparse or quantized phase-gated shell work now that the translated chart-resident lower-bank single-query story is stable across packet composition.
  Files: docs/research/increments/INC_0097_product_phase_sparse_gated_shell_pilot.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md
- 09:34:22 EDT [run] Ran tracked INC-0096 prewarm screen/focused screen/mixed screen/prewarm confirm/focused confirm/mixed confirm on the fixed chart-resident lower-bank Q01 point, then generated paired screen/confirm packet-scope compare artifacts.
  Files: configs/proxy_transfer_inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_prewarm_screen.json, configs/proxy_transfer_inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_focused_screen.json, configs/proxy_transfer_inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_mixed_screen.json, configs/proxy_transfer_inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_prewarm_confirm.json, configs/proxy_transfer_inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_focused_confirm.json, configs/proxy_transfer_inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_mixed_confirm.json, results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_focused_confirm.json, results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_mixed_confirm.json, results/analysis/inc0096_product_phase_translation_chart_resident_q01_packet_scope_audit_confirm_compare.json, docs/reports/INC0096_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_Q01_PACKET_SCOPE_AUDIT_CONFIRM_COMPARE.md
- 09:34:22 EDT [decision] Closed INC-0096 positive/explanatory: the chart-resident lower-bank Q01 point survives both focused and mixed packet shapes on hardening, so packet scope changes margin size but not the sign or the retrieval signal.
  Files: docs/research/increments/INC_0096_product_phase_translation_chart_resident_q01_packet_scope_audit.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md, docs/DECISIONS.md, results/INDEX.md
- 09:58:12 EDT [run] Reran the tracked INC-0097 sparse shell pilot after fixing the shell-controller args through `optimize_chart(...)`, then recorded the corrected 2-seed screen artifact.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0097_product_phase_sparse_gated_shell_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0097_product_phase_sparse_gated_shell_screen.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260312_095747.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0097_product_phase_sparse_gated_shell_screen.json`
- 09:58:12 EDT [decision] Closed INC-0097 negative at screen: gated and banded shell control are mechanism-live but both fail shell health, so neither candidate beats the continuous product reference on the pilot contract.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0097_product_phase_sparse_gated_shell_pilot.md, /Users/adminamn/ai-router/router-research/docs/reports/INC0097_PRODUCT_PHASE_SPARSE_GATED_SHELL_SCREEN.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 09:58:12 EDT [next] Queued INC-0098 next: decompose chart-resident routed cost on the fixed translated product stack instead of spending more time tuning local sparse shell thresholds.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0098_product_phase_translation_chart_resident_route_cost_decomposition.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 10:14:34 EDT [run] Built the curated INC-0098 cost-audit input and ran translated cost accounting on the fixed chart-resident anchors plus full-warm references.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0098_product_phase_translation_chart_resident_route_cost_decomposition_input.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0098_product_phase_translation_chart_resident_route_cost_decomposition.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0098_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_ROUTE_COST_DECOMPOSITION.md
  Cmd: `python tools/translated_cost_accounting.py --analysis results/analysis/inc0098_product_phase_translation_chart_resident_route_cost_decomposition_input.json --meta data/wikitext2_proxy/wikitext2_proxy_meta.json --route-id DENSE_Q01_T2500 --route-id CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500 --route-id FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500 --route-id DENSE_Q01_T40000 --route-id CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000 --route-id FULL_H4XH4_FIELD_A150_CPX8_Q01_T40000 --compare chart_vs_full_lower:FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500:CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500 --compare chart_vs_full_upper:FULL_H4XH4_FIELD_A150_CPX8_Q01_T40000:CHART_H4XH4_FIELD_A150_CPX8_Q01_T40000 --output-json results/analysis/inc0098_product_phase_translation_chart_resident_route_cost_decomposition.json --output-md docs/reports/INC0098_PRODUCT_PHASE_TRANSLATION_CHART_RESIDENT_ROUTE_COST_DECOMPOSITION.md`
- 10:14:34 EDT [decision] Closed INC-0098 positive/explanatory: chart-resident translated routing is already robust at both fixed anchors, and the remaining gap to full warm is split between route-index build and retrieval-search overhead rather than a single hidden route-materialization failure.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0098_product_phase_translation_chart_resident_route_cost_decomposition.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 10:14:34 EDT [next] Queued INC-0099 next: return to sparse event-driven proxy trainability with the translated chart-resident stack frozen as the current hardware-side reference.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0099_product_phase_sparse_event_proxy_pilot.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 10:51:28 EDT [decision] Closed INC-0099 positive/narrow: H4XH4_FIELD_A150_EVT_T070 is a healthy soft-sparse proxy point with event_gate_mean about 0.319, but hard event firing remains unresolved.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0099_product_phase_sparse_event_proxy_pilot.md, /Users/adminamn/ai-router/router-research/docs/reports/INC0099_PRODUCT_PHASE_SPARSE_EVENT_PROXY_CONFIRM.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 10:51:28 EDT [run] Ran INC-0099 sparse-event proxy screen and confirm packets after targeted regression tests passed.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0099_product_phase_sparse_event_proxy_screen.json, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0099_product_phase_sparse_event_proxy_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0099_product_phase_sparse_event_proxy_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0099_product_phase_sparse_event_proxy_confirm.json
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0099_product_phase_sparse_event_proxy_screen.json && python tools/proxy_sweep.py --config configs/proxy_transfer_inc0099_product_phase_sparse_event_proxy_confirm.json`
- 10:51:28 EDT [next] Queue INC-0100 next: carry the fixed sparse-event point into the translated retrieval harness while keeping the chart-resident stack frozen as the downstream systems reference.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0100_product_phase_sparse_event_translation_pilot.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md
- 10:51:28 EDT [edit] Implemented proxy-only soft sparse-event gating in router_proxy_eval, extended proxy_sweep summaries, and added regression coverage for the new event surface.
  Files: /Users/adminamn/ai-router/router-research/tasks/router_proxy_eval.py, /Users/adminamn/ai-router/router-research/tools/proxy_sweep.py, /Users/adminamn/ai-router/router-research/tests/test_router_proxy_eval.py, /Users/adminamn/ai-router/router-research/tests/test_cli_contract.py, /Users/adminamn/ai-router/router-research/tests/test_cache_contract.py, /Users/adminamn/ai-router/router-research/tests/test_proxy_sweep.py
- 11:11:05 EDT [decision] Closed INC-0100 positive/narrow: the soft sparse controller preserves translated routed retrieval signal, materially improves runtime versus the continuous routed product reference, and reaches a knife-edge lower-bank amortized tie with dense exact while remaining soft-sparse rather than hard-firing.
  Files: docs/research/increments/INC_0100_product_phase_sparse_event_translation_pilot.md, docs/reports/INC0100_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_CONFIRM.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md, docs/DECISIONS.md, results/INDEX.md
- 11:11:05 EDT [edit] Threaded the INC-0099 sparse-event controller into the translated retrieval harness and exposed retrieval-side sparse-event metrics.
  Files: tasks/router_retrieval_eval.py, tests/test_router_retrieval_eval.py, tests/test_cli_contract.py, configs/proxy_transfer_inc0100_product_phase_sparse_event_translation_prewarm_screen.json, configs/proxy_transfer_inc0100_product_phase_sparse_event_translation_screen.json, configs/proxy_transfer_inc0100_product_phase_sparse_event_translation_prewarm_confirm.json, configs/proxy_transfer_inc0100_product_phase_sparse_event_translation_confirm.json
- 11:11:05 EDT [run] Ran the tracked INC-0100 prewarm, screen, and confirm packets on the translated lower-bank chart-resident stack.
  Files: results/analysis/inc0100_product_phase_sparse_event_translation_screen.json, results/analysis/inc0100_product_phase_sparse_event_translation_confirm.json, docs/governance/gates/gate_20260312_110233.md, docs/governance/gates/gate_20260312_110324.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0100_product_phase_sparse_event_translation_confirm.json`
- 11:11:05 EDT [next] Queue INC-0101 next: test hard or near-hard event activation on the fixed product route law at the proxy level before reopening translated mapping.
  Files: docs/research/increments/INC_0101_product_phase_hard_event_proxy_pilot.md
- 11:51:21 EDT [edit] Completed INC-0101 proxy hard-event surface, configs, reports, and handoff-state reconciliation.
  Files: /Users/adminamn/ai-router/router-research/tasks/router_proxy_eval.py, /Users/adminamn/ai-router/router-research/tests/test_router_proxy_eval.py, /Users/adminamn/ai-router/router-research/tests/test_cli_contract.py, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0101_product_phase_hard_event_proxy_screen.json, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0101_product_phase_hard_event_proxy_confirm.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0101_PRODUCT_PHASE_HARD_EVENT_PROXY_SCREEN.md, /Users/adminamn/ai-router/router-research/docs/reports/INC0101_PRODUCT_PHASE_HARD_EVENT_PROXY_CONFIRM.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0101_product_phase_hard_event_proxy_pilot.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0102_product_phase_near_hard_event_translation_pilot.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 11:51:21 EDT [run] Ran INC-0101 screen and confirm plus full regression suite.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0101_product_phase_hard_event_proxy_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0101_product_phase_hard_event_proxy_confirm.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260312_113952.md, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260312_114258.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0101_product_phase_hard_event_proxy_screen.json && python tools/proxy_sweep.py --config configs/proxy_transfer_inc0101_product_phase_hard_event_proxy_confirm.json && python -m unittest discover -s tests -v`
- 11:51:21 EDT [decision] Closed INC-0101 positive/narrow on near-hard event activation, promoted H4XH4_FIELD_A150_EVT_T070_TAU002 as the proxy near-hard reference, and queued INC-0102 for translated carry-forward.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0101_product_phase_hard_event_proxy_pilot.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0102_product_phase_near_hard_event_translation_pilot.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 11:58:21 EDT [run] Ran INC-0102 translated near-hard prewarm and screen on the fixed product route law.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0102_product_phase_near_hard_event_translation_prewarm_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0102_product_phase_near_hard_event_translation_screen.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260312_115408.md, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260312_115418.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0102_product_phase_near_hard_event_translation_prewarm_screen.json && python tools/proxy_sweep.py --config configs/proxy_transfer_inc0102_product_phase_near_hard_event_translation_screen.json`
- 11:58:21 EDT [decision] Closed INC-0102 negative at screen: near-hard translated carry-forward preserved retrieval signal but lost the systems tradeoff, so the translated sparse-event story remains explicitly soft and the queue moves to INC-0103.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0102_product_phase_near_hard_event_translation_pilot.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0103_product_phase_soft_sparse_translation_quality_recovery.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 12:33:21 EDT [run] Ran tracked INC-0103 confirm, INC-0104 prewarm/screen/confirm, and INC-0105 upper-bank prewarm/screen/confirm on the fixed sparse translated stack.
  Files: configs/proxy_transfer_inc0103_product_phase_soft_sparse_translation_quality_recovery_confirm.json, results/analysis/inc0103_product_phase_soft_sparse_translation_quality_recovery_confirm.json, configs/proxy_transfer_inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json, results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json, configs/proxy_transfer_inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json, results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0103_product_phase_soft_sparse_translation_quality_recovery_confirm.json && python tools/proxy_sweep.py --config configs/proxy_transfer_inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json && python tools/proxy_sweep.py --config configs/proxy_transfer_inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json`
- 12:33:33 EDT [decision] Closed INC-0103 negative on quality recovery; closed INC-0104 negative on quality recovery but positive on lower-bank systems refinement; closed INC-0105 positive/narrow on upper-bank systems carry-forward.
  Files: docs/research/increments/INC_0103_product_phase_soft_sparse_translation_quality_recovery.md, docs/research/increments/INC_0104_product_phase_soft_sparse_translation_backfill_recovery.md, docs/research/increments/INC_0105_product_phase_soft_sparse_translation_upper_bank_carry_forward.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md, docs/DECISIONS.md, results/INDEX.md
- 12:33:44 EDT [next] Queued INC-0106 next: decompose the lower and upper sparse translated systems leads against the fixed quality references using cost accounting, without reopening new recovery heuristics or T/Q mapping.
  Files: docs/research/increments/INC_0106_product_phase_sparse_translation_systems_cost_decomposition.md, docs/research/CURRENT_DIRECTION.md, docs/research/HANDOFF_CURRENT.md, docs/research/LIVE_WORKLOG.md, docs/routes/ROUTE_MATRIX.md
- 12:49:07 EDT [run] Ran the completed INC-0106 sparse translated systems cost decomposition and the new INC-0107 per-seed component stability audit on the fixed INC-0104/0105 confirm artifacts.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0106_product_phase_sparse_translation_systems_cost_decomposition.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0107_product_phase_sparse_translation_component_stability_audit.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0106_PRODUCT_PHASE_SPARSE_TRANSLATION_SYSTEMS_COST_DECOMPOSITION.md, /Users/adminamn/ai-router/router-research/docs/reports/INC0107_PRODUCT_PHASE_SPARSE_TRANSLATION_COMPONENT_STABILITY_AUDIT.md
  Cmd: `python tools/translated_cost_accounting.py ... && python tools/translated_component_stability.py ...`
- 12:49:07 EDT [edit] Added the translated component stability audit tool and reconciled the increment/state docs through INC-0107, queuing INC-0108 next.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_component_stability.py, /Users/adminamn/ai-router/router-research/tests/test_translated_component_stability.py, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0106_product_phase_sparse_translation_systems_cost_decomposition.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0107_product_phase_sparse_translation_component_stability_audit.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0108_product_phase_sparse_translation_repeated_timing_hardening.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 12:49:07 EDT [decision] Closed INC-0106 positive/explanatory and INC-0107 positive/explanatory: bounded backfill stays as the sparse translated systems lead, but upper-bank route_query deltas are not seed-stable enough to drive optimization.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0106_product_phase_sparse_translation_systems_cost_decomposition.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0107_product_phase_sparse_translation_component_stability_audit.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md
- 12:49:07 EDT [next] Queued INC-0108 next: repeat the exact lower/upper sparse translated pair timings to separate timing noise from genuine route/search variance without changing the route law or retrieval heuristics.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0108_product_phase_sparse_translation_repeated_timing_hardening.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 13:18:42 EDT [run] Ran INC-0108 repeated timing hardening with two fresh warmed-chart reruns at the lower and upper sparse translated anchors, then aggregated the original confirm plus both repeats into the hardening audit.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r2.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_lower_r3.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r2.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening_upper_r3.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0108_product_phase_sparse_translation_repeated_timing_hardening.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0108_PRODUCT_PHASE_SPARSE_TRANSLATION_REPEATED_TIMING_HARDENING.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0108_* && python tools/translated_repeated_timing_hardening.py ...`
- 13:18:42 EDT [edit] Added the repeated-timing hardening audit helper, repeat packet configs, and reconciled the increment/state docs through INC-0108 with INC-0109 queued next.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_repeated_timing_hardening.py, /Users/adminamn/ai-router/router-research/tests/test_translated_repeated_timing_hardening.py, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0108_product_phase_sparse_translation_repeated_timing_hardening.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0109_product_phase_sparse_translation_robust_cost_reference.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 13:18:43 EDT [decision] Closed INC-0108 positive/explanatory: bounded backfill remains the mean sparse translated systems lead, but repeated wallclock component deltas still flip within seed, so stable pruning rather than microtiming is the reliable systems signal.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0108_product_phase_sparse_translation_repeated_timing_hardening.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md
- 13:18:44 EDT [next] Queued INC-0109 next: build a robust cost reference on the completed INC-0104/0105/0108 evidence instead of chasing more wallclock micro-heuristics.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0109_product_phase_sparse_translation_robust_cost_reference.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 13:39:11 EDT [run] Built the robust sparse translated cost-reference audit directly from the fixed INC-0104/0105 confirm artifacts and the INC-0108 repeated packets.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0109_product_phase_sparse_translation_robust_cost_reference.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0109_PRODUCT_PHASE_SPARSE_TRANSLATION_ROBUST_COST_REFERENCE.md
  Cmd: `python tools/translated_robust_cost_reference.py --analysis ... --compare ... --output-json results/analysis/inc0109_product_phase_sparse_translation_robust_cost_reference.json --output-md docs/reports/INC0109_PRODUCT_PHASE_SPARSE_TRANSLATION_ROBUST_COST_REFERENCE.md`
- 13:39:11 EDT [edit] Added the robust cost-reference audit helper and reconciled the increment/state docs through INC-0109, queuing INC-0110 next.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_robust_cost_reference.py, /Users/adminamn/ai-router/router-research/tests/test_translated_robust_cost_reference.py, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0109_product_phase_sparse_translation_robust_cost_reference.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0110_product_phase_sparse_translation_dense_robust_hardening.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 13:39:11 EDT [decision] Closed INC-0109 positive/explanatory: upper-bank bounded backfill remains the clean robust systems lead, lower-bank bounded backfill remains robust versus the continuous translated product reference, and the lower-bank soft sparse comparison is now narrowed to a pruning-first read instead of a clean robust wallclock promotion.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0109_product_phase_sparse_translation_robust_cost_reference.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 13:39:11 EDT [next] Queued INC-0110 next: harden the fixed sparse translated dense-frontier claim under repeated timing at the lower and upper Q01 anchors instead of reopening more bounded recovery heuristics.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0110_product_phase_sparse_translation_dense_robust_hardening.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 14:22:03 EDT [run] Built the dense-frontier robust hardening audit from the completed INC-0104/0105 confirms plus the repeated warmed INC-0108 lower and upper packets that already include dense.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0110_product_phase_sparse_translation_dense_robust_hardening.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0110_PRODUCT_PHASE_SPARSE_TRANSLATION_DENSE_ROBUST_HARDENING.md
  Cmd: `python tools/translated_robust_cost_reference.py --analysis ... --compare lower_dense_vs_soft_sparse:... --compare lower_dense_vs_backfill:... --compare upper_dense_vs_soft_sparse:... --compare upper_dense_vs_backfill:... --output-json results/analysis/inc0110_product_phase_sparse_translation_dense_robust_hardening.json --output-md docs/reports/INC0110_PRODUCT_PHASE_SPARSE_TRANSLATION_DENSE_ROBUST_HARDENING.md`
- 14:22:03 EDT [edit] Fixed the robust audit tool polarity for top-1 deltas, regenerated the INC-0109 audit, added the INC-0111 branch doc, and reconciled the increment/state docs through INC-0110.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_robust_cost_reference.py, /Users/adminamn/ai-router/router-research/tests/test_translated_robust_cost_reference.py, /Users/adminamn/ai-router/router-research/results/analysis/inc0109_product_phase_sparse_translation_robust_cost_reference.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0109_PRODUCT_PHASE_SPARSE_TRANSLATION_ROBUST_COST_REFERENCE.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0110_product_phase_sparse_translation_dense_robust_hardening.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0111_product_phase_sparse_translation_dense_quality_frontier.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 14:22:03 EDT [decision] Closed INC-0110 positive/explanatory: lower-bank backfill remains the only robust lower-bank dense systems promotion, both upper-bank sparse translated points remain robust dense systems promotions, and every sparse translated dense comparison still carries a robust top-1 deficit versus dense exact.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0110_product_phase_sparse_translation_dense_robust_hardening.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 14:22:03 EDT [next] Queued INC-0111 next: derive a dense quality/system frontier for the fixed sparse translated points instead of reopening more timing or recovery heuristics.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0111_product_phase_sparse_translation_dense_quality_frontier.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 16:05:12 EDT [run] Built the dense quality/system frontier audit directly from the completed INC-0104/0105 confirm artifacts and the INC-0110 robust dense summaries.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0111_product_phase_sparse_translation_dense_quality_frontier.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0111_PRODUCT_PHASE_SPARSE_TRANSLATION_DENSE_QUALITY_FRONTIER.md
  Cmd: `python tools/translated_dense_quality_frontier.py --robust-analysis results/analysis/inc0110_product_phase_sparse_translation_dense_robust_hardening.json --anchor-analysis lower=results/analysis/inc0104_product_phase_soft_sparse_translation_backfill_recovery_confirm.json --anchor-analysis upper=results/analysis/inc0105_product_phase_soft_sparse_translation_upper_bank_carry_forward_confirm.json --comparison lower_dense_vs_soft_sparse:... --comparison lower_dense_vs_backfill:... --comparison upper_dense_vs_soft_sparse:... --comparison upper_dense_vs_backfill:... --output-json results/analysis/inc0111_product_phase_sparse_translation_dense_quality_frontier.json --output-md docs/reports/INC0111_PRODUCT_PHASE_SPARSE_TRANSLATION_DENSE_QUALITY_FRONTIER.md`
- 16:05:12 EDT [edit] Added the dense quality-frontier audit helper, reconciled the INC-0111 increment doc, and updated the source-of-truth files through INC-0111 with INC-0112 queued next.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_dense_quality_frontier.py, /Users/adminamn/ai-router/router-research/tests/test_translated_dense_quality_frontier.py, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0111_product_phase_sparse_translation_dense_quality_frontier.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 16:05:12 EDT [decision] Closed INC-0111 positive/explanatory: lower-bank sparse translated dense replacement remains systems-only, both upper-bank sparse translated points now sit in the quality-near dense systems band, and the dense claim is now explicitly split between lower-bank systems-only and upper-bank near-frontier.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0111_product_phase_sparse_translation_dense_quality_frontier.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 16:05:12 EDT [next] Queued INC-0112 next: harden the small upper-bank dense quality gap on the fixed sparse translated points without reopening lower-bank rescue or new sparse heuristics.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 16:57:27 EDT [run] Ran two fresh paired upper-bank repeats and built the focused INC-0112 upper-bank robust and quality-tolerance audits.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_prewarm_r4.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_r4.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_prewarm_r5.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_r5.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0112_product_phase_sparse_translation_upper_bank_dense_robust_hardening.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0112_PRODUCT_PHASE_SPARSE_TRANSLATION_UPPER_BANK_DENSE_ROBUST_HARDENING.md, /Users/adminamn/ai-router/router-research/docs/reports/INC0112_PRODUCT_PHASE_SPARSE_TRANSLATION_UPPER_BANK_DENSE_QUALITY_TOLERANCE_HARDENING.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0112_* && python tools/translated_robust_cost_reference.py ... && python tools/translated_dense_quality_frontier.py ...`
- 16:57:27 EDT [edit] Added the focused upper-bank INC-0112 repeat configs, refined the dense-quality frontier report tool for upper-only branches, and reconciled the source-of-truth docs through INC-0112 with INC-0113 queued next.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_prewarm_r4.json, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_r4.json, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_prewarm_r5.json, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening_r5.json, /Users/adminamn/ai-router/router-research/tools/translated_dense_quality_frontier.py, /Users/adminamn/ai-router/router-research/tests/test_translated_dense_quality_frontier.py, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 16:57:27 EDT [decision] Closed INC-0112 positive/explanatory: both upper-bank sparse translated points remain near-frontier dense systems promotions under two fresh paired repeats, but the residual top-1 gap remains small and robustly negative.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0112_product_phase_sparse_translation_upper_bank_dense_quality_tolerance_hardening.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 16:57:27 EDT [next] Queued INC-0113 next: decompose the residual upper-bank dense top-1 gap into candidate omission versus in-candidate ordering loss before opening any new rescue heuristic.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 15:26:57 EDT [edit] Added retrieval query-audit support to the retrieval harness, added the dense gap decomposition tool/tests, created the tracked INC-0113 prewarm/confirm configs, and reconciled the source-of-truth docs through INC-0113 with INC-0114 queued next.
  Files: /Users/adminamn/ai-router/router-research/tasks/router_retrieval_eval.py, /Users/adminamn/ai-router/router-research/tools/translated_dense_gap_decomposition.py, /Users/adminamn/ai-router/router-research/tests/test_router_retrieval_eval.py, /Users/adminamn/ai-router/router-research/tests/test_cli_contract.py, /Users/adminamn/ai-router/router-research/tests/test_translated_dense_gap_decomposition.py, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_prewarm_confirm.json, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_confirm.json, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0114_product_phase_sparse_translation_upper_bank_dense_reference_selection.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 15:26:57 EDT [run] Ran the tracked INC-0113 prewarm/confirm packet and built the upper-bank dense gap-decomposition audit.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_prewarm_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_confirm.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0113_PRODUCT_PHASE_SPARSE_TRANSLATION_UPPER_BANK_DENSE_GAP_DECOMPOSITION.md
  Cmd: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_prewarm_confirm.json && python tools/proxy_sweep.py --config configs/proxy_transfer_inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_confirm.json && python tools/translated_dense_gap_decomposition.py --analysis results/analysis/inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition_confirm.json --compare upper_dense_vs_soft_sparse:... --compare upper_dense_vs_backfill:... --output-json results/analysis/inc0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition.json --output-report docs/reports/INC0113_PRODUCT_PHASE_SPARSE_TRANSLATION_UPPER_BANK_DENSE_GAP_DECOMPOSITION.md`
- 15:26:57 EDT [decision] Closed INC-0113 positive/explanatory: the residual upper-bank dense gap is operationally negligible on both fixed sparse translated points, omission is not the limiting mechanism, and upper-bank dense-gap rescue should leave the queue.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0113_product_phase_sparse_translation_upper_bank_dense_gap_decomposition.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 15:26:57 EDT [next] Queued INC-0114 next: select and freeze the explicit upper-bank dense-near routed reference from the completed INC-0112 and INC-0113 evidence instead of reopening rescue heuristics.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0114_product_phase_sparse_translation_upper_bank_dense_reference_selection.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 15:35:15 EDT [edit] Added the upper-bank reference-selection audit helper/tests, generated the INC-0114 selection artifact, and reconciled the source-of-truth docs through INC-0114 with INC-0115 queued next.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_upper_bank_reference_selection.py, /Users/adminamn/ai-router/router-research/tests/test_translated_upper_bank_reference_selection.py, /Users/adminamn/ai-router/router-research/results/analysis/inc0114_product_phase_sparse_translation_upper_bank_dense_reference_selection.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0114_PRODUCT_PHASE_SPARSE_TRANSLATION_UPPER_BANK_DENSE_REFERENCE_SELECTION.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0114_product_phase_sparse_translation_upper_bank_dense_reference_selection.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0115_product_phase_sparse_translation_promoted_upper_bank_carry_forward.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 15:35:15 EDT [decision] Closed INC-0114 positive/explanatory: the completed upper-bank pair now collapses to one promoted dense-near routed reference, with the bounded-backfill point demoted to comparator status.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0114_product_phase_sparse_translation_upper_bank_dense_reference_selection.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 15:35:15 EDT [next] Queued INC-0115 next: carry forward the promoted upper-bank dense-near routed reference as the sole upper-bank representative in later broader comparisons.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0115_product_phase_sparse_translation_promoted_upper_bank_carry_forward.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 16:06:49 EDT [edit] Added the promoted upper-bank carry-forward contract tool/tests, generated the INC-0115 contract artifact, and reconciled the source-of-truth docs through INC-0115 with INC-0116 queued next.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_promoted_carry_forward_contract.py, /Users/adminamn/ai-router/router-research/tests/test_translated_promoted_carry_forward_contract.py, /Users/adminamn/ai-router/router-research/results/analysis/inc0115_product_phase_sparse_translation_promoted_upper_bank_carry_forward.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0115_PRODUCT_PHASE_SPARSE_TRANSLATION_PROMOTED_UPPER_BANK_CARRY_FORWARD.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0115_product_phase_sparse_translation_promoted_upper_bank_carry_forward.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 16:06:49 EDT [decision] Closed INC-0115 positive/explanatory: the sparse translated branch now has a fixed dual-anchor default broader-comparison packet, with the lower-bank soft sparse route nondefault and the upper-bank bounded-backfill route comparator-only.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0115_product_phase_sparse_translation_promoted_upper_bank_carry_forward.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 16:06:49 EDT [next] Queued INC-0116 next: use the INC-0115 dual-anchor carry-forward packet as the default broader-comparison packet instead of reopening the old upper-bank pair or lower-bank pruning-only point.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 16:59:38 EDT [edit] Added the dual-anchor broader-packet generator/tests, generated the INC-0116 packet manifest, and reconciled the source-of-truth docs through INC-0116 with INC-0117 queued next.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_dual_anchor_broader_packet.py, /Users/adminamn/ai-router/router-research/tests/test_translated_dual_anchor_broader_packet.py, /Users/adminamn/ai-router/router-research/results/analysis/inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet.json, /Users/adminamn/ai-router/router-research/configs/packet_inc0116_product_phase_sparse_translation_dual_anchor_broader_comparison.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0116_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_BROADER_COMPARISON_PACKET.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0117_product_phase_sparse_translation_dual_anchor_broader_comparison.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md, /Users/adminamn/ai-router/router-research/results/INDEX.md
- 16:59:38 EDT [decision] Closed INC-0116 positive/explanatory: the sparse translated branch now has one reusable dual-anchor packet manifest with exact inherited route specs and explicit comparator/exclusion rules.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0116_product_phase_sparse_translation_dual_anchor_broader_comparison_packet.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 16:59:38 EDT [next] Queued INC-0117 next: start the first broader comparison branch from the fixed INC-0116 packet manifest instead of rebuilding sparse translated route forks by hand.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0117_product_phase_sparse_translation_dual_anchor_broader_comparison.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 17:08:37 EDT [edit] Added the dual-anchor broader-comparison audit helper/tests, generated the INC-0117 broader-comparison artifact, and reconciled the default broader sparse translated read.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_dual_anchor_broader_comparison.py, /Users/adminamn/ai-router/router-research/tests/test_translated_dual_anchor_broader_comparison.py, /Users/adminamn/ai-router/router-research/results/analysis/inc0117_product_phase_sparse_translation_dual_anchor_broader_comparison.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0117_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_BROADER_COMPARISON.md
- 17:08:37 EDT [decision] Closed INC-0117 positive/explanatory: lower-bank default stays systems-only, upper-bank default stays quality-near systems promotion, and the upper-bank bounded-backfill route remains optional comparator-only.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0117_product_phase_sparse_translation_dual_anchor_broader_comparison.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 17:08:37 EDT [next] Queued INC-0118 next: carry the fixed dual-anchor packet and broader sparse translated read onto the real-task side without reopening nondefault sparse translated routes.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0118_product_phase_sparse_translation_dual_anchor_task_side_extension.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 17:16:31 EDT [edit] Added the dual-anchor task-side extension helper/tests, generated the INC-0118 task-side extension artifact, and reconciled the real-task-side inheritance read.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_dual_anchor_task_side_extension.py, /Users/adminamn/ai-router/router-research/tests/test_translated_dual_anchor_task_side_extension.py, /Users/adminamn/ai-router/router-research/results/analysis/inc0118_product_phase_sparse_translation_dual_anchor_task_side_extension.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0118_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_TASK_SIDE_EXTENSION.md
- 17:16:31 EDT [decision] Closed INC-0118 positive/explanatory: the real-task side now inherits the exact dual-anchor packet, with lower bank systems-only by default, upper bank quality-near systems promotion by default, and the upper-bank bounded-backfill route remaining optional comparator-only.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0118_product_phase_sparse_translation_dual_anchor_task_side_extension.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 17:16:31 EDT [next] Queued INC-0119 next: start the next explicit real-task comparison from the completed dual-anchor task-side packet instead of rebuilding sparse translated route forks.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0119_product_phase_sparse_translation_dual_anchor_real_task_comparison.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 17:24:21 EDT [edit] Added the dual-anchor explicit real-task comparison helper/tests, generated the INC-0119 real-task comparison artifact, and reconciled the default LM-proxy comparison read.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_dual_anchor_real_task_comparison.py, /Users/adminamn/ai-router/router-research/tests/test_translated_dual_anchor_real_task_comparison.py, /Users/adminamn/ai-router/router-research/results/analysis/inc0119_product_phase_sparse_translation_dual_anchor_real_task_comparison.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0119_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_COMPARISON.md
- 17:24:21 EDT [decision] Closed INC-0119 positive/explanatory: the first explicit LM-proxy real-task comparison now inherits the exact dual-anchor packet, with lower bank systems-only by default, upper bank quality-near systems promotion by default, and the upper-bank bounded-backfill route remaining optional comparator-only.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0119_product_phase_sparse_translation_dual_anchor_real_task_comparison.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 17:24:21 EDT [next] Queued INC-0120 next: carry the explicit LM-proxy real-task comparison forward as the default downstream task-side reference instead of rebuilding sparse translated route forks.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0120_product_phase_sparse_translation_dual_anchor_real_task_carry_forward.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 17:30:22 EDT [edit] Added the dual-anchor real-task carry-forward helper/tests, generated the INC-0120 downstream carry-forward artifact, and reconciled the downstream real-task default read.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_dual_anchor_real_task_carry_forward.py, /Users/adminamn/ai-router/router-research/tests/test_translated_dual_anchor_real_task_carry_forward.py, /Users/adminamn/ai-router/router-research/results/analysis/inc0120_product_phase_sparse_translation_dual_anchor_real_task_carry_forward.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0120_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_CARRY_FORWARD.md
- 17:30:22 EDT [decision] Closed INC-0120 positive/explanatory: downstream real-task work now inherits the explicit lower-bank systems-only default and upper-bank promoted real-task default, with the upper-bank bounded-backfill route remaining optional comparator-only.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0120_product_phase_sparse_translation_dual_anchor_real_task_carry_forward.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 17:30:22 EDT [next] Queued INC-0121 next: convert the completed downstream real-task carry-forward contract into one reusable downstream packet manifest instead of rebuilding sparse translated route forks.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0121_product_phase_sparse_translation_dual_anchor_real_task_packet_manifest.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 17:56:32 EDT [edit] Added the downstream real-task packet-manifest helper/tests, generated the INC-0121 packet-manifest artifact, and reconciled the exact downstream packet inheritance set.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_dual_anchor_real_task_packet_manifest.py, /Users/adminamn/ai-router/router-research/tests/test_translated_dual_anchor_real_task_packet_manifest.py, /Users/adminamn/ai-router/router-research/results/analysis/inc0121_product_phase_sparse_translation_dual_anchor_real_task_packet_manifest.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0121_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_PACKET_MANIFEST.md
- 17:56:32 EDT [decision] Closed INC-0121 positive/explanatory: downstream real-task work now has one exact reusable packet manifest, with default lower-bank systems-only and upper-bank promoted real-task routes, and the upper-bank bounded-backfill route remaining optional comparator-only.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0121_product_phase_sparse_translation_dual_anchor_real_task_packet_manifest.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 17:56:32 EDT [next] Queued INC-0122 next: use the completed downstream real-task packet manifest on the next downstream real-task question instead of rebuilding sparse translated route forks.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0122_product_phase_sparse_translation_dual_anchor_real_task_downstream_extension.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 18:01:38 EDT [edit] Added the downstream real-task extension helper/tests, generated the INC-0122 downstream extension artifact, and reconciled the manifest-backed downstream inheritance read.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_dual_anchor_real_task_downstream_extension.py, /Users/adminamn/ai-router/router-research/tests/test_translated_dual_anchor_real_task_downstream_extension.py, /Users/adminamn/ai-router/router-research/results/analysis/inc0122_product_phase_sparse_translation_dual_anchor_real_task_downstream_extension.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0122_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_DOWNSTREAM_EXTENSION.md
- 18:01:38 EDT [decision] Closed INC-0122 positive/explanatory: downstream real-task inheritance is now explicit from the exact packet manifest, with lower-bank systems-only default, upper-bank promoted real-task default, and the upper-bank bounded-backfill route remaining optional comparator-only.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0122_product_phase_sparse_translation_dual_anchor_real_task_downstream_extension.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 18:01:38 EDT [next] Queued INC-0123 next: carry the completed downstream extension into the next explicit downstream real-task comparison instead of rebuilding sparse translated route forks.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0123_product_phase_sparse_translation_dual_anchor_real_task_downstream_comparison.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 18:09:32 EDT [edit] Added the downstream real-task comparison helper/tests, generated the INC-0123 downstream comparison artifact, and reconciled the explicit downstream comparison read.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_dual_anchor_real_task_downstream_comparison.py, /Users/adminamn/ai-router/router-research/tests/test_translated_dual_anchor_real_task_downstream_comparison.py, /Users/adminamn/ai-router/router-research/results/analysis/inc0123_product_phase_sparse_translation_dual_anchor_real_task_downstream_comparison.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0123_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_DOWNSTREAM_COMPARISON.md
- 18:09:32 EDT [decision] Closed INC-0123 positive/explanatory: the explicit downstream LM-proxy real-task comparison now inherits the exact downstream extension artifact, with lower-bank systems-only default, upper-bank promoted real-task default, and the upper-bank bounded-backfill route remaining optional comparator-only.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0123_product_phase_sparse_translation_dual_anchor_real_task_downstream_comparison.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 18:09:32 EDT [next] Queued INC-0124 next: carry the completed downstream comparison into the downstream carry-forward branch instead of rebuilding sparse translated route forks.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0124_product_phase_sparse_translation_dual_anchor_real_task_downstream_carry_forward.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 18:14:16 EDT [edit] Added the downstream real-task carry-forward helper/tests, generated the INC-0124 downstream carry-forward artifact, and reconciled the explicit downstream carry-forward contract.
  Files: /Users/adminamn/ai-router/router-research/tools/translated_dual_anchor_real_task_downstream_carry_forward.py, /Users/adminamn/ai-router/router-research/tests/test_translated_dual_anchor_real_task_downstream_carry_forward.py, /Users/adminamn/ai-router/router-research/results/analysis/inc0124_product_phase_sparse_translation_dual_anchor_real_task_downstream_carry_forward.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0124_PRODUCT_PHASE_SPARSE_TRANSLATION_DUAL_ANCHOR_REAL_TASK_DOWNSTREAM_CARRY_FORWARD.md
- 18:14:16 EDT [decision] Closed INC-0124 positive/explanatory: downstream real-task work now inherits the explicit downstream lower-bank systems-only default and upper-bank promoted default, with the upper-bank bounded-backfill route remaining optional comparator-only.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0124_product_phase_sparse_translation_dual_anchor_real_task_downstream_carry_forward.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 18:14:16 EDT [next] Queued INC-0125 next: convert the completed downstream real-task carry-forward contract into one reusable downstream packet manifest instead of rebuilding sparse translated route forks.
  Files: /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md
- 18:51:13 EDT [decision] Pivoted away from the downstream packet-manifest loop and redirected `INC-0125` to sparse-event proxy trainability hardening, using the root kill-list docs as source of truth.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0125_product_phase_sparse_event_proxy_trainability_hardening.md, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0125_product_phase_sparse_event_proxy_trainability_hardening_screen.json, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0125_product_phase_sparse_event_proxy_trainability_hardening_confirm.json
- 18:51:13 EDT [run] Ran the tracked `INC-0125` sparse-event proxy hardening screen and confirm on the fixed product route law.
  Files: /Users/adminamn/ai-router/router-research/results/analysis/inc0125_product_phase_sparse_event_proxy_trainability_hardening_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0125_product_phase_sparse_event_proxy_trainability_hardening_confirm.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260312_184338.md, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260312_184913.md
- 18:51:13 EDT [decision] Closed `INC-0125` positive/explanatory: near-hard sparse-event proxy trainability survives the harder load, the true hard gate remains mostly-on, and the next honest branch is the proxy/translation gap.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0125_product_phase_sparse_event_proxy_trainability_hardening.md, /Users/adminamn/ai-router/router-research/docs/reports/INC0125_PRODUCT_PHASE_SPARSE_EVENT_PROXY_TRAINABILITY_HARDENING.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md
- 18:51:13 EDT [next] Queued `INC-0126` next: explain the gap between hardened near-hard proxy trainability and the older translated near-hard failure before reopening translated sparse-event work.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0126_product_phase_sparse_event_proxy_translation_gap_audit.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md
- 19:03:51 EDT [edit] Added the sparse-event proxy/translation gap audit tool and generated the `INC-0126` analysis plus report.
  Files: /Users/adminamn/ai-router/router-research/tools/sparse_event_proxy_translation_gap_audit.py, /Users/adminamn/ai-router/router-research/tests/test_sparse_event_proxy_translation_gap_audit.py, /Users/adminamn/ai-router/router-research/results/analysis/inc0126_product_phase_sparse_event_proxy_translation_gap_audit.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0126_PRODUCT_PHASE_SPARSE_EVENT_PROXY_TRANSLATION_GAP_AUDIT.md
- 19:03:51 EDT [decision] Closed `INC-0126` positive/explanatory: translated near-hard preserves quality and candidate fraction, and the failure is systems-cost-only with retrieval search as the dominant driver.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0126_product_phase_sparse_event_proxy_translation_gap_audit.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md
- 19:03:51 EDT [next] Queued `INC-0127` next: reopen translated near-hard only as a narrow systems-cost rescue surface on the fixed route law.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0127_product_phase_sparse_event_translation_systems_cost_rescue.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md
- 19:17:25 EDT [edit] Added the sparse-event translated rescue-feasibility audit, made the retrieval harness explicit that sparse-event knobs are audit-only downstream, and generated the `INC-0127` analysis plus report.
  Files: /Users/adminamn/ai-router/router-research/tasks/router_retrieval_eval.py, /Users/adminamn/ai-router/router-research/tools/sparse_event_translation_systems_cost_rescue.py, /Users/adminamn/ai-router/router-research/tests/test_router_retrieval_eval.py, /Users/adminamn/ai-router/router-research/tests/test_sparse_event_translation_systems_cost_rescue.py, /Users/adminamn/ai-router/router-research/results/analysis/inc0127_product_phase_sparse_event_translation_systems_cost_rescue.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0127_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_SYSTEMS_COST_RESCUE.md
- 19:17:25 EDT [decision] Closed `INC-0127` negative/explanatory: translated soft sparse and translated near-hard differ only on `event_gate_tau`, and the current translated harness does not let sparse-event knobs change the retrieval surface.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0127_product_phase_sparse_event_translation_systems_cost_rescue.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md
- 19:17:25 EDT [next] Queued `INC-0128` next: route-couple sparse-event behavior into the translated path so downstream sparse-event work stops relying on an audit-only surface.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0128_product_phase_sparse_event_translation_route_coupled_pilot.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/research/OPEN_QUESTIONS.md
- 19:35:27 EDT [edit] Added translated train-gate pruning to the retrieval harness, fixed the post-prune label-coherence shape bug, reran the `INC-0128` screen, and wrote the route-coupled screen report.
  Files: /Users/adminamn/ai-router/router-research/tasks/router_retrieval_eval.py, /Users/adminamn/ai-router/router-research/tests/test_router_retrieval_eval.py, /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0128_product_phase_sparse_event_translation_route_coupled_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0128_product_phase_sparse_event_translation_route_coupled_screen.json, /Users/adminamn/ai-router/router-research/docs/reports/INC0128_PRODUCT_PHASE_SPARSE_EVENT_TRANSLATION_ROUTE_COUPLED_SCREEN.md
- 19:35:27 EDT [decision] Closed `INC-0128` positive/explanatory: translated sparse-event behavior is now downstream-live under train-gate pruning, but soft sparse at threshold `0.02` stayed inert and near-hard at threshold `0.02` collapsed top-1.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0128_product_phase_sparse_event_translation_route_coupled_pilot.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md
- 19:35:27 EDT [next] Queued `INC-0129` next: map the first quality-preserving threshold window on the same translated train-gate-prune coupling, or close that coupling law negative if none exists.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0129_product_phase_sparse_event_translation_route_coupled_threshold_map.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/research/OPEN_QUESTIONS.md
- 19:45:44 EDT [run] Ran the focused lower-bank translated train-gate-prune threshold-map screen.
  Files: /Users/adminamn/ai-router/router-research/configs/proxy_transfer_inc0129_product_phase_sparse_event_translation_route_coupled_threshold_map_screen.json, /Users/adminamn/ai-router/router-research/results/analysis/inc0129_product_phase_sparse_event_translation_route_coupled_threshold_map_screen.json, /Users/adminamn/ai-router/router-research/docs/governance/gates/gate_20260312_194544.md
- 19:45:44 EDT [decision] Closed `INC-0129` negative/explanatory: no train-gate-prune threshold window is promotable, because tiny pruning preserves quality but worsens runtime while stronger pruning collapses translated top-1.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0129_product_phase_sparse_event_translation_route_coupled_threshold_map.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md
- 19:45:44 EDT [next] Queued `INC-0130` next: try a softer translated coupling that can change downstream behavior without deleting train items.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0130_product_phase_sparse_event_translation_route_coupled_soft_bias_pilot.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/research/OPEN_QUESTIONS.md
- 20:14:30 EDT [edit] Added translated soft score-bias coupling plus lower-bank carry-forward configs and docs for INC-0130/0131.
- 20:14:30 EDT [result] Closed INC-0130 positive/explanatory at screen and INC-0131 positive/explanatory at confirm: TAU002 is lower-bank systems reference, SBI030 is balanced quality comparator, SBI080 is quality-first comparator.
- 20:14:30 EDT [next] Queue INC-0132 next: lower-bank sparse-event translated reference reselection using completed INC-0104/0130/0131 evidence.
- 20:23:44 EDT [edit] Added lower-bank sparse-event translated reference-selection audit tooling and tests for INC-0132.
- 20:23:44 EDT [result] Closed INC-0132 positive/explanatory: TAU002 is explicit lower-bank default, SBI030 is balanced quality comparator, SBI080 is quality-first comparator, and the old BF2 lower-bank default is stale historical-only.
- 20:23:44 EDT [next] Queue INC-0133 next: refresh broader/task-side/downstream lower-bank contracts to inherit the new TAU002 default once.
- 20:34:24 EDT [edit] Completed INC-0133 lower-bank contract refresh helper, report, and live doc updates; active lower-bank default is now TAU002 across broader/task-side/downstream surfaces
- 20:34:24 EDT [next] Queue INC-0134 to run one refreshed dual-anchor real-task comparison from the new TAU002 lower-bank contract instead of more contract-only packaging
- 20:41:37 EDT [edit] Completed INC-0134 refreshed dual-anchor real-task comparison from INC-0133 contract plus lower/upper confirm artifacts; TAU002 remains lower-bank default, SBI030 balanced comparator, SBI080 quality-first comparator
- 20:41:37 EDT [next] Queue INC-0135 to resolve whether the lower-bank TAU002/SBI030/SBI080 split is final or whether one comparator deserves a named promoted operating mode
- 20:57:15 EDT [edit] Added CORE_PROJECT_GOALS and rewired startup/bootstrap/instruction docs so future branch selection must be justified against the moonshot goal and kill list before any new work starts
- 20:57:15 EDT [next] On any future resume or compaction recovery: run context_bootstrap startup, read CORE_PROJECT_GOALS, then audit proposed next branch against the kill list before continuing
- 21:02:33 EDT [read] Re-audited the root theory corpus and live queue before continuing work.
  Files: /Users/adminamn/ai-router/router-research/CORE_PROJECT_GOALS.md, /Users/adminamn/ai-router/router-research/docs/PROJECT_CONTEXT.md, /Users/adminamn/ai-router/router-research/geometric_routing_kill_tests.md, /Users/adminamn/ai-router/router-research/NEXT_CRITICAL_EXPERIMENTS.md, /Users/adminamn/ai-router/router-research/EVIDENCE_SUMMARY.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md
- 21:02:33 EDT [decision] The translated sparse-event lower-bank frontier is valid supporting evidence, but it drifted too far downstream of the main unresolved geometry gate. `RR-061` remains the next primary proof step.
- 21:02:33 EDT [edit] Reset the live queue docs to defer `INC-0135` and queue `INC-0136` as the geometry-return branch on the measure-consistent `H^4` / Hopf route law.
  Files: /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0135_product_phase_sparse_event_translation_lower_bank_quality_systems_frontier.md, /Users/adminamn/ai-router/router-research/docs/research/increments/INC_0136_measure_consistent_h4_hopf_route_return.md, /Users/adminamn/ai-router/router-research/docs/research/CURRENT_DIRECTION.md, /Users/adminamn/ai-router/router-research/docs/research/HANDOFF_CURRENT.md, /Users/adminamn/ai-router/router-research/docs/research/LIVE_WORKLOG.md, /Users/adminamn/ai-router/router-research/docs/routes/ROUTE_MATRIX.md, /Users/adminamn/ai-router/router-research/docs/program/PROJECT_BOARD.md, /Users/adminamn/ai-router/router-research/docs/program/ISSUE_REGISTRY.md, /Users/adminamn/ai-router/router-research/docs/DECISIONS.md
- 21:02:33 EDT [next] Resume from `INC-0136`, not `INC-0135`. Re-open the measure-consistent shell/Hopf route law before more translated frontier refinement.
- 21:02:33 EDT [edit] Added canonical state docs and a validator so the repo can mechanically reject queue drift.
  Files: /Users/adminamn/ai-router/router-research/docs/research/KILL_LIST_TRACKER.md, /Users/adminamn/ai-router/router-research/docs/research/ACTIVE_STATE.md, /Users/adminamn/ai-router/router-research/docs/research/SUPPORTING_EVIDENCE.md, /Users/adminamn/ai-router/router-research/tools/check_research_state.py, /Users/adminamn/ai-router/router-research/tests/test_check_research_state.py, /Users/adminamn/ai-router/router-research/tools/context_bootstrap.py
- 21:02:33 EDT [result] Canonical-state hardening completed: startup bootstrap now includes mission, kill-list, active state, and supporting evidence, while the validator checks RR/INC agreement and branch metadata.
- 21:02:33 EDT [next] On every future resume: run `python tools/context_bootstrap.py --group startup --cat`, then `python tools/check_research_state.py`, then continue only if the canonical docs agree.

---
## Session Entry: 2026-03-13 — INC-0137 screen executed and closed

### Files read
- `docs/research/increments/INC_0137_measure_consistent_h4_hopf_shell_pressure_blend.md`
- `docs/research/increments/INC_0136_measure_consistent_h4_hopf_route_return.md`
- `results/analysis/inc0136_measure_consistent_h4_hopf_route_return_screen.json`
- `configs/proxy_transfer_inc0136_measure_consistent_h4_hopf_route_return_screen.json`
- `hyperbolic_router_so8.py` (route_addresses function, lines ~2880–3060)
- `tasks/router_proxy_eval.py` (argument list and route_addresses call sites)
- `docs/research/KILL_LIST_TRACKER.md`
- `docs/research/ACTIVE_STATE.md`

### Commands run
- Syntax check: `/usr/bin/python3 -m py_compile hyperbolic_router_so8.py` → OK
- Experiment: `python tools/proxy_sweep.py --config configs/proxy_transfer_inc0137_..._screen.json --python_bin /usr/bin/python3`

### Edits made
- `hyperbolic_router_so8.py`: added `shell_pressure_w: float = 0.0` parameter to `route_addresses(); implemented `r_eff = (1-w)*r_chart + w*r_geodesic` blend for `sector_mode="phase4d_hopf_base"`
- `tasks/router_proxy_eval.py`: added `--shell_pressure_w` CLI arg; threaded through both `route_addresses` call sites
- Created `configs/proxy_transfer_inc0137_..._screen.json` (8 routes: 3 baselines + SPW_01–04 blend points at w=0.1/0.2/0.3/0.4 + R0)
- Updated INC-0137 doc status to `Closed: KILL`
- Updated `KILL_LIST_TRACKER.md` queue → INC-0138
- Updated `ACTIVE_STATE.md` queue → INC-0138

### Results
| Route               | Health | shell_pmax | knn_overlap |
|---------------------|--------|------------|-------------|
| HOPF_BASE_K25_PHI   | PASS   | 0.5222     | 0.6738      |
| SPW_01 (w=0.1)      | PASS   | 0.7464     | 0.6738      |
| SPW_02 (w=0.2)      | FAIL   | 0.9914     | —           |
| SPW_03–04 (w≥0.3)   | FAIL   | —          | — (1 shell) |

### Conclusion
Falsification condition met. Radius-interpolation blend is not a viable lever for shell mass
concentration. knn_overlap is invariant across blend weights; shell_pmax strictly worsens.

### Cross-stage observation
The Poincaré-ball geodesic radius is more origin-concentrated than the chart radius. Any
positive blend weight pushes tokens into the innermost shell. This mechanistically explains
why direct geodesic substitution (INC-0136) and blend (INC-0137) both fail in the same
direction.

### Next actions
1. Create `INC-0138`: shell-density controller / occupancy equalizer
   - Equalize shell occupancy via post-assignment feedback rather than radius modification
   - Or: regularize chart radius to reduce radial concentration
2. Update GitHub Issue #1 (RR-061) with KILL decision for INC-0137
3. Commit all changes to `codex/RR-061-measure-consistent-route-law` and push

## Session Entry: 2026-03-13 — INC-0138 screen executed (geometry-only controls)

### Context
Active gate: Stage 2 — Measure-Consistent Shell Routing
Branch: codex/RR-061-measure-consistent-route-law
Task: run geometry-only shell activation experiment with real vs destructive controls

### Files Read
- docs/research/ACTIVE_STATE.md
- docs/research/KILL_LIST_TRACKER.md
- tasks/router_proxy_eval.py (structure of data loading, arg list, output metrics)
- configs/proxy_transfer_inc0137*.json (config format reference)
- results/analysis/inc0138_geometry_only_shell_activation_screen.json (results)

### Commands Run
- Added `--input_transform` flag (none/col_perm/gaussian) to tasks/router_proxy_eval.py
- Created configs/proxy_transfer_inc0138_geometry_only_shell_activation_screen.json
- Ran /opt/homebrew/Caskroom/miniforge/base/bin/python3 tools/proxy_sweep.py --config configs/proxy_transfer_inc0138_geometry_only_shell_activation_screen.json --python_bin /opt/homebrew/Caskroom/miniforge/base/bin/python3
- 8 runs completed (4 routes x 2 seeds), all parsed, rc=0

### Key Results
| Route | eval_shells | shell_pmax | pmax_after | buckets | sector_entropy |
|---|---|---|---|---|---|
| GEOM_ORIG (original) | 2 | 0.584 | 0.529 | 15.5 | 1.322 |
| GEOM_COL_PERM | 2 | 0.532 | 0.422 | 29.0 | 1.323 |
| GEOM_GAUSSIAN | 2 | 0.645 | 0.048 | 50.0 | 3.195 |
| R0 (learned) | 1 | 1.000 | 0.261 | 8.0 | 1.953 |

### Conclusion
Fixed geometry + adaptive shell activation produces stable 2-shell structure.
Real embeddings separate from Gaussian noise strongly at bucket/sector level.
Shell level does NOT discriminate real from col-perm (norm-driven, not semantic).
Primary carrier of semantic structure is the Hopf-base angular/sector dimension.
INC-0138 closed: REFINE — structure confirmed but shell indistinguishability finding
requires a decision in INC-0139.

### Cross-Stage Observation
The Hopf-base angular dimension (sector_entropy, pmax_after, buckets) is primary
carrier of semantic structure. This strengthens the case for Stage 3 (Hopf Angular
Correctness) being the higher-value target once Stage 2 decision is documented.

### Next Actions
1. Create INC-0139: decide whether shell law can discriminate real from col-perm
   or formally document norm-driven shell as a Stage 2 constraint
2. Update GitHub Issue #1 with INC-0138 REFINE result
3. Commit and push

## Session Entry: 2026-03-13 — Post-INC-0138 structural finding: r ≡ 1 on unit-norm embeddings

### Structural Issue Identified
Research leader observation: embeddings appear L2-normalized, so if shells depend
on radial magnitude, r ≈ constant and shells cannot form without another driver.

### Verified empirically (diagnostic: /tmp/inc0138_hopf_radial_diagnostic.py)
- Embedding norms: mean=1.0, std=2.7e-8 — confirmed L2-normalized
- Chart radius r=safe_norm(route_z): mean=1.0, std=0.0 — degenerate with identity chart
- With adaptive_shell_growth=0.0: shell_multiplier=1.0 for all, 1 shell, shell_pmax=1.0
- With adaptive_shell_growth=1.6: shell_multiplier varies (std=0.414, range 1.96-2.85),
  2 shells, shell_pmax=0.595

### Mechanism identified
Shell_multiplier = exp(growth * div_score * (1 + balance_weight * |balance|) - converge)
balance = (rho1-rho2)/(rho1+rho2)   [Hopf fiber energy asymmetry]
rho1 = ||(z[0], z[2])||,  rho2 = ||(z[4], z[6])||

Hopf balance DOES vary significantly: mean=0.066, std=0.621, range [-1, +1]
Shells form because Hopf fiber balance is heterogeneous — not because r varies.

### Why col-perm is indistinguishable at shell level
Column permutation preserves each column's marginal distribution.
balance = f(||col_pair_1||, ||col_pair_2||) depends only on marginal norms.
Hence: col-perm balance distribution ≈ original balance distribution → same shells.

### Restated Stage 2 question
Not "does H^4 radius vary?" (it doesn't on unit-norm embeddings).
But: "Can chart rotation (SO(8) learning) concentrate Hopf fiber balance in ways
that make shell assignment semantically discriminating?"
Test: run with learn_so8=1 and compare balance+shell distributions for GEOM_ORIG vs GEOM_COL_PERM.

### Updated INC-0139
INC-0139 now tests whether SO(8) chart learning creates semantically meaningful
fiber balance variation, or whether the structural constraint is fundamental.
INC-0138 doc updated with this structural finding.

### Next Actions
1. Update GitHub Issue #1 with this corrected Stage 2 framing
2. Commit updated INC-0138 doc and INC-0139
3. Proceed to INC-0139 experiment

## Session Entry: 2026-03-13 — INC-0139 Closed: REFINE — fiber balance path exhausted

### Experiment Run
- Config: `configs/proxy_transfer_inc0139_so8_fiber_balance_screen.json`
- Routes: GEOM_ORIG, GEOM_COL_PERM, LEARN_ORIG, LEARN_COL_PERM
- Seeds: 0, 1 — 8 total runs

### Results (2-seed means)
| Route | learn_so8 | shells | shell_pmax | pmax_after | buckets | sec_ent |
|---|---|---|---|---|---|---|
| GEOM_ORIG | 0 | 2.00 | 0.5846 | 0.5284 | 15.5 | 1.3220 |
| GEOM_COL_PERM | 0 | 2.00 | 0.5320 | 0.4222 | 29.0 | 1.3231 |
| LEARN_ORIG | 1 | 2.00 | 0.7848 | 0.0966 | 45.0 | 3.0706 |
| LEARN_COL_PERM | 1 | 2.50 | 0.7226 | 0.0992 | 50.5 | 2.9913 |

### Decision
- Technical criterion: |LEARN_ORIG - LEARN_COL_PERM| shell_pmax = 0.0622 > 0.05 → passes
- Mechanism is degenerate: SO(8) increases shell_pmax for BOTH inputs by +0.19 (generic
  concentration). pmax_after collapses from ~0.50 to ~0.10 (routing quality destroyed).
  Incremental improvement over identity chart: 0.0622 - 0.0526 = 0.0096 (noise level).
- REFINE: fiber balance + SO(8) path is exhausted.

### Stage 2 Redirect
Angular sector routing (Hopf base delta/chi/theta) shows meaningful real vs Gaussian
signal (INC-0138: buckets=15.5 vs 50.0). INC-0140 formally tests whether sector
routing is measure-consistent with H^4 angular measure.

### Files Updated
- `docs/research/increments/INC_0139_TBD.md` → Closed: REFINE
- `docs/research/increments/INC_0140_angular_sector_routing_measure_consistency.md` → created
- `docs/research/ACTIVE_STATE.md` → current INC → INC-0140
- `docs/research/KILL_LIST_TRACKER.md` → Stage 2 blocker updated
- `docs/routes/ROUTE_MATRIX.md`, `docs/program/PROJECT_BOARD.md`, `docs/program/ISSUE_REGISTRY.md`
- `docs/DECISIONS.md` → INC-0139 decision logged

### Next Actions
1. make state → must pass
2. Commit all docs (INC-0139, INC-0140, state docs)
3. Push branch
4. Update GitHub Issue #1 with INC-0139 REFINE verdict + Stage 2 redirect

---

## Session: INC-0168 Norm-Geometry Diagnostic (2026-03-14)

### Summary
INC-0168 tested whether the √K routing scaling is angular-only or norm-dependent.
Three experiments (A: L2 baseline, B: radial-aware via L1-normalization, C: L3/L4 surface).

### Key Result
TRANS ORIG α=0.572 is norm-invariant across all five variants (max Δα < 0.015).
Shell activation (B2) does NOT improve ORIG advantage: Gini ratio drops 1.836 → 1.486.
Routing sparsity is purely angular — a direct property of Hopf-base sector discretization.

### Verdict: KEEP

### Files Updated
- `docs/research/increments/INC_0168_norm_geometry_angular_vs_radial.md` → created, Closed: KEEP
- `docs/research/ACTIVE_STATE.md` → current INC → INC-0169 (TBD)
- `docs/research/KILL_LIST_TRACKER.md` → Stage 7 latest result updated to INC-0168
- `docs/DECISIONS.md` → INC-0168 decision logged
- `results/analysis/inc0168_norm_geometry.json` → 160 routing runs
- `_inc0168_analysis.py` → diagnostic script

### Cross-stage observation
Shell activation (B2) shows that when shells ARE active, angular concentration
(Gini ratio) decreases. This is a safeguard for Stage 2/3: if future embedding
proxies introduce non-unit L2 norms, shell splitting could dilute rather than
amplify routing concentration. This does not change current PARTIAL-PASS verdicts.

---

## Session: INC-0169 Canonical Law Freeze (2026-03-14)

### Summary
INC-0169 is a synthesis increment. No new experiments run. Canonical routing law
frozen from INC-0162 through INC-0168 evidence. Design implications derived.

### Key Result
Canonical law: eff_buckets = 2.957 × K^0.572 (TRANS ORIG, static, L2-normalized).
Law is norm-invariant (Δα < 0.015 across L1/L2/L3/L4). Shell hierarchy excluded.
Hardware consequence: 3.0–4.9× eff_cost reduction, 2.5–2.9× fewer LRU misses (TRANS).
INC-0170 proposal: Large-K capacity test, K=1000–5000 (predicted eff_ratio ~4.4× at K=5000).

### Verdict: KEEP

### Files Updated
- `docs/research/increments/INC_0169_canonical_law_freeze_design_implications.md` → created, Closed: KEEP
- `docs/research/ACTIVE_STATE.md` → current INC → INC-0170 (TBD)
- `docs/research/KILL_LIST_TRACKER.md` → Stage 7 latest result updated to INC-0169
- `docs/DECISIONS.md` → INC-0169 decision logged
- `docs/routes/ROUTE_MATRIX.md` → updated to INC-0170
- `docs/program/PROJECT_BOARD.md` → updated to INC-0170
- `docs/program/ISSUE_REGISTRY.md` → updated to INC-0170

---

## Session Entry: 2026-04-05 — Prime Transport Interaction Statistics

### Scope
- stayed entirely inside the exact recursive-system layer
- no quotient targets, no runtime router changes, no MUDBench changes
- tested whether visible threshold is better organized by interaction between
  inherited parent return grammar and the new local stencil at the lift prime

### Files Added / Updated
- `docs/research/prime_transport_interaction_statistics.md` → created
- `tools/prime_transport/analyze_interaction_statistics.py` → executed
- `results/prime_transport_recursive_system/visible_threshold_interaction_stats.csv` → created
- `results/prime_transport_recursive_system/visible_threshold_interaction_scores.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Interaction Statistics Tested
- `weighted_gap_forbidden_hit_rate`
- `weighted_gap_mean_forbidden_distance`
- `largest_gap_forbidden_distance`
- `most_common_gap_forbidden_distance`

These compare exact parent return-gap residues mod `p` against the forbidden
residue set of the new stencil.

### Main Result
- strongest interaction-sensitive score:
  `weighted_gap_mean_forbidden_distance` with Spearman `0.17411086930415728`
- strongest previously-established coarse controls remain much larger:
  - `new_prime`: `0.8125966644585016`
  - `parent_admissible_density`: `-0.7346753721413163`
  - `local_allowed_count`: `0.7213299617065603`
  - `gap_max`: `0.5951493353290387`

### Interpretation
- the first global parent-return/stencil alignment summaries are too weak to
  serve as the leading visibility law
- this does not rule out the broader parent-grammar / new-stencil interaction
  law class
- it only rules out this first coarse family of interaction proxies as the main
  explanatory object

### Next Exact-Layer Step
1. identify which parent return-gap classes are split first under each lift
2. test a first-split statistic tied to those classes
3. compare it against `p`, parent density, and `gap_max`

---

## Session Entry: 2026-04-05 — Prime Transport First Splitting Classes

### Scope
- stayed entirely inside the exact recursive-system layer
- no quotient targets, no runtime router changes, no MUDBench changes
- moved from global interaction averages to event-level splitting at the exact
  visible threshold

### Files Added / Updated
- `docs/research/prime_transport_first_splitting_classes.md` → created
- `tools/prime_transport/analyze_first_splitting_classes.py` → created and executed
- `results/prime_transport_recursive_system/visible_threshold_first_splitting_classes.csv` → created
- `results/prime_transport_recursive_system/visible_threshold_first_splitting_event_stats.csv` → created
- `results/prime_transport_recursive_system/visible_threshold_first_splitting_scores.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Exact Event Definition
- for a row with exact `H_visible_first = H* > 1`, the pre-threshold parent
  classes are `(b, spin_{H*-1})`
- a class is a first splitting class if its descendants in the child wheel
  produce more than one distinct child predictive class at horizon `H*`

### Main Result
- strongest event-based statistics:
  - `num_first_splitting_classes`: Spearman `0.820410093133918`
  - `total_extra_child_classes_from_first_split`: `0.7982368473735418`
  - `fraction_first_splitting_classes`: `0.7464992739326641`
- best earlier global interaction summary was only:
  - `weighted_gap_mean_forbidden_distance`: `0.17411086930415728`
- strongest control on the current event table:
  - parent admissible density: `-0.8593693199561495`

### Interpretation
- first-splitting-event structure is a substantially better target than the
  earlier global interaction averages
- the visible threshold appears to come from a broad release of unresolved
  parent predictive classes, not from a single rare exceptional class
- parent density still slightly outperforms the tested first-splitting
  statistics, so the event-law class is plausible but not yet complete

### Next Exact-Layer Step
1. condition on parent admissible density
2. compare rows with similar density but different first-splitting multiplicity
3. test whether first-splitting statistics explain residual threshold
   variation after that conditioning

---

## Session Entry: 2026-04-05 — Prime Transport Density-Conditioned First Splitting

### Scope
- stayed entirely inside the exact recursive-system layer
- no quotient targets, no runtime router changes, no MUDBench changes
- tested whether first-splitting-event multiplicity still organizes visible
  threshold after coarse conditioning on parent admissible density

### Files Added / Updated
- `docs/research/prime_transport_density_conditioned_first_splitting.md` → created
- `tools/prime_transport/analyze_density_conditioned_first_splitting.py` → created and executed
- `results/prime_transport_recursive_system/visible_threshold_density_conditioned_rows.csv` → created
- `results/prime_transport_recursive_system/visible_threshold_density_conditioned_band_scores.csv` → created
- `results/prime_transport_recursive_system/visible_threshold_density_conditioned_residual_scores.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Conditioning Scheme
- three coarse density bands:
  - `low_density_le_0p08`
  - `mid_density_0p08_to_0p18`
  - `high_density_gt_0p18`
- no extra rows added in this step

### Main Result
- pooled within-band residual ranking:
  - `num_first_splitting_classes_residual`: Spearman `0.8144338729465652`
  - `total_extra_child_classes_residual`: `0.7879242712955016`
  - `fraction_first_splitting_classes_residual`: `0.7776149817645324`
- so first-splitting multiplicity still matters after coarse density
  conditioning

### Interpretation
- visible threshold is now best described by a density-plus-first-splitting
  event law class
- this is stronger than a density-only reading and stronger than the earlier
  global interaction-average line
- the bands remain small, so this is a narrowing result rather than a finished
  formula

### Next Exact-Layer Step
1. add a very small number of exact rows chosen to tighten density-matched
   contrasts
2. keep parent density roughly fixed
3. vary first-splitting multiplicity and split mass
4. test whether event statistics remain stronger than `gap_max` under tighter
   conditioning

---

## Session Entry: 2026-04-05 — Prime Transport Tight Density-Matched First Splitting

### Scope
- stayed entirely inside the exact recursive-system layer
- no quotient targets, no runtime router changes, no MUDBench changes
- tightened density matching by adding a single small exact family on
  `2310 -> 30030`

### Files Added / Updated
- `docs/research/prime_transport_tight_density_matched_first_splitting.md` → created
- `tools/prime_transport/run_tight_density_matched_first_split.py` → created and executed
- `results/prime_transport_recursive_system/visible_threshold_tight_density_matched_rows.csv` → created
- `results/prime_transport_recursive_system/visible_threshold_tight_density_matched_first_split_classes.csv` → created
- `results/prime_transport_recursive_system/visible_threshold_tight_density_matched_scores.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Matched Family
- exact lift: `2310 -> 30030`
- exact parent density for all six rows:
  `0.05454545454545454`
- tuplets:
  - `quadruplet`
  - `double_twins_p17`
  - `quad_alt_0618`
  - `quad_alt_0826`
  - `quad_alt_1836`
  - `quad_alt_3854`

### Main Result
- within this exact density-matched family:
  - `num_first_splitting_classes`: Spearman `0.9710083124552245`
  - `total_extra_child_classes_from_first_split`: `0.9710083124552245`
  - `fraction_first_splitting_classes`: `0.9710083124552245`
  - `gap_max`: `0.2537596094612761`
- visible thresholds span `6, 16, 21, 26` while density is constant

### Interpretation
- first-splitting multiplicity remains much stronger than `gap_max` under tight
  density matching
- the best current exact-layer law class is now
  `density + first-splitting event`
- no leading `gap_max` correction is earned on the present matched-family
  evidence

### Next Exact-Layer Step
1. build one more tiny density-matched family on a different lift
2. repeat the same comparison
3. check whether first-splitting multiplicity still dominates simple
   arrangement statistics there
---

## Session Entry: 2026-04-05 — Prime Transport Second Tight Density-Matched First Splitting

### Scope
- stayed entirely inside the exact recursive-system layer
- no quotient targets, no runtime router changes, no MUDBench changes
- replicated the tight matched-family comparison on a second lift

### Files Added / Updated
- `docs/research/prime_transport_second_tight_density_matched_first_splitting.md` → created
- `tools/prime_transport/run_second_tight_density_matched_first_split.py` → created and executed
- `results/prime_transport_recursive_system/visible_threshold_second_tight_density_matched_rows.csv` → created
- `results/prime_transport_recursive_system/visible_threshold_second_tight_density_matched_scores.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Matched Family
- exact lift: `30030 -> 510510`
- exact parent density for all four rows:
  `0.03776223776223776`
- tuplets:
  - `quad_alt_0406`
  - `double_twins_p17`
  - `quad_alt_0618`
  - `quadruplet`

### Main Result
- visible thresholds: `36`, `41`, `46`, `51`
- matched-family ranking:
  - `num_first_splitting_classes`: Spearman `1.0`
  - `total_extra_child_classes_from_first_split`: `1.0`
  - `fraction_first_splitting_classes`: `1.0`
  - `gap_max`: `0.31622776601683794`

### Interpretation
- the tight matched result now replicates across two different lifts
- the best current exact-layer law class is robustly
  `density + first-splitting event`
- simple arrangement statistics remain secondary on the matched evidence so far

### Next Exact-Layer Step
1. if a tiny matched family can be built cleanly on the `19` lift, test it
2. otherwise stop extension and formalize the current law class as the stable
   working hypothesis before proposing cautious bounds

---

## Session Entry: 2026-04-05 — Prime Transport Visible Threshold Canonical Law Class

### Scope
- no new experiments
- no quotient targets, no runtime router changes, no MUDBench changes
- consolidated the current exact-layer threshold-law result into one durable
  canonical note

### Files Added / Updated
- `docs/research/prime_transport_visible_threshold_canonical_law_class.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Canonical Exact-Layer Conclusion
- internal split is immediate in the current exact table
- visible split is the delayed threshold object
- density alone is insufficient
- burden and simple arrangement alone are insufficient
- the best current exact-layer law class is
  `density + first-splitting event`
- arrangement is secondary on the current tightly matched evidence

### Status
- this is now the stable exact-layer reference for future prime-transport
  routing-theory work
- it is a law class / mechanism class, not a closed formula

---

## Session Entry: 2026-04-05 — Prime Transport Exact-Layer Synthesis

### Scope
- no new experiments
- no quotient targets, no runtime router changes, no MUDBench changes
- wrote one compact synthesis note tying together the canonical exact-layer
  stack and the canonical threshold-law result

### Files Added / Updated
- `docs/research/prime_transport_exact_layer_synthesis.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Synthesis Result
- exact state is organized as phase-fiber-scale factorization
- finite-horizon spin is the internal predictive compression object
- delayed visibility is the mechanism from hidden refinement to visible
  predictive refinement
- the current best visible-threshold law class is
  `density + first-splitting event`
- the minimal routing interpretation is now stated explicitly without changing
  runtime code

---

## Session Entry: 2026-04-05 — Prime Transport Mock Router Grouping Key

### Scope
- no runtime router changes
- no MUDBench seam changes
- bounded offline comparison only
- goal: replace literal `R_min` route identity with a coarser exact-layer
  routing partition

### Files Added / Updated
- `tools/prime_transport/run_mock_router_grouping_eval.py` → created
- `docs/research/prime_transport_mock_router_grouping_key.md` → created
- `results/prime_transport_recursive_system/prime_transport_mock_router_grouping_eval.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- literal `R_min` keys are confirmed to be too fine-grained for trace-level
  routing
- `gap_only` is too coarse and promotes almost everything
- `phi_gap` stays too close to literal identity
- `base_zero_gap` lowers ambiguity modestly but is not meaningfully smaller
  than the average full-spin partition
- the best current first coarse routing key is
  `base_gap = (b, r, next_return_gap)`

### Status
- the next offline prototype step should switch the mock router partition to
  `base_gap` and rerun the trace-level promotion path

---

## Session Entry: 2026-04-05 — Prime Transport Base-Gap Prototype

### Scope
- offline only
- no live router changes
- no MUDBench seam changes
- goal: test the first full routing prototype using stored `R_min`,
  `base_gap` partitioning, and selective fallback to `R_full`

### Files Added / Updated
- `tools/prime_transport/run_mock_router_base_gap_prototype.py` → created
- `docs/research/prime_transport_mock_router_base_gap_prototype.md` → created
- `results/prime_transport_recursive_system/prime_transport_mock_router_base_gap_prototype.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- `literal_rmin` remains exact but is not a reusable partition
- `gap_only` reaches high resolution only by promoting almost everything
- `base_gap` gives the best current tradeoff:
  substantial route reuse, meaningful unresolved ambiguity, moderate
  promotion, and near-complete post-promotion resolution

### Status
- `base_gap = (b, r, next_return_gap)` is now the best current first serious
  offline routing prototype
- the next offline step should inspect promoted `base_gap` cases and test
  whether one minimal exact-layer refinement reduces promotion materially

---

## Session Entry: 2026-04-05 — Prime Transport Base-Gap Refinement

### Scope
- offline only
- no live router changes
- no MUDBench seam changes
- goal: test whether one minimal exact-layer refinement can beat `base_gap`
  without collapsing back toward address-like routing

### Files Added / Updated
- `tools/prime_transport/run_mock_router_base_gap_refinement.py` → created
- `docs/research/prime_transport_mock_router_base_gap_refinement.md` → created
- `results/prime_transport_recursive_system/prime_transport_mock_router_base_gap_refinement.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- promoted `base_gap` cases are missing a real phi-side distinction
- the tested refinement `base_gap_phi0 = (b, r, next_return_gap, phi0)` does
  reduce promotion materially
- but it also makes the partition much finer and reduces route reuse sharply
- so it is not a better enough tradeoff to replace `base_gap`

### Status
- `base_gap` remains the best current first offline routing prototype
- any next refinement should be smaller than a direct phi-coordinate add-on,
  or the offline design can stop here and move to implementation-side wiring

---

## Session Entry: 2026-04-05 — Prime Transport Base-Gap Routing Loop

### Scope
- offline only
- no live router changes
- no MUDBench seam changes
- goal: test the selected `base_gap` partition inside a cached routing loop
  over the bounded traces

### Files Added / Updated
- `tools/prime_transport/run_base_gap_routing_loop_eval.py` → created
- `docs/research/prime_transport_base_gap_routing_loop_eval.md` → created
- `results/prime_transport_recursive_system/prime_transport_base_gap_routing_loop_eval.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- `base_gap` route classes are reused heavily across the bounded traces
- route decisions stay stable once cached
- promotion remains selective rather than universal
- effective resolved fraction stays high in loop operation

### Status
- `base_gap` is now mature enough for a guarded non-runtime integration
  experiment
- no live integration is justified yet, but the offline design can now move
  from pure analysis scripts toward a bounded prototype boundary

---

## Session Entry: 2026-04-05 — Prime Transport Base-Gap Adapter Integration

### Scope
- guarded and offline only
- no live router changes
- no MUDBench seam changes
- goal: package the selected `base_gap` policy into a callable research-side
  adapter boundary

### Files Added / Updated
- `tools/prime_transport/base_gap_routing_adapter.py` → created
- `tools/prime_transport/run_base_gap_routing_adapter_demo.py` → created
- `docs/research/prime_transport_base_gap_adapter_integration.md` → created
- `results/prime_transport_recursive_system/prime_transport_base_gap_routing_adapter_demo.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the selected `base_gap` policy now has a minimal callable adapter boundary
- the tiny benchmark-like demo exercises initialize, step, route, and
  promotion fallback on bounded exact traces
- the adapter preserves the same qualitative reuse/promotion pattern as the
  loop evaluation

### Status
- this is now the first guarded non-runtime integration boundary for the
  current routing policy
- the first future hookup point should be a research-only benchmark harness
  wrapper around the existing bounded trace-building path

---

## Session Entry: 2026-04-05 — Prime Transport Base-Gap Benchmark Wrapper

### Scope
- guarded and research-only
- no live router changes
- no MUDBench seam changes
- goal: hook the guarded adapter into the existing bounded trace-building path
  and expose a benchmark-like wrapper boundary

### Files Added / Updated
- `tools/prime_transport/base_gap_benchmark_wrapper.py` → created
- `tools/prime_transport/run_base_gap_benchmark_wrapper.py` → created
- `docs/research/prime_transport_base_gap_benchmark_wrapper.md` → created
- `results/prime_transport_recursive_system/prime_transport_base_gap_benchmark_wrapper.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the existing bounded trace-building path is now reused directly behind a
  research-only benchmark wrapper
- the wrapper preserves the same route reuse, promotion, and fallback behavior
  as the prior bounded loop evaluation
- this is the smallest future hookup point for a guarded benchmark-harness
  experiment

### Status
- the adapter is now ready for a future guarded benchmark-harness experiment
- no live integration is justified yet

---

## Session Entry: 2026-04-05 — Prime Transport Guarded Benchmark Harness Eval

### Scope
- guarded and research-only
- no live router changes
- no MUDBench seam changes
- goal: compare the selected `base_gap` policy against simple controls through
  the research-only benchmark wrapper boundary

### Files Added / Updated
- `tools/prime_transport/run_guarded_benchmark_harness_eval.py` → created
- `docs/research/prime_transport_guarded_benchmark_harness_eval.md` → created
- `results/prime_transport_recursive_system/prime_transport_guarded_benchmark_harness_eval.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- `static_only` remains exact but address-like and therefore not reusable
- `gap_only` remains coarse but too fallback-heavy
- `base_gap` sits in the intended middle position:
  reusable, stable, and materially less fallback-heavy than `gap_only`

### Status
- `base_gap` is now strong enough for a future guarded benchmark-side
  integration step
- the remaining unresolved risk is fallback burden on bounded traces, not route
  instability

---

## Session Entry: 2026-04-05 — Prime Transport Base-Gap Fallback Burden

### Scope
- research-only
- no live router changes
- no MUDBench seam changes
- goal: profile only the promoted/fallback cases of the current `base_gap`
  policy and determine whether one cheap bounded refinement is justified

### Files Added / Updated
- `tools/prime_transport/analyze_base_gap_fallback_burden.py` → created
- `docs/research/prime_transport_base_gap_fallback_burden.md` → created
- `results/prime_transport_recursive_system/prime_transport_base_gap_fallback_burden.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- aggregate fallback burden is moderately structured rather than sharply
  concentrated
- the deeper lift family is closer to diffuse than to concentrated
- there is no single obvious cheap extra refinement that current bounded
  evidence justifies before guarded benchmark-side integration

### Status
- `base_gap` should remain unchanged for the first guarded integration step
- fallback accounting should remain explicit and observable

---

## Session Entry: 2026-04-05 — Prime Transport Guarded Benchmark Hookup Plan

### Scope
- research-only planning
- no live router changes
- no MUDBench seam changes
- goal: define the smallest future benchmark-side hookup plan for the current
  `base_gap` policy

### Files Added / Updated
- `docs/research/prime_transport_guarded_benchmark_hookup_plan.md` → created
- `results/prime_transport_recursive_system/prime_transport_guarded_benchmark_hookup_checklist.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the smallest future hookup point is now fixed as the existing research-only
  wrapper around the bounded trace-building path
- policy inputs, outputs, controls, and required metrics are now stated
  explicitly
- fallback accounting is recorded as a mandatory safeguard rather than an
  optional metric

### Status
- no further offline policy refinement is required before the first guarded
  benchmark-side hookup experiment
- no live seam work is justified yet

---

## Session Entry: 2026-04-05 — Prime Transport Research-Side Integration Readiness

### Scope
- research-side consolidation only
- no live router changes
- no MUDBench seam changes
- no new experiments
- goal: write one final readiness note for the current guarded routing policy

### Files Added / Updated
- `docs/research/prime_transport_research_side_integration_readiness.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the current policy, wrapper boundary, benchmark comparison result, and
  fallback-risk reading are now consolidated into one compact readiness note
- the policy is marked ready for a research-side integration trial
- the existing research-only wrapper boundary remains the smallest safe hookup
  point

### Status
- this is now the top-level readiness reference for the current
  prime-transport routing policy
- no live integration is justified yet

---

## Session Entry: 2026-04-05 — Prime Transport Research-Side Integration Trial

### Scope
- research-only
- no live router changes
- no MUDBench seam changes
- goal: execute the smallest research-side integration trial through the
  existing wrapper boundary and bounded trace family

### Files Added / Updated
- `docs/research/prime_transport_research_side_integration_trial.md` → created
- `results/prime_transport_recursive_system/prime_transport_guarded_benchmark_harness_eval.csv` → refreshed
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the trial reproduces the prior guarded benchmark result without drift
- `base_gap` remains the stable middle policy between `static_only` and
  `gap_only`
- the remaining guarded risk is still fallback burden, not instability

### Status
- the policy is now ready for a larger research-side benchmark experiment
- no live seam work is justified yet

---

## Session Entry: 2026-04-05 — Prime Transport Larger Research-Side Benchmark

### Scope
- research-only
- no live router changes
- no MUDBench seam changes
- goal: run the first larger research-side benchmark through the same guarded
  wrapper boundary on a broader exact-row trace set

### Files Added / Updated
- `tools/prime_transport/run_larger_research_side_benchmark.py` → created
- `docs/research/prime_transport_larger_research_side_benchmark.md` → created
- `results/prime_transport_recursive_system/prime_transport_larger_research_side_benchmark.csv` → created
- `tools/prime_transport/base_gap_benchmark_wrapper.py` → updated to accept explicit source paths while preserving the existing boundary
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- `base_gap` remains stable on the larger exact-row trace set
- fallback burden does not grow materially and is lower than in the smaller
  guarded trial
- the policy still occupies the intended middle position between `static_only`
  and `gap_only`

### Status
- the current policy remains the right candidate for future guarded
  benchmark-side work beyond the original tiny trial
- no live integration is justified yet

---

## Session Entry: 2026-04-05 — Prime Transport Research-Side Benchmark Validation

### Scope
- documentation-only consolidation
- no live router changes
- no MUDBench seam changes
- goal: freeze the current research-side benchmark validation status of the
  `base_gap` routing policy in one durable note

### Files Added / Updated
- `docs/research/prime_transport_research_side_benchmark_validation.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the note consolidates the guarded benchmark trial and the larger
  research-side benchmark into one validation status reference
- `base_gap` remains stable, route-decision instability remains zero, and
  fallback burden remains the main explicit guarded cost

### Status
- the current policy is documented as sufficiently validated for future guarded
  research-side integration work beyond the current wrapper boundary
- no live seam work is justified yet

---

## Session Entry: 2026-04-05 — Prime Transport vs Angular Hopf Head-to-Head

### Scope
- research-only
- no live router changes
- no MUDBench seam changes
- goal: run the first guarded direct comparison between the canonical angular
  Hopf practical baseline and the prime-transport `base_gap + fallback` policy

### Files Added / Updated
- `tools/prime_transport/angular_hopf_benchmark_adapter.py` → created
- `tools/prime_transport/run_prime_transport_vs_angular_hopf_head_to_head.py` → created
- `docs/research/prime_transport_vs_angular_hopf_head_to_head.md` → created
- `results/prime_transport_recursive_system/prime_transport_vs_angular_hopf_head_to_head.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- on the shared larger guarded trace set, the canonical angular Hopf baseline
  is extremely compact but too coarse, with effective resolved fraction only
  `0.1681`
- `base_gap + fallback` remains stable and reaches effective resolved fraction
  `0.9900` with bounded fallback burden and zero instability
- `gap_only` remains the coarse high-fallback control

### Status
- `base_gap + fallback` is now the stronger bounded practical policy on this
  research-side comparison boundary
- delayed refinement and explicit fallback accounting are now the main ideas
  worth transferring back into any future angular-side practical routing work

---

## Session Entry: 2026-04-05 — Angular Hopf vs Prime Transport With Transfers

### Scope
- research-only
- no live router changes
- no MUDBench seam changes
- goal: test whether the prime-side win survives after delayed refinement,
  fallback accounting, and predictive partitioning are transferred back into
  the unchanged angular Hopf path

### Files Added / Updated
- `tools/prime_transport/run_angular_hopf_vs_prime_transport_with_transfers.py` → created
- `docs/research/angular_hopf_vs_prime_transport_with_transfers.md` → created
- `results/prime_transport_recursive_system/angular_hopf_vs_prime_transport_with_transfers.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- guarded angular Hopf recovers predictive resolution only by promoting almost
  everything (`0.9883` step fraction)
- `base_gap + fallback` remains near-complete in effective resolution while
  keeping fallback much lower (`0.3312`)
- the remaining prime-side advantage therefore survives the discipline
  transfer and now looks structural rather than merely procedural

### Status
- delayed refinement and fallback accounting now look like generally useful
  routing discipline
- `base_gap + fallback` remains the stronger practical policy on the guarded
  research-side boundary

---

## Session Entry: 2026-04-05 — Prime Transport Routing Decision Freeze

### Scope
- documentation-only consolidation
- no live router changes
- no MUDBench seam changes
- goal: freeze the current guarded research-side routing decision in one
  durable note

### Files Added / Updated
- `docs/research/prime_transport_routing_decision.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the note consolidates the guarded benchmark validation, the direct
  head-to-head, and the guarded angular transfer comparison into one policy
  decision
- `base_gap + fallback` is now recorded as the leading research-side routing
  candidate
- canonical angular Hopf is now recorded as the practical comparison baseline

### Status
- the next guarded research-side benchmark integration trial should use prime
  transport as the primary policy under test and angular Hopf as baseline
- no live seam work is justified yet

---

## Session Entry: 2026-04-05 — Prime Transport Guarded Real-Signal Trial

### Scope
- research-only
- no live router changes
- no MUDBench seam changes
- goal: test the prime-transport policy against real MUDBench replay signal
  through the guarded research-side wrapper boundary

### Files Added / Updated
- `tools/prime_transport/real_signal_benchmark_wrapper.py` → created
- `tools/prime_transport/run_prime_transport_guarded_real_signal_trial.py` → created
- `docs/research/prime_transport_guarded_real_signal_trial.md` → created
- `results/prime_transport_recursive_system/prime_transport_guarded_real_signal_trial.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- on the first replay-based real-signal trial, `base_gap` retains zero
  fallback burden and full effective resolution
- guarded angular still requires materially more fallback to reach full
  resolution
- the main remaining weakness is that current real-signal `base_gap` keying is
  too fine to show route reuse on the tiny bounded traces

### Status
- the prime policy does not collapse outside the synthetic trace family
- the next bounded real-signal step should focus on route reuse / class
  coarsening rather than on fallback correctness

---

## Session Entry: 2026-04-05 — Prime Transport Real-Signal Reuse Recovery

### Scope
- research-only
- no live router changes
- no MUDBench seam changes
- goal: determine why `base_gap` shows zero reuse on replay-based real signal
  and test the smallest guarded coarsening/alignment candidates

### Files Added / Updated
- `tools/prime_transport/run_prime_transport_real_signal_reuse_recovery.py` → created
- `docs/research/prime_transport_real_signal_reuse_recovery.md` → created
- `results/prime_transport_recursive_system/prime_transport_real_signal_reuse_recovery.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- zero reuse is now traced mainly to exact base phase on very short replay
  traces rather than to exact `next_return_gap`
- gap bucketing alone changes nothing
- the first tiny base-phase coarsening recovers reuse, but only by giving back
  the fallback advantage

### Status
- the original real-signal `base_gap` policy should remain unchanged for now
- real-signal reuse remains the main open practical problem, and the first
  cheap coarsening did not solve it well enough

---

## Session Entry: 2026-04-05 — Prime Transport Real-Signal Reuse Decision

### Scope
- documentation-only consolidation
- no live router changes
- no MUDBench seam changes
- goal: freeze the current real-signal reuse conclusion and define the next
  larger guarded replay slice for a fair reuse test

### Files Added / Updated
- `docs/research/prime_transport_real_signal_reuse_decision.md` → created
- `results/prime_transport_recursive_system/prime_transport_next_real_signal_slice_checklist.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the note freezes the conclusion that `base_gap` remains the correct first
  real-signal policy on the tiny replay set
- further tiny coarsening work is not justified before moving to a larger
  replay slice
- the next exact guarded slice is now fixed as the paired
  `timing_mode_eval_v1` baseline replay artifacts

### Status
- the next fair real-signal reuse test should use the paired baseline replay
  slice with the same guarded comparison metrics
- no live seam work is justified yet

---

## Session Entry: 2026-04-05 — Prime Transport Router Memory Workspace Experiment

### Scope
- research-only bounded workspace-system prototype
- no live router changes
- no MUDBench seam changes
- no training or read-path redesign
- goal: test the unchanged router-memory layer as a small local-plus-shared
  workspace system

### Files Added / Updated
- `tools/prime_transport/run_router_memory_workspace_experiment.py` → created
- `results/prime_transport_recursive_system/prime_transport_router_memory_workspace_experiment.csv` → created
- `docs/research/prime_transport_router_memory_workspace_experiment.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the unchanged memory architecture now supports bounded workspace-style local
  plus shared state maintenance
- per-entity retrieval accuracy, shared-ledger retrieval accuracy, and joint
  snapshot accuracy are all `1.0`
- route reuse remains high at `0.8411588411588411`
- promoted-query burden remains real at `0.48843742442001076`, but instability
  stays `0.0`

### Status
- the branch is strong enough to justify a future larger router-native systems
  experiment beyond this bounded workspace prototype
- promoted querying remains the main explicit architectural cost

---

## Session Entry: 2026-04-05 — Prime Transport Router Memory Agent Loop

### Scope
- research-only bounded agent-style control loop
- no live router changes
- no MUDBench seam changes
- no training or memory-architecture redesign
- goal: test whether the unchanged workspace memory supports repeated
  read/decide/write control behavior

### Files Added / Updated
- `tools/prime_transport/run_router_memory_agent_loop.py` → created
- `results/prime_transport_recursive_system/prime_transport_router_memory_agent_loop.csv` → created
- `docs/research/prime_transport_router_memory_agent_loop_family.md` → created
- `docs/research/prime_transport_router_memory_agent_loop.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- local and shared retrieval remain exact under the bounded control loop
- route reuse remains `0.8411588411588411`
- route decision instability remains `0.0`
- action correctness is `0.5975357975357976`
- joint control-loop correctness is `0.4287376200235855`

### Status
- memory-substrate coherence remains validated
- the next honest step is to improve bounded control policy before scaling to a
  larger coordination-logic prototype

---

## Session Entry: 2026-04-05 — Prime Transport Router Memory Context Packet

### Scope
- research-only bounded context-packet comparison
- no live router changes
- no MUDBench seam changes
- no training, no memory redesign, no promotion change
- goal: test whether a minimal structured context packet improves control-loop
  quality over the same unchanged workspace memory

### Files Added / Updated
- `tools/prime_transport/run_router_memory_context_packet.py` → created
- `results/prime_transport_recursive_system/prime_transport_router_memory_context_packet.csv` → created
- `docs/research/prime_transport_router_memory_context_packet.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- action correctness improves from `0.5975357975357976` to
  `0.7577422577422577`
- joint control-loop correctness improves from `0.4287376200235855` to
  `0.4768426712341106`
- route reuse, promoted-query burden, and instability remain unchanged

### Status
- the context packet is now the best bounded control surface tested on top of
  the current router-memory workspace
- future coordination experiments should keep the same memory architecture and
  use structured context packets rather than the looser baseline controller

---

## Session Entry: 2026-04-05 — Prime Transport Router Memory Packet Control Improvement

### Scope
- research-only bounded controller comparison
- no live router changes
- no MUDBench seam changes
- no training
- no routing, memory, promotion, or packet redesign
- goal: improve control quality only by upgrading the bounded controller on the
  unchanged context-packet surface

### Files Added / Updated
- `tools/prime_transport/run_router_memory_context_packet.py` → extended
- `results/prime_transport_recursive_system/prime_transport_router_memory_context_packet.csv` → refreshed
- `docs/research/prime_transport_router_memory_packet_control_improvement.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- action correctness improves from `0.5975357975357976` to
  `0.7676989676989677`
- joint control-loop correctness improves from `0.4287376200235855` to
  `0.49339896262483907`
- route reuse, promoted-query burden, retrieval accuracy, and instability are
  unchanged

### Status
- the context packet is now validated as a strong bounded control surface
- the branch is ready for a future larger coordination-logic prototype on the
  same unchanged routing and memory stack

---

## Session Entry: 2026-04-05 — Prime Transport Attractor Identity Experiment

### Scope
- research-only bounded attractor-identity comparison
- no live router changes
- no MUDBench seam changes
- no training
- no routing, memory, promotion, or packet redesign
- goal: test one minimal convergence-identity signal on top of the unchanged
  router-memory plus packet stack

### Files Added / Updated
- `tools/prime_transport/run_router_memory_context_packet.py` → extended
- `results/prime_transport_recursive_system/prime_transport_router_memory_context_packet.csv` → refreshed
- `docs/research/prime_transport_attractor_identity_experiment.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- attractor identity does not reduce promoted-query burden
- action correctness improves to `0.8437895437895437`
- joint control-loop correctness improves to `0.6188811188811189`
- retrieval remains exact and instability remains `0.0`

### Status
- the tested attractor identity is a useful bounded convergence aid for
  controller quality
- it is not yet a validated promoted-query reduction mechanism

---

## Session Entry: 2026-04-05 — Prime Transport Router Memory Control Stack Decision

### Scope
- documentation-only architectural freeze
- no live router changes
- no MUDBench seam changes
- no training and no code changes
- goal: freeze the best current controller surface and define the next bounded
  coordination-logic prototype

### Files Added / Updated
- `docs/research/prime_transport_router_memory_control_stack_decision.md` → created
- `docs/research/prime_transport_router_memory_coordination_prototype.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the current best bounded stack is fixed as unchanged router-memory plus
  unchanged packet surface plus attractor identity
- attractor identity is recorded as a control-quality aid, not a promoted-query
  reduction
- the next prototype is defined as a bounded reassignment and dependency
  coordination loop on the same unchanged stack

### Status
- the branch is ready to test harder bounded coordination logic without
  redesigning routing, memory, packet structure, or promotion behavior

---

## Session Entry: 2026-04-05 — Prime Transport Router Memory Coordination Experiment

### Scope
- research-only bounded four-entity coordination prototype
- no live router changes
- no MUDBench seam changes
- no training and no architecture redesign
- goal: test the current best controller surface on a harder reassignment and
  dependency-coordination loop

### Files Added / Updated
- `tools/prime_transport/run_router_memory_coordination_experiment.py` → created
- `results/prime_transport_recursive_system/prime_transport_router_memory_coordination_experiment.csv` → created
- `docs/research/prime_transport_router_memory_coordination_experiment.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- action correctness reaches `0.8075674325674326`
- reassignment / handoff correctness reaches `0.9285714285714286`
- local and shared retrieval remain exact
- route decision instability remains `0.0`
- joint full-loop coordination correctness is mixed at `0.4391505191725942`

### Status
- the current best stack is strong enough for a bounded internal
  system-architecture demo
- the next remaining challenge is harder full-loop coordination quality, not
  routing or memory coherence

---

## Session Entry: 2026-04-05 — Router-Native Architecture Demo Package

### Scope
- documentation-only internal checkpoint packaging
- no live router changes
- no MUDBench seam changes
- no new experiments
- no code changes
- goal: package the current router-native branch into a stable internal demo
  checkpoint for continued work

### Files Added / Updated
- `docs/research/router_native_architecture_demo.md` → created
- `results/prime_transport_recursive_system/router_native_architecture_demo_manifest.csv` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the current best stack is frozen as unchanged router-memory plus unchanged
  promotion/query behavior plus packet plus attractor identity
- the validated stages, open costs, key drivers, key CSVs, and next experiment
  are now collected in one internal package

### Status
- the branch now has a stable internal launchpad for the next coordination-
  quality experiments without needing to reconstruct the control stack from
  many separate notes

---

## Session Entry: 2026-04-05 — Coordination Policy Improvement

### Scope
- bounded controller-only improvement on the unchanged router-native stack
- no live router changes
- no MUDBench seam changes
- no memory, routing, packet, attractor, or promotion redesign
- goal: improve harder coordination quality on the same validated substrate

### Files Added / Updated
- `tools/prime_transport/run_coordination_policy_improvement.py` → created
- `results/prime_transport_recursive_system/prime_transport_coordination_policy_improvement.csv` → created
- `docs/research/prime_transport_coordination_policy_improvement.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the most likely bounded controller failure mode is local-versus-shared ledger
  conflict under partial dependency visibility, combined with over-eager
  transfer actions
- the improved policy reconciles local record stage against shared flags and
  delays transfer until direct local progress paths are exhausted
- aggregate action correctness rises to `0.8164585414585415`
- reassignment / handoff correctness reaches `1.0`
- joint coordination-loop correctness rises to `0.4549983440244225`
- retrieval stays exact, route reuse stays `0.8425859854431283`, promoted-query
  burden stays `0.5047922252391286`, and instability stays `0.0`

### Status
- controller quality improved modestly without changing the stack
- the branch remains strong enough for a larger bounded systems prototype
- the next remaining headroom is still coordination-policy quality rather than
  substrate repair

---

## Session Entry: 2026-04-05 — Coordination Lookahead Test

### Scope
- bounded controller-only lookahead / replanning test
- same unchanged router-memory substrate
- same unchanged packet plus attractor controller surface
- no live router changes
- no memory, routing, packet, attractor, query, or promotion redesign

### Files Added / Updated
- `tools/prime_transport/run_coordination_lookahead.py` → created
- `results/prime_transport_recursive_system/prime_transport_coordination_lookahead.csv` → created
- `docs/research/prime_transport_coordination_lookahead.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the small lookahead / replanning controller preserves exact retrieval, route
  reuse, promoted-query burden, and zero instability
- it does **not** improve aggregate coordination quality over the improved
  local coordination policy
- aggregate action correctness changes from `0.8164585414585415` to
  `0.8162837162837163`
- aggregate joint coordination-loop correctness changes from
  `0.4549983440244225` to `0.45384747564191574`

### Status
- this specific one-step lookahead is a rejected controller variant
- the next remaining headroom is still in coordination logic, but likely needs
  a more structural design than another tiny replanning tweak

---

## Session Entry: 2026-04-05 — Coordination Conflict Arbitration Test

### Scope
- bounded controller-only conflict / dependency arbitration test
- same unchanged router-memory substrate
- same unchanged packet plus attractor controller surface
- no live router changes
- no memory, routing, packet, attractor, query, or promotion redesign

### Files Added / Updated
- `tools/prime_transport/run_coordination_conflict_arbitration.py` → created
- `results/prime_transport_recursive_system/prime_transport_coordination_conflict_arbitration.csv` → created
- `docs/research/prime_transport_coordination_conflict_arbitration.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the explicit conflict / dependency arbitration controller preserves exact
  retrieval, route reuse, promoted-query burden, and zero instability
- it does **not** improve aggregate coordination quality over the improved
  local policy
- aggregate action correctness is `0.8162837162837163`
- aggregate joint coordination-loop correctness is `0.45384747564191574`
- on the aggregate row it matches the rejected one-step lookahead controller

### Status
- this arbitration variant is a second rejected controller family on the
  current surface
- the improved local coordination policy remains the best bounded controller
  tested so far
- the next larger step should be more structural than another small tweak on
  the same controller surface

---

## Session Entry: 2026-04-05 — Coordination Frame Experiment

### Scope
- bounded controller-only coordination-frame / transaction-layer test
- same unchanged router-memory substrate
- same unchanged packet plus attractor controller surface
- no live router changes
- no memory, routing, packet, attractor, query, or promotion redesign

### Files Added / Updated
- `tools/prime_transport/run_coordination_frame_experiment.py` → created
- `results/prime_transport_recursive_system/prime_transport_coordination_frame_experiment.csv` → created
- `docs/research/prime_transport_coordination_frame_experiment.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the coordination-frame controller preserves exact retrieval, route reuse,
  promoted-query burden, and zero instability
- it slightly improves the harder joint coordination-loop metric over the
  improved local controller
- aggregate action correctness is `0.8164085914085915`
- aggregate joint coordination-loop correctness is `0.45520759282124196`
- aggregate reassignment / handoff correctness remains `1.0`

### Status
- the coordination frame is now the best bounded higher-level controller
  variant tested on this stack
- the gain is real but modest
- the next larger bounded systems prototype can now reasonably use explicit
  coordination episodes as the current best controller direction

---

## Session Entry: 2026-04-05 — Geometry-Native Controller Test

### Scope
- bounded controller-only geometry-native signal test
- same unchanged router-memory substrate
- same unchanged packet plus attractor controller surface
- no live router changes
- no memory, routing, packet, attractor, query, or promotion redesign

### Files Added / Updated
- `tools/prime_transport/run_geometry_native_controller.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_controller.csv` → created
- `docs/research/prime_transport_geometry_native_controller.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the first tested math-native controller signal is
  `visibility_zone = (r_band, next_return_gap_band)`
- it preserves exact retrieval, route reuse, promoted-query burden, and zero
  instability
- it exactly matches the current best coordination-frame aggregate
- aggregate action correctness remains `0.8164085914085915`
- aggregate joint coordination-loop correctness remains `0.45520759282124196`

### Status
- this first geometry-native controller is a parity result, not a new gain
- the coordination-frame controller remains the best bounded controller result
- the next math-native step, if desired, should use a richer or differently
  structured geometry signal

---

## Session Entry: 2026-04-05 — Phase-Fiber Geometry Controller Test

### Scope
- bounded controller-only richer geometry-signal test
- same unchanged router-memory substrate
- same unchanged packet plus attractor controller surface
- no live router changes
- no memory, routing, packet, attractor, query, or promotion redesign

### Files Added / Updated
- `tools/prime_transport/run_phase_fiber_geometry_controller.py` → created
- `results/prime_transport_recursive_system/prime_transport_phase_fiber_geometry_controller.csv` → created
- `docs/research/prime_transport_phase_fiber_geometry_controller.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the richer tested math-native signal is
  `phase_fiber_visibility_basin = (r_band, b_sector, phi_bucket, gap_band)`
- it preserves exact retrieval, route reuse, promoted-query burden, and zero
  instability
- it degrades aggregate action correctness to `0.606943056943057`
- it degrades aggregate joint coordination-loop correctness to
  `0.2999959120268171`

### Status
- this richer phase-fiber-aware basin signal is a rejected controller variant
- the coordination-frame controller remains the best bounded controller result
- the next math-native step, if pursued, should be more selective than a broad
  phase-fiber basin override

---

## Session Entry: 2026-04-05 — Geometry-Native Sequence Model Test

### Scope
- first direct bounded sequence-model test of the main thesis
- geometry as the primary sequence-processing engine
- no live router changes
- no transformer blocks inside the geometry-native path
- tiny transformer allowed only as the comparison baseline

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model.csv` → created
- `docs/research/prime_transport_geometry_native_sequence_model.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the bounded task is a phase-fiber-controlled symbolic stream whose token
  semantics depend on the evolving exact routing state `(b, phi, r,
  next_return_gap)`
- the geometry-native model uses explicit geometric transport, routed
  structured memory, and a small learned readout
- aggregate geometry-native result:
  - test accuracy `0.592881917953`
  - query accuracy `0.885010242462`
  - test loss `0.815225899220`
- aggregate tiny transformer result:
  - test accuracy `0.430338531733`
  - query accuracy `0.503764569759`
  - test loss `1.063794016838`

### Status
- this is the first direct bounded result in which geometry acts as the main
  sequence engine rather than only as routing support
- the result supports continuing the thesis on stronger bounded sequence tests
- it is not yet a broad replacement claim, because the task is still small and
  architecture-aligned

---

## Session Entry: 2026-04-05 — Geometry-Native Sequence Model V2

### Scope
- second direct bounded thesis test
- stronger and less architecture-aligned sequence task
- no live router changes
- no transformer blocks inside the geometry-native path
- tiny transformer allowed only as the comparison baseline

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v2.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v2.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v2.csv` → created
- `docs/research/prime_transport_geometry_native_sequence_model_v2.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the new bounded task is a compositional stack-rewrite stream with
  `PUSH`, `POP`, `SWAP2`, `MERGE`, and `QUERY`
- the geometry-native model uses explicit geometric transport plus bounded
  structured stack memory as the main sequence engine
- aggregate geometry-native result:
  - test accuracy `1.0`
  - query accuracy `1.0`
  - test loss `0.000000022366`
- aggregate tiny transformer result:
  - test accuracy `0.790161132812`
  - query accuracy `0.770186364651`
  - test loss `0.445333242416`

### Status
- this strengthens the thesis in the bounded experimental sense
- the result is still a bounded explicit geometry-native engine result, not a
  broad learned-sequence replacement claim
- the next sharp test should be held-out generalization on the same stronger
  task family

---

## Session Entry: 2026-04-05 — Geometry-Native Sequence Model V3

### Scope
- third direct bounded thesis test
- more language-like context-dependent sequence task
- no live router changes
- no transformer blocks inside the geometry-native path
- tiny transformer allowed only as the comparison baseline

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v3.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v3.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v3.csv` → created
- `docs/research/prime_transport_geometry_native_sequence_model_v3.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the new bounded task is a discourse-style reference stream with overloaded
  `TAG`, `REFER`, and `ASK` tokens whose meaning depends on evolving speaker,
  topic, focus, style, and exact geometry
- the geometry-native model uses explicit geometric transport plus explicit
  discourse-role memory as the main sequence engine
- aggregate geometry-native result:
  - test accuracy `0.998567700386`
  - query accuracy `0.991875946522`
  - test loss `0.005087816156`
- aggregate tiny transformer result:
  - test accuracy `0.695442736149`
  - query accuracy `0.679468214512`
  - test loss `0.693002045155`

### Status
- this further strengthens the thesis in the bounded contextual-sequence
  setting
- the result is still a bounded explicit geometry-native engine result, not a
  broad natural-language replacement claim
- the next sharp test should be held-out generalization on the discourse-style
  family

---

## Session Entry: 2026-04-05 — Geometry-Native Sequence Model V4

### Scope
- fourth direct bounded thesis test
- held-out generalization on the discourse-style contextual family
- no live router changes
- no transformer blocks inside the geometry-native path
- tiny transformer allowed only as the comparison baseline

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v4.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v4.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v4.csv` → created
- `docs/research/prime_transport_geometry_native_sequence_model_v4.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- training excludes one explicit overloaded discourse query mode:
  speaker-reference queries under `style = 2` with distinct speaker/topic
  assignments
- test restores that removed mode and evaluates held-out generalization there
- aggregate geometry-native result:
  - held-out test accuracy `0.996438443661`
  - held-out query accuracy `0.979522168636`
  - test loss `0.017076831311`
- aggregate tiny transformer result:
  - held-out test accuracy `0.620404422283`
  - held-out query accuracy `0.453924924135`
  - test loss `0.954713940620`

### Status
- this is the strongest direct thesis result so far
- it strengthens the thesis beyond same-distribution bounded competence
- it is still a bounded explicit geometry-native engine result, not a broad
  natural-language replacement claim

---

## Session Entry: 2026-04-06 — Geometry-Native Sequence Model V5

### Scope
- fifth direct bounded thesis test
- multi-axis held-out generalization on the discourse-style contextual family
- no live router changes
- no transformer blocks inside the geometry-native path
- tiny transformer allowed only as the comparison baseline

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v5.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v5.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v5.csv` → created
- `docs/research/prime_transport_geometry_native_sequence_model_v5.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- training excludes the v4 held-out speaker-reference query mode, uses shorter
  contexts, and allows at most one style shift per sequence
- test restores the removed query mode inside longer sequences with at least
  four style shifts and at least three held-out queries
- aggregate geometry-native result:
  - held-out test accuracy `0.996558785439`
  - held-out query accuracy `0.985419213772`
  - test loss `0.011096824892`
- aggregate tiny transformer result:
  - held-out test accuracy `0.559709846973`
  - held-out query accuracy `0.469015806913`
  - test loss `1.370441794395`

### Status
- this is the strongest direct thesis result so far
- it strengthens the thesis beyond single-axis held-out competence
- it is still a bounded explicit geometry-native engine result, not a broad
  natural-language replacement claim

---

## Session Entry: 2026-04-06 — Geometry-Native Sequence Model V6

### Scope
- sixth direct bounded thesis test
- cross-family transfer on related contextual sequence families
- no live router changes
- no transformer blocks inside the geometry-native path
- tiny transformer allowed only as the comparison baseline

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v6.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v6.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v6.csv` → created
- `docs/research/prime_transport_geometry_native_sequence_model_v6.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- training uses the original discourse/reference family
- test uses a related delegation-style family with changed topic update,
  style update, tag binding, and query-resolution semantics
- aggregate geometry-native result:
  - transfer test accuracy `0.994750976562`
  - transfer query accuracy `0.971504330635`
  - test loss `0.014314385131`
- aggregate tiny transformer result:
  - transfer test accuracy `0.634033203125`
  - transfer query accuracy `0.661365151405`
  - test loss `1.114169836044`

### Status
- this is the strongest direct thesis result so far
- it strengthens the thesis beyond within-family held-out generalization
- it is still a bounded explicit geometry-native engine result, not a broad
  natural-language replacement claim

---

## Session Entry: 2026-04-06 — Geometry-Native Sequence Model V7

### Scope
- seventh direct bounded thesis test
- cross-family transfer under reduced shared abstract-state alignment
- no live router changes
- no transformer blocks inside the geometry-native path
- tiny transformer allowed only as the comparison baseline

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v7.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v7.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v7.csv` → created
- `docs/research/prime_transport_geometry_native_sequence_model_v7.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- training uses the original discourse/reference family
- test uses a related but more entangled target family whose true latent state
  is projected back into the old schema only through a lossy proxy
- aggregate geometry-native result:
  - transfer test accuracy `0.479204952717`
  - transfer query accuracy `0.544108569622`
  - test loss `8.150774955750`
- aggregate tiny transformer result:
  - transfer test accuracy `0.480468750000`
  - transfer query accuracy `0.502776086330`
  - test loss `2.312074422836`

### Status
- this is the first clear boundary result in the direct thesis line
- overall transfer advantage collapses to near parity once schema alignment is
  reduced enough
- a small query-specific geometry-native edge still survives

---

## Session Entry: 2026-04-06 — Geometry-Native Chart Realignment Test

### Scope
- first direct adaptive-geometry test after the v7 boundary result
- same reduced-schema-alignment transfer setting as v7
- one small local chart realignment rule only
- no live router changes
- no transformer blocks inside the geometry-native path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v8.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v8.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v8.csv` → created
- `docs/research/prime_transport_geometry_native_chart_realignment.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the realignment mechanism is a local angular role-frame rotation and
  candidate-basis rebasing selected by
  `chart_rotation = (b + r + style) mod 3`
- aggregate v8 geometry-native result:
  - transfer test accuracy `0.568933844566`
  - transfer query accuracy `0.624918460846`
  - test loss `6.270763874054`
- aggregate v8 tiny transformer result:
  - transfer test accuracy `0.477711409330`
  - transfer query accuracy `0.527071118355`
  - test loss `2.381304502487`

### Status
- chart realignment recovers a meaningful part of the v7 loss
- the recovered gain is geometry-native in a meaningful bounded sense
- the reduced-alignment problem is only partially repaired, not solved

---

## Session Entry: 2026-04-06 — Geometry-Native Multi-Chart Selection

### Scope
- bounded multi-chart adaptive-geometry test
- same reduced-schema-alignment transfer setting as v7/v8
- same unchanged downstream geometry-native engine
- no live router changes
- no transformer blocks inside the geometry-native path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v9.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v9.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v9.csv` → created
- `docs/research/prime_transport_geometry_native_multichart.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism uses a fixed family of three local chart variants built from
  the v8 rule plus offsets `{0,1,2}`
- per-sequence chart selection is based on mean max-softmax coherence from the
  existing readout
- aggregate v9 geometry-native result:
  - transfer test accuracy `0.547449469566`
  - transfer query accuracy `0.597468376160`
  - test loss `6.499938011169`
- aggregate v9 tiny transformer result:
  - transfer test accuracy `0.480124086142`
  - transfer query accuracy `0.491139233112`
  - test loss `2.158302783966`

### Status
- multi-chart selection still helps relative to the unrecovered v7 case
- it remains geometry-native in a meaningful bounded sense
- it does not materially improve on the simpler v8 single-rule realignment

---

## Session Entry: 2026-04-06 — Geometry-Native Chart Calibration

### Scope
- bounded chart-selection / calibration test after v8 and v9
- same reduced-schema-alignment transfer setting as v7/v8/v9
- same unchanged downstream geometry-native engine
- no live router changes
- no transformer blocks inside the geometry-native path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v10.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v10.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v10.csv` → created
- `docs/research/prime_transport_geometry_native_chart_calibration.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism uses a fixed three-chart family with short prefix-based chart
  calibration before committing to a chart for the full sequence
- calibration combines prefix query confidence, global confidence, and an
  inconsistency penalty over repeated recovered referent groups
- aggregate v10 geometry-native result:
  - transfer test accuracy `0.539407193661`
  - transfer query accuracy `0.569453179836`
  - test loss `6.897306919098`
- aggregate v10 tiny transformer result:
  - transfer test accuracy `0.485064327717`
  - transfer query accuracy `0.500942826271`
  - test loss `2.247161388397`

### Status
- v10 still improves on the unrecovered v7 reduced-alignment case
- v10 still beats the tiny transformer baseline
- v10 does not beat the simpler v8 single-rule chart realignment

---

## Session Entry: 2026-04-06 — Geometry-Native Divisibility Bridge

### Scope
- bounded divisibility-mediated realignment test after v7-v10
- same reduced-schema-alignment transfer setting as v7-v10
- same unchanged downstream geometry-native engine
- no live router changes
- no transformer blocks inside the geometry-native path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v11.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v11.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v11.csv` → created
- `docs/research/prime_transport_geometry_native_divisibility_bridge.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism uses prime-coded atomic role increments, semiprime bridge
  states, and small divisibility hubs for query and binding transitions
- aggregate v11 geometry-native result:
  - transfer test accuracy `0.883501827717`
  - transfer query accuracy `0.969032287598`
  - test loss `1.610011100769`
- aggregate v11 tiny transformer result:
  - transfer test accuracy `0.477366715670`
  - transfer query accuracy `0.510322570801`
  - test loss `2.036663055420`

### Status
- v11 materially improves on v8, v9, and v10
- v11 strongly reopens the gap over the tiny transformer baseline
- divisibility-mediated transition structure now looks like a plausible missing
  ingredient for reduced-alignment transfer

---

## Session Entry: 2026-04-06 — Geometry-Native Divisibility Bridge v12

### Scope
- stronger reduced-schema-alignment transfer stress test for the v11 bridge
- same bounded family and same unchanged downstream geometry-native engine
- longer sequences plus one additional latent entanglement term and noisier
  proxy projection
- no live router changes
- no transformer blocks inside the geometry-native path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v12.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v12.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v12.csv` → created
- `docs/research/prime_transport_geometry_native_divisibility_bridge_v12.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism keeps the v11 divisibility-bridge design spirit and minimally
  extends it with a second bounded hub term to mediate the added mismatch
- aggregate v12 geometry-native result:
  - transfer test accuracy `0.552269339561`
  - transfer query accuracy `0.584205031395`
  - test loss `6.065035820007`
- aggregate v12 tiny transformer result:
  - transfer test accuracy `0.452380955219`
  - transfer query accuracy `0.464435160160`
  - test loss `2.171264886856`

### Status
- v12 still improves on the unrecovered v7 case
- v12 still beats the tiny transformer baseline
- v12 weakens sharply relative to v11 and no longer beats v8

---

## Session Entry: 2026-04-06 — Geometry-Native Bridge Calibration

### Scope
- bounded support-window bridge calibration on the stronger v12 family
- same unchanged downstream geometry-native engine
- same divisibility-bridge design spirit, but with small bridge-family
  selection from early local evidence
- no live router changes
- no transformer blocks inside the geometry-native path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v13.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v13.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v13.csv` → created
- `docs/research/prime_transport_geometry_native_bridge_calibration.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism calibrates among three bounded bridge variants
  (`v11_like`, `v12_base`, `hybrid`) using a short support window with query
  confidence, global confidence, and disagreement penalty
- aggregate v13 geometry-native result:
  - transfer test accuracy `0.593377947807`
  - transfer query accuracy `0.633928596973`
  - test loss `5.709022045135`
- aggregate v13 tiny transformer result:
  - transfer test accuracy `0.459077388048`
  - transfer query accuracy `0.459033608437`
  - test loss `2.079487323761`

### Status
- v13 materially improves on the fixed v12 bridge
- v13 slightly beats the earlier v8 chart recovery on the stronger setting
- v13 still remains well below the v11 peak

---

## Session Entry: 2026-04-06 — Geometry-Native Bridge Switching

### Scope
- bounded within-sequence bridge switching on the stronger v12 family
- same unchanged downstream geometry-native engine
- same bounded bridge family as v13, but with one mid-sequence re-choice
- no live router changes
- no transformer blocks inside the geometry-native path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v14.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v14.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v14.csv` → created
- `docs/research/prime_transport_geometry_native_bridge_switching.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism selects one bridge for the first half and allows one
  recalibrated bridge choice for the second half using the same bounded
  support-window scoring rule as v13
- aggregate v14 geometry-native result:
  - transfer test accuracy `0.606119811535`
  - transfer query accuracy `0.660416662693`
  - test loss `5.524695396423`
- aggregate v14 tiny transformer result:
  - transfer test accuracy `0.444289445877`
  - transfer query accuracy `0.451562494040`
  - test loss `2.441468954086`

### Status
- v14 improves on v13
- v14 supports local temporal bridge recalibration over sequence-level bridge
  calibration alone
- v14 still remains well below the v11 peak

---

## Session Entry: 2026-04-06 — Geometry-Native Event Switch

### Scope
- bounded event-triggered bridge switching on the stronger v12 family
- same unchanged downstream geometry-native engine
- same tiny bridge family as v13/v14, but with one local-trigger switch rule
- no live router changes
- no transformer blocks inside the geometry-native path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v15.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v15.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v15.csv` → created
- `docs/research/prime_transport_geometry_native_event_switch.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism begins with the best early bridge and allows one later switch
  only when local coherence drops and an alternative bridge wins by a bounded
  margin
- aggregate v15 geometry-native result:
  - transfer test accuracy `0.591052830219`
  - transfer query accuracy `0.637893617153`
  - test loss `5.772159576416`
- aggregate v15 tiny transformer result:
  - transfer test accuracy `0.460844486952`
  - transfer query accuracy `0.467969596386`
  - test loss `2.170051097870`

### Status
- v15 still improves on the fixed v12 bridge
- v15 still beats the tiny transformer baseline
- v15 does not beat the simpler fixed midpoint switch from v14

---

## Session Entry: 2026-04-06 — Geometry-Native GCD Bridge Revision

### Scope
- bounded GCD-style factor-overlap bridge revision on the stronger v12 family
- same unchanged downstream geometry-native engine
- same tiny bridge family, but with retained-versus-replaced factor revision
  instead of whole-bridge switching
- no live router changes
- no transformer blocks inside the geometry-native path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v16.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v16.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v16.csv` → created
- `docs/research/prime_transport_geometry_native_gcd_bridge_revision.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism revises bridge suffixes by retaining shared prime factors and
  replacing only contradicted partner factors when local disagreement-rich
  evidence supports an alternative bridge
- aggregate v16 geometry-native result:
  - transfer test accuracy `0.592540919781`
  - transfer query accuracy `0.651993811131`
  - test loss `5.578721046448`
- aggregate v16 tiny transformer result:
  - transfer test accuracy `0.457589298487`
  - transfer query accuracy `0.469704806805`
  - test loss `2.272322416306`

### Status
- v16 still improves on the fixed v12 bridge
- v16 still beats the tiny transformer baseline
- v16 does not beat the current best v14 midpoint switch

---

## Session Entry: 2026-04-06 — Geometry-Native Conflict Revision

### Scope
- bounded query/binding-conflict-triggered factor revision on the stronger v12
  family
- same unchanged downstream geometry-native engine
- same tiny bridge family, but with explicit structural conflict as the
  revision trigger
- no live router changes
- no transformer blocks inside the geometry-native path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v17.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v17.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v17.csv` → created
- `docs/research/prime_transport_geometry_native_conflict_revision.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism opens one local revision window only when query-side and
  binding-side factor conflicts co-occur at a sufficiently high rate and an
  alternative bridge wins locally, then retains shared factors and replaces
  contradicted partners
- aggregate v17 geometry-native result:
  - transfer test accuracy `0.597052812576`
  - transfer query accuracy `0.643229186535`
  - test loss `5.901119232178`
- aggregate v17 tiny transformer result:
  - transfer test accuracy `0.459914058447`
  - transfer query accuracy `0.489458352327`
  - test loss `2.239838600159`

### Status
- v17 improves on weaker adaptive variants such as v15
- v17 still beats the fixed v12 bridge and the tiny transformer baseline
- v17 does not beat the current best v14 midpoint switch

---

## Session Entry: 2026-04-06 — Geometry-Native Microrepair

### Scope
- bounded micro-window conflict-centered factor repair on the stronger v12
  family
- same unchanged downstream geometry-native engine
- same tiny bridge family, but with very small local repair scope around the
  contradiction cluster
- no live router changes
- no transformer blocks inside the geometry-native path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v18.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v18.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v18.csv` → created
- `docs/research/prime_transport_geometry_native_microrepair.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism opens one very small repair window around a local
  query/binding contradiction cluster and retains shared factors while
  replacing only contradicted partner factors inside that micro-window
- aggregate v18 geometry-native result:
  - transfer test accuracy `0.580117166042`
  - transfer query accuracy `0.636718750000`
  - test loss `6.090361118317`
- aggregate v18 tiny transformer result:
  - transfer test accuracy `0.457589298487`
  - transfer query accuracy `0.495560944080`
  - test loss `2.003650903702`

### Status
- v18 still beats the fixed v12 bridge and the tiny transformer baseline
- v18 weakens relative to the broader factor-revision line
- v18 does not beat the current best v14 midpoint switch

---

## Session Entry: 2026-04-06 — Geometry-Native Mesorepair

### Scope
- bounded meso-window contradiction-centered factor repair on the stronger v12
  family
- same unchanged downstream geometry-native engine
- same tiny bridge family, but with a moderate regional repair window centered
  on the first strong contradiction cluster
- no live router changes
- no transformer blocks inside the geometry-native path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v19.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v19.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v19.csv` → created
- `docs/research/prime_transport_geometry_native_mesorepair.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism opens one moderate-sized repair region around the first strong
  query/binding contradiction cluster and retains shared factors while
  replacing contradicted partner factors only inside that region
- aggregate v19 geometry-native result:
  - transfer test accuracy `0.587890625000`
  - transfer query accuracy `0.645161271095`
  - test loss `5.766983985901`
- aggregate v19 tiny transformer result:
  - transfer test accuracy `0.447963953018`
  - transfer query accuracy `0.484281718731`
  - test loss `2.139752149582`

### Status
- v19 improves on the v18 micro-window repair
- v19 still beats the fixed v12 bridge and the tiny transformer baseline
- v19 does not beat the current best v14 midpoint switch

---

## Session Entry: 2026-04-06 — Geometry-Native Grown Repair

### Scope
- bounded contradiction-grown regional repair on the stronger v12 family
- same unchanged downstream geometry-native engine
- same tiny bridge family, but with repair boundaries expanded until
  arithmetic contradiction stabilizes
- no live router changes
- no transformer blocks inside the geometry-native path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v20.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v20.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v20.csv` → created
- `docs/research/prime_transport_geometry_native_grownrepair.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism seeds a small contradiction window and grows it outward while
  arithmetic conflict remains above a stabilization threshold, then applies
  retained-versus-replaced factor repair only inside the final grown region
- aggregate v20 geometry-native result:
  - transfer test accuracy `0.604437589645`
  - transfer query accuracy `0.643454015255`
  - test loss `5.770006656647`
- aggregate v20 tiny transformer result:
  - transfer test accuracy `0.457682281733`
  - transfer query accuracy `0.479310333729`
  - test loss `2.233343601227`

### Status
- v20 is the strongest factor-repair variant so far
- v20 still beats the fixed v12 bridge and the tiny transformer baseline
- v20 still does not beat the current best v14 midpoint switch

---

## Session Entry: 2026-04-06 — Geometry-Native Regional Schema Induction

### Scope
- bounded regional schema-induction / calibration on the stronger v12 family
- same unchanged downstream geometry-native engine
- same tiny divisibility-bridge family, with a tiny support-window prototype
  selector choosing among existing bounded strategies
- no live router changes
- no transformer blocks inside the geometry-native path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v21.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v21.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v21.csv` → created
- `docs/research/prime_transport_geometry_native_schema_induction.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism summarizes short support-window geometry/conflict statistics,
  fits one prototype centroid per bounded strategy on a calibration split, and
  then chooses among `v11_like`, `v12_base`, `hybrid`, `midpoint_switch`, and
  `grownrepair` before the unchanged geometry-native engine proceeds
- aggregate v21 geometry-native result:
  - transfer test accuracy `0.605003714561`
  - transfer query accuracy `0.686405777931`
  - test loss `5.769324779510`
- aggregate v21 tiny transformer result:
  - transfer test accuracy `0.438523054123`
  - transfer query accuracy `0.456230700016`
  - test loss `2.415073633194`

### Status
- v21 beats the handcrafted repair line, including v20 grownrepair
- v21 sets the best query accuracy so far on the stronger mismatch line
- v21 still narrowly misses the v14 midpoint switch on overall transfer accuracy

---

## Session Entry: 2026-04-06 — Geometry-Native Hybrid Regional Regime

### Scope
- bounded hybrid regional segmentation plus local schema induction on the
  stronger v12 family
- same unchanged downstream geometry-native engine
- same divisibility-bridge structure, with one coarse two-region split and a
  tiny regional schema selector inside each half
- no live router changes
- no transformer blocks inside the geometry-native path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v22.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v22.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v22.csv` → created
- `docs/research/prime_transport_geometry_native_hybrid_regime.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism splits each sequence into two coarse contiguous regions, fits
  support-window centroids per region on a calibration split, and then lets
  each region independently choose among `v11_like`, `v12_base`, and `hybrid`
  before the unchanged geometry-native engine proceeds
- aggregate v22 geometry-native result:
  - transfer test accuracy `0.646577358246`
  - transfer query accuracy `0.722751319408`
  - test loss `4.806250572205`
- aggregate v22 tiny transformer result:
  - transfer test accuracy `0.449311763048`
  - transfer query accuracy `0.455555558205`
  - test loss `2.245343446732`

### Status
- v22 is the strongest stronger-mismatch recovery so far
- v22 beats both the v14 overall leader and the v21 query-accuracy leader
- v22 is the clearest evidence so far for adaptive regional geometry

---

## Session Entry: 2026-04-06 — Geometry-Native Hybrid Boundary

### Scope
- bounded lightly induced split-boundary hybrid on the stronger v12 family
- same unchanged downstream geometry-native engine
- same two-region hybrid architecture as v22, but with boundary choice from a
  small candidate set instead of a fixed midpoint
- no live router changes
- no transformer blocks inside the geometry-native path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v23.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v23.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v23.csv` → created
- `docs/research/prime_transport_geometry_native_hybrid_boundary.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism keeps the same two-region hybrid regime logic as v22, but
  chooses the split boundary from a small bounded candidate set by regional
  prototype fit before applying local schema induction in each region
- aggregate v23 geometry-native result:
  - transfer test accuracy `0.647507429123`
  - transfer query accuracy `0.728548288345`
  - test loss `4.979188442230`
- aggregate v23 tiny transformer result:
  - transfer test accuracy `0.452194929123`
  - transfer query accuracy `0.465191572905`
  - test loss `2.128999710083`

### Status
- v23 is the new strongest stronger-mismatch recovery so far
- v23 improves on v22 on both overall accuracy and query accuracy
- the gain is incremental, which suggests boundary placement is now the main
  remaining lever

---

## Session Entry: 2026-04-06 — Geometry-Native Hybrid Boundary V24

### Scope
- bounded contradiction-aware boundary induction on the stronger v12 family
- same unchanged downstream geometry-native engine
- same two-region hybrid architecture as v23, but with a contradiction term
  added to the candidate boundary score and one local refinement step
- no live router changes
- no transformer blocks inside the geometry-native path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v24.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v24.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v24.csv` → created
- `docs/research/prime_transport_geometry_native_hybrid_boundary_v24.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism keeps the same two-region hybrid regime logic as v23, but
  scores candidate boundaries by prototype fit plus local contradiction near
  the split, with one local refinement step around the best candidate
- aggregate v24 geometry-native result:
  - transfer test accuracy `0.649925589561`
  - transfer query accuracy `0.719008266926`
  - test loss `4.979606151581`
- aggregate v24 tiny transformer result:
  - transfer test accuracy `0.446056544781`
  - transfer query accuracy `0.462293386459`
  - test loss `2.306171178818`

### Status
- v24 is the strongest stronger-mismatch result so far on overall accuracy
- v24 does not beat v23 on query accuracy
- the result suggests remaining gains are now mainly about boundary-objective
  tradeoff rather than missing hybrid architecture structure

---

## Session Entry: 2026-04-06 — Geometry-Native Hybrid Boundary V25

### Scope
- bounded two-objective / Pareto boundary scoring on the stronger v12 family
- same unchanged downstream geometry-native engine
- same two-region hybrid architecture as v22-v24, but with explicit
  two-objective boundary comparison over the same small candidate set
- no live router changes
- no transformer blocks inside the geometry-native path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v25.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v25.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v25.csv` → created
- `docs/research/prime_transport_geometry_native_hybrid_boundary_v25.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism keeps the same two-region hybrid regime logic, but compares
  candidate boundaries using a two-objective Pareto rule over prototype-fit
  quality and query-sensitive contradiction quality near the boundary
- aggregate v25 geometry-native result:
  - transfer test accuracy `0.634393572807`
  - transfer query accuracy `0.697614133358`
  - test loss `5.259162425995`
- aggregate v25 tiny transformer result:
  - transfer test accuracy `0.440011173487`
  - transfer query accuracy `0.453838169575`
  - test loss `2.288605690002`

### Status
- v25 does not improve the hybrid tradeoff
- v25 lands below both v23 and v24 on overall accuracy and query accuracy
- this is strong evidence that the bounded two-region hybrid line is near its
  local optimum on this task family

---

## Session Entry: 2026-04-06 — Geometry-Native Regime Field

### Scope
- bounded learned regional boundary/regime field on the stronger v12 family
- same unchanged downstream geometry-native engine
- same two-region hybrid architecture as v22-v25, with a tiny learned scorer
  selecting among the existing bounded split candidates
- no live router changes
- no transformer blocks inside the geometry-native computation path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v26.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v26.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v26.csv` → created
- `docs/research/prime_transport_geometry_native_regime_field.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism keeps the same two-region hybrid regime logic, but trains a
  tiny learned boundary field over the existing split candidates using compact
  regional geometry/conflict features and downstream sequence labels from a
  bounded calibration split
- aggregate v26 geometry-native result:
  - transfer test accuracy `0.654854893684`
  - transfer query accuracy `0.732780933380`
  - test loss `4.916808128357`
  - parameter count `6516`
- aggregate v26 tiny transformer result:
  - transfer test accuracy `0.461774557829`
  - transfer query accuracy `0.456240296364`
  - test loss `2.194231033325`

### Status
- v26 is the strongest stronger-mismatch result so far
- v26 beats the handcrafted hybrid family on both the best-overall and
  best-query reference points
- this is the clearest support so far for a learned regional regime field on
  top of geometry-native computation

---

## Session Entry: 2026-04-06 — Geometry-Native Regime Field V27

### Scope
- bounded stronger-family transfer test for the learned regional-field hybrid
- same unchanged downstream geometry-native engine
- same tiny learned regional field and same divisibility-bridge structure as
  v26, but evaluated on a longer and more entangled shifted family
- no live router changes
- no transformer blocks inside the geometry-native computation path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v27.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v27.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v27.csv` → created
- `docs/research/prime_transport_geometry_native_regime_field_v27.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism keeps the same tiny learned regional field as v26, but moves
  it onto a stronger shifted family with longer sequences, stronger latent-role
  entanglement, harsher lossy projection, and denser style/query/tag dynamics
- aggregate v27 geometry-native result:
  - transfer test accuracy `0.540852844715`
  - transfer query accuracy `0.584938704967`
  - test loss `6.327512264252`
  - parameter count `6516`
- aggregate v27 tiny transformer result:
  - transfer test accuracy `0.383219391108`
  - transfer query accuracy `0.435639232397`
  - test loss `2.621042013168`

### Status
- v27 still beats the tiny transformer baseline clearly under stronger shift
- v27 loses a large amount of the v26 margin from the prior family
- the result supports the architecture direction but shows the current tiny
  learned field is not yet robust enough for stronger family transfer

---

## Session Entry: 2026-04-06 — Geometry-Native Regime Field V28

### Scope
- slightly richer learned regional regime field on the same stronger shifted
  family as v27
- same unchanged downstream geometry-native engine
- same bounded split candidates and same local schema/bridge machinery, but
  with a slightly richer learned scorer
- no live router changes
- no transformer blocks inside the geometry-native computation path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v28.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v28.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v28.csv` → created
- `docs/research/prime_transport_geometry_native_regime_field_v28.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism keeps the same stronger shifted family and same bounded
  regional decomposition as v27, but uses a slightly richer learned scorer
  with a wider hidden projection and one extra mixing stage
- aggregate v28 geometry-native result:
  - transfer test accuracy `0.548014342785`
  - transfer query accuracy `0.584723412991`
  - test loss `6.508901119232`
  - parameter count `7316`
- aggregate v28 tiny transformer result:
  - transfer test accuracy `0.369710296392`
  - transfer query accuracy `0.438542574644`
  - test loss `2.805939912796`

### Status
- v28 improves modestly on v27 in overall accuracy
- v28 does not improve on v27 in query accuracy
- the result suggests the next bottleneck is field structure rather than a
  small increase in learned scorer capacity

---

## Session Entry: 2026-04-06 — Geometry-Native Regime Field V29

### Scope
- structured learned regional field on the same stronger shifted family as
  v27-v28
- same unchanged downstream geometry-native engine
- same bounded split candidates and same local schema/bridge machinery, but
  with a tiny convolutional field over the ordered candidate boundaries
- no live router changes
- no transformer blocks inside the geometry-native computation path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v29.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v29.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v29.csv` → created
- `docs/research/prime_transport_geometry_native_regime_field_v29.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism keeps the same stronger shifted family and same bounded split
  candidates as v27-v28, but adds a tiny structured field over the ordered
  boundary lattice so nearby split candidates share evidence before selection
- aggregate v29 geometry-native result:
  - transfer test accuracy `0.549641907215`
  - transfer query accuracy `0.589491665363`
  - test loss `6.501174449921`
  - parameter count `8676`
- aggregate v29 tiny transformer result:
  - transfer test accuracy `0.386393219233`
  - transfer query accuracy `0.457923978567`
  - test loss `2.942277193069`

### Status
- v29 improves on both v27 and v28 on the stronger shifted family
- v29 suggests structured regional field topology helps more than simple
  scorer-width increases
- v29 still remains far below the earlier-family v26 margin

---

## Session Entry: 2026-04-06 — Geometry-Native Regime Field V30

### Scope
- explicit structured chart field over contiguous regions on the same stronger
  shifted family as v27-v29
- same unchanged downstream geometry-native engine
- same local schema/bridge machinery, but with a tiny learned per-block chart
  field decoded into at most three contiguous regions
- no live router changes
- no transformer blocks inside the geometry-native computation path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v30.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v30.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v30.csv` → created
- `docs/research/prime_transport_geometry_native_regime_field_v30.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism replaces split-candidate scoring with an explicit piecewise
  chart field over contiguous blocks, then collapses that field into up to
  three contiguous regions and applies the existing geometry-native local
  variants inside each region
- aggregate v30 geometry-native result:
  - transfer test accuracy `0.543050110340`
  - transfer query accuracy `0.596061289310`
  - test loss `6.582681179047`
  - parameter count `8078`
- aggregate v30 tiny transformer result:
  - transfer test accuracy `0.391276031733`
  - transfer query accuracy `0.441137850285`
  - test loss `2.638148069382`

### Status
- v30 improves on v29 in query accuracy but not in overall transfer accuracy
- v30 remains clearly better than the tiny transformer baseline
- explicit contiguous chart structure looks meaningful, but it is not yet a
  decisive recovery of the lost v26 stronger-family margin

---

## Session Entry: 2026-04-06 — Geometry-Native Regime Field V31

### Scope
- small global learned chart field on the same stronger shifted family as
  v27-v30
- same unchanged downstream geometry-native engine
- same local schema/bridge machinery, but with a broader low-rank global chart
  map over coarse blocks before bounded contiguous decoding
- no live router changes
- no transformer blocks inside the geometry-native computation path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v31.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v31.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v31.csv` → created
- `docs/research/prime_transport_geometry_native_regime_field_v31.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism uses a tiny low-rank global chart map over coarse blocks to
  modulate local blockwise chart scores before bounded contiguous decoding
- aggregate v31 geometry-native result:
  - transfer test accuracy `0.527669250965`
  - transfer query accuracy `0.581842005253`
  - test loss `6.738704204559`
  - parameter count `6914`
- aggregate v31 tiny transformer result:
  - transfer test accuracy `0.381754547358`
  - transfer query accuracy `0.432125717402`
  - test loss `2.734487295151`

### Status
- v31 remains clearly better than the tiny transformer baseline
- v31 does not improve on the better shifted-family structured-field results
  from v29 or v30
- broader chart-field scope alone does not recover the lost v26 margin on the
  stronger shifted family

---

## Session Entry: 2026-04-06 — Geometry-Native Regime Field V32

### Scope
- bounded multiscale chart field on the same stronger shifted family as
  v27-v31
- same unchanged downstream geometry-native engine
- same local schema/bridge machinery, but with local fine-block chart scores
  plus a nearby coarse regional chart coordinator
- no live router changes
- no transformer blocks inside the geometry-native computation path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v32.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v32.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v32.csv` → created
- `docs/research/prime_transport_geometry_native_regime_field_v32.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism uses a two-level multiscale chart field: local fine-block
  chart scores plus a small coarse neighboring coordinator before bounded
  contiguous decoding
- aggregate v32 geometry-native result:
  - transfer test accuracy `0.547607421875`
  - transfer query accuracy `0.579540550709`
  - test loss `6.906696319580`
  - parameter count `6537`
- aggregate v32 tiny transformer result:
  - transfer test accuracy `0.384521484375`
  - transfer query accuracy `0.444299966097`
  - test loss `2.868544340134`

### Status
- v32 clearly beats the tiny transformer baseline
- v32 recovers some overall accuracy relative to the weaker global-field
  variant from v31
- v32 still does not beat the better shifted-family structured-field results
  from v29 or v30

---

## Session Entry: 2026-04-06 — Geometry-Native Regime Field V33

### Scope
- sparse region-interaction field on the same stronger shifted family as
  v29-v32
- same unchanged downstream geometry-native engine
- same local schema/bridge machinery, but with one bounded region-level
  interaction pass over adjacent regions after a v30-style decomposition
- no live router changes
- no transformer blocks inside the geometry-native computation path

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_v33.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_v33.py` → created
- `results/prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v33.csv` → created
- `docs/research/prime_transport_geometry_native_regime_field_v33.md` → created
- `EVIDENCE_SUMMARY.md`, `results/INDEX.md`, `docs/research/SESSION_LEDGER.md` → updated

### Result
- the mechanism keeps a v30-style contiguous decomposition, then passes one
  sparse adjacent-region interaction round using neighbor average, difference,
  and product features before choosing each region's geometry-native variant
- aggregate v33 geometry-native result:
  - transfer test accuracy `0.539062500000`
  - transfer query accuracy `0.578022003174`
  - test loss `6.073722839355`
  - parameter count `10289`
- aggregate v33 tiny transformer result:
  - transfer test accuracy `0.386962890625`
  - transfer query accuracy `0.443076908588`
  - test loss `2.301008701324`

### Status
- v33 remains clearly better than the tiny transformer baseline
- v33 does not improve on the stronger shifted-family regional/structured
  leaders from v29 or v30
- this small sparse routing-style coordination layer does not appear to be the
  missing piece on this shifted-family benchmark

---

## Session Entry: 2026-04-06 — Prime Transport Inner Architecture Audit

### Scope
- audit the inner bundle/orbit/transport representation used by the direct
  sequence-model line
- compare intended exact-layer / routing abstractions against the actual
  implementation path in v3, v7, v11-v14, v21, and v29-v33
- no live router changes
- no new benchmark mechanism

### Files Added / Updated
- `docs/research/prime_transport_inner_architecture_audit.md` → created
- `results/prime_transport_recursive_system/prime_transport_inner_architecture_audit.csv` → created
- `docs/research/SESSION_LEDGER.md` → updated

### Result
- the current sequence-model line does contain explicit discrete chart counters
  and bounded discourse memory
- but the learned computation is still a snapshot-feature encoder plus MLP
  readout, not a native bundle/orbit transport engine
- divisibility / semiprime transport is represented only as a temporary rewrite
  of `referent_role` and `referent_entity`, not as persistent state
- orbital/spin coordinates and admissibility filtering are not natively present
  in the sequence-model computation path

### Status
- the single most likely gap is in the inner representation, not just the
  wrapper/search branch
- recommended next branch: rebuild part of the inner representation

---

## Session Entry: 2026-04-06 — Prime Transport Inner Representation Rebuild V1

### Scope
- start the first direct rebuild of the inner representation on the bounded v3
  discourse/query task
- introduce native persistent composite transport state, chart/spin
  coordinates, and admissibility-constrained transitions into the real
  computation path
- no live router changes
- no further wrapper or field tuning

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_r1.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_r1.py` → created
- `docs/research/prime_transport_inner_representation_rebuild_v1.md` → created
- `results/prime_transport_recursive_system/prime_transport_inner_representation_rebuild_v1.csv` → created
- `docs/research/SESSION_LEDGER.md` → updated

### Result
- r1 does successfully move three intended pieces into the actual computation
  path:
  - persistent semiprime transport state
  - native cyclic chart / spin-style coordinates
  - active admissibility projection on state transitions
- but performance drops sharply relative to the v3 approximate geometry-native
  baseline:
  - v3 reference: test accuracy `0.9978`, query accuracy `0.9878`
  - r1 rebuild: test accuracy `0.7361`, query accuracy `0.7009`
  - tiny transformer: test accuracy `0.6958`, query accuracy `0.6707`

### Status
- this rebuild is a real architectural clarification, not a performance win
- the inner representation was indeed missing from the computation path
- once part of it is restored, the current feature-vector-plus-MLP readout
  proves too weak to use the rebuilt state well
- next smallest honest rebuild step: replace immediate flattening with a native
  composite/chart state update-and-readout block

---

## Session Entry: 2026-04-06 — Prime Transport Inner Representation Rebuild V2

### Scope
- remove the remaining flatten-to-feature-vector plus MLP core from `r1`
- keep the bounded v3 discourse/query task as the comparison surface
- keep the native semiprime/chart/admissibility state from `r1`
- rebuild the learned path as a structured native readout over that state

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_r2.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_r2.py` → created
- `docs/research/prime_transport_inner_representation_rebuild_v2.md` → created
- `results/prime_transport_recursive_system/prime_transport_inner_representation_rebuild_v2.csv` → created
- `docs/research/SESSION_LEDGER.md` → updated

### Result
- `r2` does remove the core flatten-to-MLP path
- the learned core is now a structured table-driven readout over native state:
  - chart indices
  - semiprime pair state
  - semiprime overlap
  - admissibility bit
  - spin bits
  - dynamic role-to-entity / role-to-tag mappings
- bounded performance relative to the same v3 task:
  - v3 reference: test accuracy `0.9978`, query accuracy `0.9878`
  - r1 rebuild: test accuracy `0.7361`, query accuracy `0.7009`
  - r2 rebuild: test accuracy `0.7426`, query accuracy `0.7468`
  - tiny transformer: test accuracy `0.6958`, query accuracy `0.6707`

### Status
- `r2` is still far from the old approximate v3 line
- but it is a real architectural improvement over `r1`
- the core learned path is now structurally honest, and query behavior improves
  materially with only `219` learned parameters
- next smallest honest rebuild step: rebuild the native transition operator
  itself, not just the readout

---

## Session Entry: 2026-04-06 — Prime Transport Inner Representation Rebuild V3

### Scope
- replace the remaining hand-built proposal-plus-projection transition from
  `r1`/`r2`
- keep the bounded v3 discourse/query task
- keep the no-flatten structured readout from `r2`
- rebuild transitions as a native factor-space transport operator

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_r3.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_r3.py` → created
- `docs/research/prime_transport_inner_representation_rebuild_v3.md` → created
- `results/prime_transport_recursive_system/prime_transport_inner_representation_rebuild_v3.csv` → created
- `docs/research/SESSION_LEDGER.md` → updated

### Result
- `r3` fully removes proposal-plus-projection
- the transition is now native factor transport:
  - decode semiprimes into factor pairs
  - retain one anchor factor
  - transport the partner factor using chart, discourse, spin, token, and
    cross-composite signals
  - recompose directly into the next semiprime
- bounded performance:
  - v3 reference: test accuracy `0.9978`, query accuracy `0.9878`
  - r1 rebuild: test accuracy `0.7361`, query accuracy `0.7009`
  - r2 rebuild: test accuracy `0.7426`, query accuracy `0.7468`
  - r3 rebuild: test accuracy `0.7424`, query accuracy `0.7504`
  - tiny transformer: test accuracy `0.6958`, query accuracy `0.6707`

### Status
- `r3` is not a performance recovery
- but it does complete the third honest rebuild step:
  - native state
  - native readout
  - native transition
- query behavior improves slightly over `r2`
- admissibility is now satisfied by construction rather than repair
- next smallest honest rebuild step: learn the factor-space transport law
  itself without reintroducing flattening or projection

---

## Session Entry: 2026-04-06 — Prime Transport Inner Representation Rebuild V4

### Scope
- replace the hand-designed factor transport law from `r3`
- keep the bounded v3 discourse/query task
- keep native persistent composite state, native structured readout, and
  admissibility by construction
- learn partner-factor motion directly over native structured state

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_r4.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_r4.py` → created
- `docs/research/prime_transport_inner_representation_rebuild_v4.md` → created
- `results/prime_transport_recursive_system/prime_transport_inner_representation_rebuild_v4.csv` → created
- `docs/research/SESSION_LEDGER.md` → updated

### Result
- `r4` learns the transport law as an additive native factor-space kernel over:
  - mode
  - token
  - retained anchor factor
  - coupled opposite factor
  - chart turn
  - discourse turn
  - spin turn
  - tag turn
  - query-token flag
  - tag-token flag
- no flatten-to-MLP path was reintroduced
- no proposal-plus-projection path was reintroduced
- admissibility remains satisfied by construction:
  - `admissible_mean = 1.0`
  - `admissible_min = 1`
  - `admissible_max = 1`
- bounded performance:
  - v3 reference: test accuracy `0.9978`, query accuracy `0.9878`
  - r1 rebuild: test accuracy `0.7361`, query accuracy `0.7009`
  - r2 rebuild: test accuracy `0.7426`, query accuracy `0.7468`
  - r3 rebuild: test accuracy `0.7424`, query accuracy `0.7504`
  - r4 rebuild: test accuracy `0.7500`, query accuracy `0.7862`
  - tiny transformer: test accuracy `0.6958`, query accuracy `0.6707`

### Status
- `r4` is the first clear positive result after the inner rebuild branch began
- the last major hand-designed bottleneck is now replaced with a learned native
  transport law
- the rebuilt line still trails the old approximate v3 reference badly, but it
  is now both more honest architecturally and better than `r1`-`r3`
- next smallest honest rebuild step: learn anchor selection or a richer
  chart-conditioned transport kernel without reintroducing flattening or repair

---

## Session Entry: 2026-04-06 — Prime Transport Inner Representation Rebuild V5

### Scope
- replace the remaining hand-designed anchor choice from `r4`
- keep the bounded v3 discourse/query task
- keep native persistent composite state, native structured readout, and native
  learned partner transport
- learn anchor selection directly over native factor/composite/chart state

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_r5.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_r5.py` → created
- `docs/research/prime_transport_inner_representation_rebuild_v5.md` → created
- `results/prime_transport_recursive_system/prime_transport_inner_representation_rebuild_v5.csv` → created
- `docs/research/SESSION_LEDGER.md` → updated

### Result
- `r5` learns anchor selection as additive logits over:
  - mode
  - token
  - candidate factor
  - other same-semiprime factor
  - opposite composite left/right factors
  - chart turn
  - discourse turn
  - spin turn
  - tag turn
  - query-token flag
  - tag-token flag
- no flatten-to-MLP path was reintroduced
- no projection path was reintroduced
- admissibility remains satisfied by construction:
  - `admissible_mean = 1.0`
  - `admissible_min = 1`
  - `admissible_max = 1`
- bounded performance:
  - v3 reference: test accuracy `0.9978`, query accuracy `0.9878`
  - r1 rebuild: test accuracy `0.7361`, query accuracy `0.7009`
  - r2 rebuild: test accuracy `0.7426`, query accuracy `0.7468`
  - r3 rebuild: test accuracy `0.7424`, query accuracy `0.7504`
  - r4 rebuild: test accuracy `0.7500`, query accuracy `0.7862`
  - r5 rebuild: test accuracy `0.7503`, query accuracy `0.7927`
  - tiny transformer: test accuracy `0.6958`, query accuracy `0.6707`

### Status
- `r5` is a small but real improvement over `r4`
- the next major hand-designed bottleneck in the rebuilt line is now gone
- the rebuilt architecture still trails the old approximate v3 reference by a
  large margin, but the native line continues improving without giving up its
  architectural constraints
- next smallest honest rebuild step: learn a richer joint anchor-plus-transport
  kernel over the same native state

---

## Session Entry: 2026-04-06 — Prime Transport Inner Representation Rebuild V6

### Scope
- replace the separate learned anchor selector and learned partner transport
  from `r5`
- keep the bounded v3 discourse/query task
- keep native persistent composite state, native structured readout, and
  admissibility by construction
- learn one joint anchor-plus-partner transition operator over native state

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_r6.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_r6.py` → created
- `docs/research/prime_transport_inner_representation_rebuild_v6.md` → created
- `results/prime_transport_recursive_system/prime_transport_inner_representation_rebuild_v6.csv` → created
- `docs/research/SESSION_LEDGER.md` → updated

### Result
- `r6` learns one joint structured kernel over six admissible
  `(anchor choice, partner factor)` transitions
- no flatten-to-MLP path was reintroduced
- no projection path was reintroduced
- admissibility remains satisfied by construction:
  - `admissible_mean = 1.0`
  - `admissible_min = 1`
  - `admissible_max = 1`
- bounded performance:
  - v3 reference: test accuracy `0.9978`, query accuracy `0.9878`
  - r1 rebuild: test accuracy `0.7361`, query accuracy `0.7009`
  - r2 rebuild: test accuracy `0.7426`, query accuracy `0.7468`
  - r3 rebuild: test accuracy `0.7424`, query accuracy `0.7504`
  - r4 rebuild: test accuracy `0.7500`, query accuracy `0.7862`
  - r5 rebuild: test accuracy `0.7503`, query accuracy `0.7927`
  - r6 rebuild: test accuracy `0.7382`, query accuracy `0.7310`
  - tiny transformer: test accuracy `0.6958`, query accuracy `0.6707`

### Status
- `r6` is a clean negative relative to `r5`
- the joint operator is architecturally honest, but the smallest additive joint
  kernel is worse than the separate learned anchor-plus-partner decomposition
- if this branch continues, the next honest move is a richer native joint
  kernel with stronger factor interaction terms, not a return to wrappers or
  projection

---

## Session Entry: 2026-04-06 — Prime Transport Radial Compatibility Audit

### Scope
- audit the current native `r6` transport system for radial/fiber/spin class
  incompatibility
- no model improvement step
- no learned changes
- optional constrained evaluation only

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_r6_compat_audit.py` → created
- `docs/research/prime_transport_radial_compatibility_audit.md` → created
- `results/prime_transport_recursive_system/prime_transport_radial_compatibility_audit.csv` → created
- `docs/research/SESSION_LEDGER.md` → updated

### Result
- incompatible class comparison/transport is common in `r6`
- selected transitions:
  - radial/fiber mismatched `50.82%`
  - spin mismatched `70.90%`
- candidate-set pollution is high:
  - average incompatible candidates per side-step `5.3117 / 6`
- a simple compatibility filter improves evaluation even without retraining:
  - loss `0.6617 -> 0.6512`
  - accuracy `0.7382 -> 0.7436`
  - query accuracy `0.7310 -> 0.7518`

### Status
- current `r6` does compare/transport across proxy-incompatible radial/fiber
  and spin classes often
- the mismatch correlation is not perfectly clean at the raw transition level
- but the candidate pollution plus filtered-eval improvement make this look
  like a real structural bottleneck in the rebuilt line

---

## Session Entry: 2026-04-06 — Prime Transport Inner Representation Rebuild V7

### Scope
- rebuild the native joint transport operator so compatibility is part of the
  transport law itself
- use the bounded radial/fiber, spin, and composite compatibility proxies from
  the compatibility audit
- no wrapper tuning, no projection, no flattening, no post-hoc evaluation-only
  filter

### Files Added / Updated
- `tools/prime_transport/geometry_native_sequence_model_r7.py` → created
- `tools/prime_transport/run_geometry_native_sequence_model_r7.py` → created
- `docs/research/prime_transport_inner_representation_rebuild_v7.md` → created
- `results/prime_transport_recursive_system/prime_transport_inner_representation_rebuild_v7.csv` → created
- `docs/research/SESSION_LEDGER.md` → updated

### Result
- `r7` moves compatibility into the native transport law:
  - only candidates in the best available compatibility tier are scored by the
    learned joint operator
  - composite-compatibility mismatches are eliminated in practice:
    - selected compat mismatch fraction `0.0000`
    - tier-4 usage `0.0000`
- bounded performance on the v3 discourse/query task:
  - `r6`: test accuracy `0.7382`, query accuracy `0.7310`, loss `0.6617`
  - filtered `r6` audit reference: test accuracy `0.7436`, query accuracy
    `0.7518`, loss `0.6512`
  - `r7`: test accuracy `0.7400`, query accuracy `0.7339`, loss `0.6685`
- compatibility remains only partially satisfied:
  - selected radial/fiber mismatch fraction `0.5082`
  - selected spin mismatch fraction `0.6503`
  - compared incompatible fraction among scored candidates `0.7956`
  - average scored candidates per side-step `2.0104`

### Status
- compatibility is now inside the native transport operator itself
- but `r7` does not recover the gains seen in the compatibility-filtered audit
- the result is architecturally positive and empirically negative
- the main remaining bottleneck appears to be the native compatibility-aware
  move geometry, not the absence of compatibility conditioning itself
