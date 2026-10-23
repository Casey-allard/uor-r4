# ORDER: m5-laptop-full-path-measurement, antigravity-31

Lab: antigravity. Item: [m5-laptop-full-path-measurement](../../../../roadmap/items/m5-laptop-full-path-measurement.md). Base: `3ceb5257b`.

## Goal

Address all 5 remaining findings and the symbol/instruction-count verification note from [Claude's re-review of PR `#3012` at `3ceb5257b6d71401bbefe9f358ba94b7541c49cc` (`6355794112`)](https://github.com/UOR-Foundation/uor-r4/pull/3012#issuecomment-6355794112):
1. **Audit `alloc::` callees in the transitive call-graph walk (`report_output.rs`):** Remove bare `"alloc"`, `"free"`, and `"realloc"` from `ALLOW_PATTERNS` in `is_call_graph_allowlisted` and allow only exact raw allocator entry points (`EXACT_ALLOC_ENTRYPOINTS`) plus `"alloc::raw_vec"`, `"5alloc7raw_vec"`, `"handle_alloc_error"`, and `"precondition_check"` so all `alloc::vec::*` (`SpecFromIter::from_iter`, etc.) and `alloc::slice::*` callees are audited, and plant a multiply inside `<alloc::vec::Vec<i32> as alloc::vec::spec_from_iter::SpecFromIter<i32, I>>::from_iter`.
2. **Fix `clippy -D warnings` (`report_output.rs:1167`, `uor-r4-stack.rs:426`):** Collapse the nested `if` at `report_output.rs:1167` and remove `.enumerate()` at `uor-r4-stack.rs:426`.
3. **Close the remaining x86-64 and AArch64 opcode gaps & document `ldr d0..d31`:** Add x86-64 AT&T `mulxq`/`mulxl`, `sqrtsd`/`sqrtss`/`vsqrtsd`/`vsqrtss`/`sqrtps`/`sqrtpd`, `ucomisd`/`ucomiss`/`comisd`/`comiss`, `cvt*`/`vcvt*`, `vpmullq`/`pmullq`, and `vpdpbusd`/`vpdpbusds`/`vpdpwssd`/`vpdpwssds`, plus AArch64 `sqrdmlah`/`sqrdmlsh`/`sqrdmlah2`/`sqrdmlsh2` and `h0..h31` scalar FP16 register loads/stores; state the `ldr d0..d31` 64-bit NEON 2x`i32` (`sqadd.2s`) rule in `disassembly_audit.scope` and Protocol v3.
4. **Reject indirect calls in audited symbols:** Flag indirect calls (`blr`, `blrda`, `blrdb`, `blraa`, `blrab`, `call *`, `callq *`) outside `rayon_core`/TLS wrappers, add planted `blr x8` and `callq *%rax` cancel probes, and state this in `disassembly_audit.scope`.
5. **Fix the v2 3-run diagnostic label & print exact symbol/instruction counts:** Label the archived v2 3-run CI95 diagnostic as `"serving.energy.net_j_per_tok (vs reconstructed gross_j_per_tok 3-run SD)"` and print/assert the exact `disassembly_audit` counts in `m5_full_path_cost_controls.rs` and `REPORT.md`.

## Cancel probe (run first)

Planting `mul x0, x1, x2` inside an `<alloc::vec::Vec<i32> as alloc::vec::spec_from_iter::SpecFromIter<i32, I>>::from_iter` callee, `mulxq %rcx, %rbx, %rax`, or `blr x8` inside `IntegerStackSession::step` must fail `verify_step_disassembly_text`.

## Files in scope

- `crates/uor-r4-integer/**`
- `docs/compute/energy-measurement-protocol-v3.md`
- `docs/labs/m5-laptop-full-path-measurement/**`
- `docs/loop/tasks/m5-laptop-full-path-measurement/**`

## Done when

- All 5 findings are fixed, `m5_full_path_cost_controls` passes, and `uor-harness loop pr-check --local` passes.

## Stop and ask if

- Any file outside `scope` must be modified.
