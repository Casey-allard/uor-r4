# DECISION: Narrow `m4-lawful-likelihood-bpb` (`#2861`) to its docs/registration scope and unhold `object-store-consumer-m3` (`#2873`) post-M1 (`CHARTER.md` §5.1, D68)

- **Lab:** `antigravity`
- **Order:** [`docs/loop/tasks/infra-loop-harness/antigravity-112/ORDER.md`](ORDER.md)
- **Roadmap items:**
  - [`docs/roadmap/items/m4-lawful-likelihood-bpb.md`](../../../../roadmap/items/m4-lawful-likelihood-bpb.md) (Issue `#2861`, Milestone M4 `#2032`)
  - [`docs/roadmap/items/object-store-consumer-m3.md`](../../../../roadmap/items/object-store-consumer-m3.md) (Issue `#2873`, Milestone M3 `#2031`)
  - [`docs/roadmap/items/infra-loop-harness.md`](../../../../roadmap/items/infra-loop-harness.md) (Issue `#2904`)
- **Authority:** `CHARTER.md` §5.1 (`D68`: *"A bar, status, scope fix, classification or stop rule goes ahead when the lab's own decision review passes. Each review reads the whole project before it approves... The owner's queue is a log of the runs and decisions made on the owner's behalf, each with its review record and open to reversal"*).

---

## 1. Whole-Project Context

1. **M1 (`#2029`) and all 8 steps of the M1 ranked path (`PLAN.md` Steps 1–8) are `status: done` on `origin/main`:**
   - `m1-close-saved-model` (`#2837`, PR `#3312` -> `b6b5a78e1524be0a2da7e4a432a8ae3edf9d030b`) and `m1-fresh-panel-transfer` (`#2841`, PR `#3318` -> `5709a65072650fccacbd807858fb246eeb46f0b6`) closed M1 (`#2029`) with the two-seed 19-table `UORLUT01` (`trunk-joint-pointer-v1-d11`, `uor-r4.lut-stack/3`) integer artifacts (`s1-trunk-joint-pointer-v1-d11.lut` SHA-256 `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`, `7,386 / 14,287` on `ft/dev` and `7,703 / 14,479` on `Fresh V2`; `s2-trunk-joint-pointer-v1-d11.lut` SHA-256 `1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de`, `7,374 / 14,287` on `ft/dev` and `7,722 / 14,479` on `Fresh V2`), archived at `caseyallard/uor-r4-store:results/antigravity/m1-close-saved-model-2026-10-26/outputs/m1-close-saved-model-20261026.tar` (`a758ca0985c6b3b349c2466ab22f6af358f9b76a674011ff6b44ec51c747e998`).
2. **Why `m4-lawful-likelihood-bpb` (`#2861`, M4) was falsely blocked on `loop next` / `loop claim`:**
   - `m4-lawful-likelihood-bpb` is the prerequisite for `m4-export-uorlut01-cli` (`#2859`) and 5 downstream `M5`–`M8` items (`6` total downstream milestone items).
   - Its Rust implementation (`crates/uor-r4-integer/src/lawful_likelihood.rs`) and 9 finite host control tests were already delivered and merged in PR `#2945` (`deepseek-17`), and its design note (`#2940`) and owner answers (`#2028#issuecomment-6339008955` and `#2028#issuecomment-6379430093` / `#2861#issuecomment-6379432987`) are recorded.
   - As noted in `#2861#issuecomment-6374580394`, only the docs-only registration update to `docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md` (pinning the exported `UORLUT01` base artifact SHA-256s + seeds and recording Casey's `6379430093` answer, followed by a fresh-context `TEST FITNESS: FIT` review) and its companion documentation/notebook rows remain to satisfy `#2861`'s `Done when`.
   - Retaining `crates/uor-r4-discrete/**`, `crates/uor-r4-integer/**`, and `docs/mechanisms/lut-serving.md` in `m4-lawful-likelihood-bpb.md`'s `scope:` caused `uor-harness loop next` to print `WAIT` whenever `d11-exporter-float-trunk` (`PR #3320`) or `infra-loop-harness` (`PR #3319`) was open.
3. **Why `object-store-consumer-m3` (`#2873`, M3) is ready to unhold (`status: held -> ready`):**
   - `object-store-consumer-m3` was held (`antigravity-57`, `#2873#issuecomment-6394411611`) solely because M1 was not yet met and no candidate M1/M2 `UORLUT01` integer stack artifact was selected (`docs/roadmap/items/object-store-consumer-m3.md:23`, `docs/labs/object-store-consumer-m3/README.md:75`).
   - Its durable consumer (`crates/uor-r4-object-store/src/durable_consumer.rs`), `UR4SESS1` session codec interop (`crates/uor-r4-integer/src/session_codec.rs`, `#2960` and `#3106`), frozen 384-item panel (`docs/labs/object-store-consumer-m3/m3-durable-memory-panel-v1.jsonl`), owner-registered M3 acceptance thresholds (`#2028#issuecomment-6356034555`), and non-author `TEST FITNESS: FIT` (`#2873#issuecomment-6394411357`) are all on `origin/main`.
   - Now that M1 (`#2029`) is `CLOSED_PASS` with the selected two-seed 19-table `UORLUT01` (`trunk-joint-pointer-v1-d11`, `uor-r4.lut-stack/3`) integer stack artifacts (`04652a3d…` and `1b49e8bd…`), `object-store-consumer-m3` can be unheld (`status: ready`) and its `scope:` narrowed to `crates/uor-r4-object-store/**` plus its docs/lab/prep/task/notebook paths (dropping `crates/uor-r4-core/**` and `crates/uor-r4-integer/**` so it does not collide with `d11-exporter-float-trunk`).

---

## 2. Decision

1. **Narrow `docs/roadmap/items/m4-lawful-likelihood-bpb.md` (`#2861`) `scope:`** to `[docs/model/serving-and-export.md, docs/research/findings.md, docs/research/open-questions.md, docs/labs/m4-lawful-likelihood-bpb/**, docs/harness/prep/m4-lawful-likelihood-bpb*, docs/loop/tasks/m4-lawful-likelihood-bpb/**, docs/research/notebook/m4-serving.md]`.
2. **Release `docs/roadmap/items/object-store-consumer-m3.md` (`#2873`) from `status: held` to `status: ready`**, add `notebook: docs/research/notebook/m3-durable-memory.md`, narrow its `scope:` to `[crates/uor-r4-object-store/**, docs/mechanisms/exact-memory.md, docs/mechanisms/object-store.md, docs/labs/object-store-consumer-m3/**, docs/harness/prep/object-store-consumer-m3*, docs/loop/tasks/object-store-consumer-m3/**, docs/research/notebook/m3-durable-memory.md]`, and re-render `docs/roadmap/README.md`.

---

## 3. Reversibility and Owner Queue Log

- **Cost to reverse:** One 2-file revert restoring the previous `scope:` and `status:` lines in `docs/roadmap/items/{m4-lawful-likelihood-bpb,object-store-consumer-m3}.md` and re-rendering `docs/roadmap/README.md`.
- **CHARTER §3 / GPU cap check:** Zero changes to `CHARTER.md`, `§3` technical constraints, or GPU caps (`$0` GPU spend).
