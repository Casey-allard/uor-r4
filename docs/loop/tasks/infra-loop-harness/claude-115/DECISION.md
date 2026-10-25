# Decision card (D68): a new M1 item for the D11 exporter CLI, GPU panel scorer and tok/s harness

Lab: claude, claude-115. New item: [`m1-d11-artifact-scorer`](../../../../roadmap/items/m1-d11-artifact-scorer.md) (#3248). Status: **approved**. The fresh decision review returned APPROVE in round 1; its exact-head CHANGES are applied.

## Why, and why now

- **The chain.** M1 closes on the saved-model path (D79). Its bar is read "through the D11 export … on the same DEV panel, with tokens/s reported" (CHARTER §4.8). PLAN.md step 6 (`m1-export-gap-and-toks`, #2839) measures that, and steps 7 and 8 depend on it.
- **The state of step 6.** It is `held` until step 4 (BVG-K Stage T, #2836, antigravity-84) saves its models. Its sheet, `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`, has a FIT and a D68 review (antigravity-74, #2839 comment `6398709883`).
- **The gap.** The sheet's run plan uses tools that are not on `main`:
  - the `discrete-stack export-d11` subcommand and `export_d11_to_path` (sheet `:32`, `:81`). Only the library `export_d11` exists (`crates/uor-r4-discrete/src/d11_export.rs:227`), and the "#2998" cited is a different PR;
  - a GPU scorer for the exported artifact. `crates/uor-r4-integer` has no `.cu` or `build.rs`, and `IntegerDiscreteModel::evaluate_prefix` (`discrete_d11.rs:1165`) runs on the host one prefix at a time;
  - a split prefill/decode tok/s harness for `lut-discrete` files.
  So when step 4 lands, step 6 cannot run as written.
- **Why build now.** Building the tools now takes them off the critical path. This item is a separate tool item, so it does not edit step 6's sheet, status or ORDER, all of which are antigravity's.

## The facts this rests on

1. **CHARTER §3.6:** "Panel scoring, qualification and milestone evaluation of an exported artifact remain GPU research compute (§3.5); only product serving, and measuring its tokens/s and energy, run on client devices." So the scorer must run on the GPU.
2. **The comparison must be clean-room.** `IntegerDiscreteModel` (`discrete_d11.rs:1-8`) is a clean-room integer evaluator of the artifact. Scoring the artifact by re-importing it into the training-side CUDA evaluator would compare the training code with itself, so a GPU scorer must port `evaluate_prefix`'s semantics and prove bit-identity with it.
3. **Not in the serving crate.** CUDA launches need `unsafe`, and §3.13 keeps `forbid(unsafe_code)` in the portable runtime (`crates/uor-r4-integer/src/lib.rs:8`). A detached instrument (its own `[workspace]`, path dependencies only) adds the GPU code without touching the serving crate.
   - **Precedents:** `docs/labs/m1-w2-code-utilization/instrument/` and `docs/labs/r-prime-zeta-vs-nulls/instrument/`. Neither builds CUDA, so they set the precedent for detaching only. The nvcc pattern to follow is `crates/uor-r4-discrete/build.rs`.
4. **No lock.** Every ready discrete item, Stage T included, scopes `crates/uor-r4-discrete/**`, and `loop claim` compares by prefix. An item scoped only to `docs/labs/…`, `docs/harness/prep/…` and its task folder takes no crate lock and blocks no lab.
5. **No overlap.**
   - `m4-export-uorlut01-cli` is the M4 product CLI and depends on `m1-close-saved-model`, so it cannot serve step 6.
   - No open PR touches these tools (open: #3245, #3239, #3169).

## Decision (recommended)

**A. Add `m1-d11-artifact-scorer`.** It is `ready`, M1, `needs_gpu: true`, depends on `m1-d11-exporter-discrete` (done), and has the scope above. Its done-when covers the three tools, each with a test that can fail:
- **`export`:** byte-identical to `export_d11`, and accepted by `IntegerDiscreteModel::from_bytes`.
- **`score`:** a CUDA port that is bit-identical to `evaluate_prefix` on fixtures, with a set-aside test, followed by a GPU qualification on one RTX 5090.
- **`toks`:** the step-6 sheet's protocol, on the laptop.

**B. The A8 checkpoint is for qualification only.** The A8 FrozenZero `continuation-05248.json`, inside the store's `a8-full-dose-rerun-20261025.tar.gz`, may be used to qualify the scorer. It is never step 6's reported measurement, which stays on step 4's model under step 6's FIT'd sheet.

**C. Leave the step-6 sheet alone.** This item changes none of step 6's records. A note on #2839 names the commands once they are built. Whether the sheet's tool lines need a correction is then antigravity's call.

**D. The issue.** #3248 was created by hand for this item, and `docs/roadmap/issues.json` maps it. A `roadmap sync` dry run planned to create it and to update 33 other issues; that sweep is the PM's.

## What each outcome unlocks

| Result | Meaning | Decision |
|---|---|---|
| approved | the step-6 tools are a claimable M1 item, with no lock | a lab builds them now; step 6 then runs as soon as step 4 saves its models |
| refused | the tools stay unowned | step 6 waits on step 4 and then on building its own tools |

Nothing here changes a bar, a sequence, a stop rule or the caps. PLAN.md is unchanged, and step 6 keeps its place.

## The fresh decision review

- **Round 1: DECISION APPROVE; exact-head CHANGES at `458a99fe2`.**
  - It verified that the three tools are absent and that #2998 is a different PR.
  - It confirmed that antigravity's REPORTs and #2839 assume the tools exist, and do not plan to build them in step 6's unit, so this item collides with nothing antigravity owns.
  - It confirmed that every ready discrete item, Stage T included, holds `crates/uor-r4-discrete/**`, while this item holds no crate path, and that nothing changes a bar, a sequence, a cap or PLAN.md.
- **The Required fix:** the done-when bullet "the step-6 sheet's tool lines are then true" could never pass, because this item does not edit that sheet. It now asks for a note on #2839 that maps each sheet tool line (`:32`, `:81`, `:82`) to the instrument command that replaces it.
- **Advisories applied:**
  - the precedents build no CUDA, so the nvcc pattern is `crates/uor-r4-discrete/build.rs`;
  - `docs/research/notebook/m4-serving.md` is added to the scope;
  - "off the GPU" became "on the pod's host CPU, not the GPU";
  - the A8 checkpoint is named as inside its store tarball, which was checked in the store;
  - the render was re-run after the `issues.json` line.
