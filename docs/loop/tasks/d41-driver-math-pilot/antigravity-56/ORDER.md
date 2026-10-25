# ORDER: d41-driver-math-pilot, antigravity-56

Lab: antigravity. Item: [d41-driver-math-pilot](../../../../roadmap/items/d41-driver-math-pilot.md). Base: `89d8cea20`.

## Goal

The item's done-when reads:

> The driver math optimization lands with parity controls, and the ~1M-token pilot is run and recorded.

## Cancel probe (run first)

- Command:
  - `shasum -a 256 docs/labs/bvgk-rowsk-scoring-2026-10-18/evidence/run1/clean/rowsk-qualification.json docs/labs/injective-preflight-and-c2-qualify-2026-10-19/evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json docs/labs/address-cells-preflight-2026-10-19/evidence/grading.json archive/integration/DECISIONS-2026-10-19.md`
  - `~/.cargo/bin/cargo test -p uor-r4-discrete --lib rowsk -- --nocapture`
  - `~/.cargo/bin/cargo test -p uor-r4-discrete --lib bvgk -- --nocapture`
- Cancel if: any of the qualified driver-math parity receipts (`rowsk-qualification.json`, `bvgk-qualification.json`, `address-cells-preflight` `grading.json`) is missing or fails verification, or `rowsk`/`bvgk` unit tests fail on host.
- Probe result (2026-10-25, classed `instrument`): PASS. All three qualified driver-math receipts verify (`rowsk-qualification.json` `d5ec5348d64e2abd74a334319d4f78e6ed7b174b5a2fc0bb381505dc46e09e56`, `bvgk-qualification.json` `2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`, `address-cells-preflight` `grading.json` `90c113adbcc2de7c2321a6c6fca9b89a7e72478f8123ae6e3a67dd6d3ee61bb3`); `rowsk` (3/3) and `bvgk` (13/13) unit tests pass on host. Inspection of `CHARTER.md` §3.10, §5.1, `PLAN.md` lines 75–77, and `archive/integration/DECISIONS-2026-10-19.md` (D41 items 1–5) confirms that while the driver-math optimization and parity controls (row-parallel counterfactual `684/684`, address-cell occupancy `256/256` and `B3_init = 1,351 / 3,847`, BVG-K C1 `5,130/5,130` at `16.25x`, and BVG-K C2 `5/5` at `8.0x`) have landed and passed CUDA qualification, the second half of `d41-driver-math-pilot`—the ~1M-token pilot on `openai/gpt-oss-120b` (`distill-d1` pilot)—is part of D41's one-time neural donor pass, which `CHARTER.md` §3.10 and §5.1 explicitly require an owner dispatch release for after its pre-registration sheet, non-author `TEST FITNESS: FIT` review, and M1 Stage T (`m1-bvg-k-stage-t-training` #2836) complete.

## Files in scope

- `docs/harness/prep/d41-driver-math-pilot*`
- `docs/labs/d41-driver-math-pilot/**`
- `docs/loop/tasks/d41-driver-math-pilot/**`

## Done when

- The landed driver-math optimization and CUDA parity-control receipts (`counterfactual-rows-parity`, `address-cells-preflight`, `bvgk-rowsk-scoring` C1, and `bvgk-c2-qualify` C2) are reconciled and pinned in `docs/labs/d41-driver-math-pilot/README.md` and `docs/labs/d41-driver-math-pilot/driver-math-parity-summary.json`; the one-page pre-registration sheet `docs/harness/prep/d41-driver-math-pilot-2026-10-25.md` for the ~1M-token `openai/gpt-oss-120b` donor pilot (`distill-d1-pilot-1m`) is authored and receives a non-author fresh-context `TEST FITNESS: FIT` review posted on `#2821`; and under a fresh-context D68 decision review (`CHARTER.md` §3.10, §5.1, `PLAN.md` M1→M2 route, D41 items 3 and 5), `docs/roadmap/items/d41-driver-math-pilot.md` is set to `status: held` pending M1 Stage T (`m1-bvg-k-stage-t-training`) completion and the owner's §3.10/§5.1 dispatch release for the `openai/gpt-oss-120b` donor pilot.

## Stop and ask if

- An owner-only boundary (`CHARTER.md`, §3 technical constraints, GPU caps, or `docs/harness/owner-paths.json`) would need to change, or before dispatching any `openai/gpt-oss-120b` donor pass (`CHARTER.md` §3.10, §5.1).
