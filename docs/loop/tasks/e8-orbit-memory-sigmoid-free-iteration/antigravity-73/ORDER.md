# ORDER: e8-orbit-memory-sigmoid-free-iteration, antigravity-73

Lab: antigravity. Item: [e8-orbit-memory-sigmoid-free-iteration](../../../../roadmap/items/e8-orbit-memory-sigmoid-free-iteration.md) (#2822). Base: `d9dfbdb401f66d749afe4215afdb0a1370168852`.

## Goal

The item's done-when reads:

> A sigmoid-free gate form, with the record's named next iteration, runs on the GPU and is compared with learned keys, reporting the 0.023 gap.

Under `docs/roadmap/items/e8-orbit-memory-sigmoid-free-iteration.md`'s docs-only scope (`docs/harness/prep/e8-orbit-memory-sigmoid-free-iteration*`, `docs/labs/e8-orbit-memory-sigmoid-free-iteration/**`, `docs/loop/tasks/e8-orbit-memory-sigmoid-free-iteration/**`), stage the pre-registration sheet `docs/harness/prep/e8-orbit-memory-sigmoid-free-iteration-2026-10-25.md` with a non-author `TEST FITNESS: FIT` review (`CHARTER.md` §4.5) and the companion design & runbook record `docs/labs/e8-orbit-memory-sigmoid-free-iteration/README.md`, and complete a D68 decision review (`CHARTER.md` §5.1) setting `status: held` until `crates/uor-r4-float-train` E8-memory CUDA kernel wiring and `d11-exporter-float-trunk` (`CHARTER.md` §3.7) land.

## Cancel probe (run first)

Ran on `d9dfbdb401f66d749afe4215afdb0a1370168852`:

```sh
ls -1 crates/uor-r4-float-train/src/
command grep -n "exp()" crates/uor-r4-training/src/e8_memory.rs
command grep -n "e8.query.weight" crates/uor-r4-training/src/geometric_stack.rs
command grep -n "to_vec1::<f32>" crates/uor-r4-training/src/e8_memory.rs
```

**Result (`instrument` — scope & dependency barrier, not a mechanism negative):**
1. **Item scope excludes crate code:** `docs/roadmap/items/e8-orbit-memory-sigmoid-free-iteration.md` scopes only `docs/harness/prep/e8-orbit-memory-sigmoid-free-iteration*`, `docs/labs/e8-orbit-memory-sigmoid-free-iteration/**`, and `docs/loop/tasks/e8-orbit-memory-sigmoid-free-iteration/**`.
2. **No compliant E8-memory implementation exists in `crates/uor-r4-float-train`:** `crates/uor-r4-float-train/src/` (`#2823`) has no `e8_memory` module. The legacy `crates/uor-r4-training/src/e8_memory.rs` and `geometric_stack.rs` contain three `CHARTER.md` §3 blockers (`docs/mechanisms/e8-orbit-memory.md:40-44, 82-87`): (a) `e8_memory.rs:415-421` uses a banned logistic sigmoid gate (`exp()` + `recip()`, violating §3.2 / D23); (b) `geometric_stack.rs:5995` uses a banned dense query matmul (`linear(u, e8.query.weight)`, violating §3.3 / D25/D30); and (c) `e8_memory.rs:339, 405` copies `query` to the host (`to_vec1::<f32>()`) and runs `e8_select` on the CPU via Rayon (violating §3.5 / D24). It also restricts `k` to `{1, 2, 4, 8}` (`e8_memory.rs:351`) without the record's named next iteration (`k = 16` + per-orbit learned residual offset $\delta_{h,o} \in \mathbb{R}^8$).
3. **`d11-exporter-float-trunk` is not yet built:** `docs/roadmap/items/d11-exporter-float-trunk.md` is still `status: ready`, and `CHARTER.md` §3.7 (D37) requires every offline-float candidate to export to D11 integer form and report its export gap.

## Files in scope

- `docs/harness/prep/e8-orbit-memory-sigmoid-free-iteration*`
- `docs/labs/e8-orbit-memory-sigmoid-free-iteration/**`
- `docs/loop/tasks/e8-orbit-memory-sigmoid-free-iteration/**`
- `docs/roadmap/items/e8-orbit-memory-sigmoid-free-iteration.md` (status line)
- `docs/roadmap/README.md` (shared registration file)

## Done when

- `docs/harness/prep/e8-orbit-memory-sigmoid-free-iteration-2026-10-25.md` and `docs/labs/e8-orbit-memory-sigmoid-free-iteration/README.md` are committed with a non-author `TEST FITNESS: FIT` review (`CHARTER.md` §4.5);
- A fresh-context D68 decision review (`CHARTER.md` §5.1) approves setting `status: held` on `e8-orbit-memory-sigmoid-free-iteration`, logged on `#2822`;
- `uor-harness loop pr-check --local` and `uor-harness loop gates --full` pass at the head (`References #2822`).

## Stop and ask if

- Any file outside the declared scope is needed;
- Any check in `uor-harness loop gates --full` fails.
