# DECISION: Unblock `m3-qualify-memory` (`#2858`) on the Selected M1 `UORLUT01` Artifacts and Complete M3–M7 Scope Declarations (`CHARTER.md` §5.1, D68)

- **Lab:** `antigravity`
- **Order:** [`docs/loop/tasks/infra-loop-harness/antigravity-114/ORDER.md`](ORDER.md)
- **Roadmap items:**
  - [`docs/roadmap/items/m3-qualify-memory.md`](../../../../roadmap/items/m3-qualify-memory.md) (Issue `#2858`, Milestone M3 `#2031`)
  - [`docs/roadmap/items/m4-export-uorlut01-cli.md`](../../../../roadmap/items/m4-export-uorlut01-cli.md) (Issue `#2859`, Milestone M4 `#2032`)
  - [`docs/roadmap/items/m5-laptop-full-path-measurement.md`](../../../../roadmap/items/m5-laptop-full-path-measurement.md) (Issue `#2862`, Milestone M5 `#2033`)
  - [`docs/roadmap/items/m6-qualify-reasoning-coding.md`](../../../../roadmap/items/m6-qualify-reasoning-coding.md) (Issue `#2868`, Milestone M6 `#2034`)
  - [`docs/roadmap/items/m7-parity-library-cli-browser.md`](../../../../roadmap/items/m7-parity-library-cli-browser.md) (Issue `#2869`, Milestone M7 `#2035`)
  - [`docs/roadmap/items/infra-loop-harness.md`](../../../../roadmap/items/infra-loop-harness.md) (Issue `#2904`)
- **Authority:** `CHARTER.md` §5.1 (`D68`: *"A bar, status, scope fix, classification or stop rule goes ahead when the lab's own decision review passes. Each review reads the whole project before it approves... The owner's queue is a log of the runs and decisions made on the owner's behalf, each with its review record and open to reversal"*).

---

## 1. Whole-Project Context

1. **M1 (`#2029`) is `status: done` on `origin/main` with two selected 19-table `UORLUT01` (`trunk-joint-pointer-v1-d11`, `uor-r4.lut-discrete-trunk/1`) artifacts:**
   - `m1-close-saved-model` (`#2837`, PR `#3312` -> `b6b5a78e1524be0a2da7e4a432a8ae3edf9d030b`) and `m1-fresh-panel-transfer` (`#2841`, PR `#3318` -> `5709a65072650fccacbd807858fb246eeb46f0b6`) closed M1 (`#2029`) on the two-seed `UORLUT01` integer exports (`t16-frozen-zero-seed1-trunk.d11` `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` and `t16-frozen-zero-seed2-trunk.d11` `1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de`).
   - `m4-lawful-likelihood-bpb` (`#2861`, PR `#3324` -> `8f8f06dd5c09fec2790f68725c48e232e811b02a`) is also `status: done` on `origin/main`, pinning those exact two M1 `UORLUT01` exports (`04652a3d…` and `1b49e8bd…`) as the base artifacts for M4 held-out stream scoring.
   - `object-store-consumer-m3` (`#2873`, released in `antigravity-112` `#3322` and executing in PR `#3323`) binds those exact two M1 `UORLUT01` exports on the M3 durable-memory bars (`m3-durable-memory-panel-v1.jsonl`, `582e3879…`).
2. **Why `m3-qualify-memory` (`#2858`) and 6 downstream `M4`–`M8` items (`7` total milestone items) are falsely blocked in `uor-harness loop next`:**
   - The M3 bar in [`docs/integration/project-track.md:116`](../../../integration/project-track.md) reads verbatim: *"On the selected M1/M2 artifact, `m3-memory` passes provenance, contradiction and correction, abstention on unstated or retracted facts, explicit reset and process restart; saved and restored sessions continue identically token-for-token (`IntegerStackSession` + `UR4SESS1`); and non-memory conversational prose does not silently overwrite stored facts (`#1718`)."*
   - Likewise, [`docs/roadmap/items/m3-qualify-memory.md`](../../../../roadmap/items/m3-qualify-memory.md) lines 19 and 36 state: *"waits on an M1 or M2 model"* and *"Name the selected artifact by hash (blocked until M1 or M2 yields one)."*
   - However, `m3-qualify-memory.md`'s frontmatter `depends_on:` listed `[m2-train-grounded-replies, m3-memory-turn-discriminator-gate, m3-policy-precedence-and-sieve-tests]` instead of `[m1-close-saved-model, object-store-consumer-m3, m3-memory-turn-discriminator-gate, m3-policy-precedence-and-sieve-tests]`. Because `m2-train-grounded-replies` (`#2853`) depends on `m2-donor-corpus-generation` (`#2848`, `status: held` under D41's `CHARTER.md` §3.10 / §5.1 owner-dispatch gate), `uor-harness loop next` treated `m3-qualify-memory` (`#2858`) and all 6 downstream milestone items (`m4-export-uorlut01-cli` `#2859`, `m5-laptop-full-path-measurement` `#2862`, `m6-qualify-reasoning-coding` `#2868`, `m7-parity-library-cli-browser` `#2869`, `m8-release-packaging` `#2871`, and `m8-independent-verification` `#2870`) as blocked on dependencies.
