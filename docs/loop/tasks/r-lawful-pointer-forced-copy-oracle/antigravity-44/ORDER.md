# ORDER: r-lawful-pointer-forced-copy-oracle, antigravity-44

Lab: antigravity. Item: [r-lawful-pointer-forced-copy-oracle](../../../../roadmap/items/r-lawful-pointer-forced-copy-oracle.md). Base: `ac3b556a4`.

## Goal

Address all blocking and should-fix findings from Claude's exact-head review (`issuecomment-6371944551`) on PR `#3086` (`r-lawful-pointer-forced-copy-oracle`, `#2880`):
1. **Unconfound `O1` vs `N` (`crates/uor-r4-training/src/lawful_pointer.rs`):** Keep the base-margin comparison `total_copy_margin > base_top_margin` in `O1SpanLearnedGate` using the forced span's canonical route score (`oracle_route_score`), so `O1` and `N` share the exact same gating/margin law and `O1 - N` isolates source addressing alone.
2. **Pin all selection and tie-break clauses in finite unit tests (`crates/uor-r4-training/src/lawful_pointer.rs`):**
   - Pin `total_copy_margin > base_top_margin` with a test case where `learned_gate_margin >= gate_threshold_margin` (`120 >= 100`) and the span is admitted, but `base_top_margin` outscores `total_copy_margin` (`1500 >= 1474`), verifying both `N` and `O1` emit `base_top_token` (`copied == false`).
   - Pin both the gate-closed and unadmitted-candidate branches of `CopySpanOracle::Off`.
   - Pin `RouteAdmissionMode::SharedAtomGcd` prime rarity ranking (`ilog2_gcd_weight_q8`) and the latest-span-start (`span.0`) tie-break rule.
3. **Enforce alphanumeric word/token boundaries in `locate_last_value_span`:** Reject substring matches inside longer alphanumeric tokens (so searching for `"4"` never matches inside `"47"` or `"14"`), with a dedicated unit test.
4. **Update `docs/labs/r-lawful-pointer-forced-copy-oracle/README.md` and `docs/harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md`:** Document the unconfounded `O1` comparison, make the diagnostic outcome branches (`A`, `B`, `C`, `D`) a total partition, add the wrong-span distractor negative control (`W`), specify the 16-block paired cluster SE $\text{SE}_{\Delta}$ alongside the unconditional cluster SE, pin seeds `1, 2, 3`, note the checkpoint `learned_gate_margin` requirement, and update Stop 1(d) to chat approval (`#2895`).

## Cancel probe (run first)

- Probe: `uor-harness loop gates --quick` (cancelled with SIGINT)
- Cancel signal: `SIGINT` (`exit 130`), clean exit in `< 1s`.

## Files in scope

- `crates/uor-r4-training/src/lawful_pointer.rs`
- `docs/harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md`
- `docs/labs/r-lawful-pointer-forced-copy-oracle/README.md`
- `docs/loop/tasks/r-lawful-pointer-forced-copy-oracle/antigravity-44/ORDER.md`
- `docs/loop/tasks/r-lawful-pointer-forced-copy-oracle/antigravity-44/REPORT.md`

## Done when

- `cargo test -p uor-r4-training --lib lawful_pointer` (`9 / 9` tests pass), `cargo check -p uor-r4-training --bins`, and `cargo fmt --check -p uor-r4-training` pass.
- `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness check --only harness,prep` pass.

## Stop and ask if

- Any file outside `r-lawful-pointer-forced-copy-oracle`'s `scope:` would need to change.