3. **Why `m3-qualify-memory.md` (`#2858`), `m4-export-uorlut01-cli.md` (`#2859`), `m5-laptop-full-path-measurement.md` (`#2862`), `m6-qualify-reasoning-coding.md` (`#2868`), and `m7-parity-library-cli-browser.md` (`#2869`) need completed `scope:` and `notebook:` frontmatter:**
   - `m3-qualify-memory.md` lacked `notebook: docs/research/notebook/m3-durable-memory.md`, included `crates/uor-r4-training/**` (the float `stack_grounded_session.rs` module that line 19 explicitly excludes from the integer-path verdict), and omitted `crates/uor-r4-object-store/**`, `docs/mechanisms/exact-memory.md`, `docs/mechanisms/object-store.md`, `docs/harness/prep/m3-qualify-memory*`, `docs/loop/tasks/m3-qualify-memory/**`, and `docs/research/notebook/m3-durable-memory.md`.
   - `m4-export-uorlut01-cli.md` lacked `crates/uor-r4-discrete/**` (needed for GPU `DeviceScorer` evaluation of the discrete trunk on `heldout.u16`), `docs/harness/prep/m4-export-uorlut01-cli*`, `docs/loop/tasks/m4-export-uorlut01-cli/**`, and `docs/research/notebook/m4-serving.md`.
   - `m5-laptop-full-path-measurement.md`, `m6-qualify-reasoning-coding.md`, and `m7-parity-library-cli-browser.md` lacked `notebook:` and `docs/harness/prep/<id>*` / `docs/loop/tasks/<id>/**` entries in `scope:` (and `m6-qualify-reasoning-coding.md` lacked `crates/uor-r4-integer/**` and `crates/uor-r4-discrete/**` for executing the M6 panel on the served model).

---

## 2. Decision

1. **`docs/roadmap/items/m3-qualify-memory.md` (`#2858`):**
   - Change `depends_on:` to `[m1-close-saved-model, object-store-consumer-m3, m3-memory-turn-discriminator-gate, m3-policy-precedence-and-sieve-tests]`;
   - Add `notebook: docs/research/notebook/m3-durable-memory.md`;
   - Update `scope:` to `[crates/uor-r4-integer/**, crates/uor-r4-object-store/**, docs/mechanisms/exact-memory.md, docs/mechanisms/object-store.md, docs/harness/prep/m3-qualify-memory*, docs/labs/m3-qualify-memory/**, docs/loop/tasks/m3-qualify-memory/**, docs/research/notebook/m3-durable-memory.md]`.
2. **`docs/roadmap/items/m4-export-uorlut01-cli.md` (`#2859`):**
   - Update `scope:` to `[crates/uor-r4-integer/**, crates/uor-r4-discrete/**, crates/uor-r4-lut/**, docs/mechanisms/lut-serving.md, docs/model/serving-and-export.md, docs/harness/prep/m4-export-uorlut01-cli*, docs/labs/m4-export-uorlut01-cli/**, docs/loop/tasks/m4-export-uorlut01-cli/**, docs/research/notebook/m4-serving.md]`.
3. **`docs/roadmap/items/{m5-laptop-full-path-measurement,m6-qualify-reasoning-coding,m7-parity-library-cli-browser}.md` (`#2862`, `#2868`, `#2869`):**
   - Add `notebook:` and the missing `docs/harness/prep/<id>*`, `docs/loop/tasks/<id>/**`, and `crates/uor-r4-{integer,discrete}/**` (for M6) entries to `scope:`, and re-render `docs/roadmap/README.md`.

---

## 3. Reversibility and Owner Queue Log

- **Cost to reverse:** One 5-file revert restoring the previous `depends_on:`, `notebook:`, and `scope:` lines in `docs/roadmap/items/{m3-qualify-memory,m4-export-uorlut01-cli,m5-laptop-full-path-measurement,m6-qualify-reasoning-coding,m7-parity-library-cli-browser}.md` and re-rendering `docs/roadmap/README.md`.
- **CHARTER §3 / GPU cap check:** Zero changes to `CHARTER.md`, `§3` technical constraints, or GPU caps (`$0` GPU spend).
