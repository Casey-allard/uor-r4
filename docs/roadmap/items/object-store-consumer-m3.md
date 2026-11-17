---
id: object-store-consumer-m3
title: Wire the object store into the model and run the durable-memory bars
milestone: M3
area: architecture
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
notebook: docs/research/notebook/m3-durable-memory.md
scope: [crates/uor-r4-object-store/**, docs/mechanisms/exact-memory.md, docs/mechanisms/object-store.md, docs/labs/object-store-consumer-m3/**, docs/harness/prep/object-store-consumer-m3*, docs/loop/tasks/object-store-consumer-m3/**, docs/research/notebook/m3-durable-memory.md]
---

## Why it matters

The object store is the project's exact, hashed durable storage, and M3 (durable conversation memory) is the milestone that needs it. The consumer now reads and writes it through finite host controls, but the on-model memory bars have not run. A consumer also gives later replies exact bytes to quote from. See [the transformer-to-geometry map](../../start/transformer-vs-geometry.md) and [the object store page](../../mechanisms/object-store.md).

## What is missing

**November 17, 2026.** The store consumer and registered thresholds have host evidence. M3 still needs the selected model, fixed panel and scoring controls for provenance, contradiction/correction, abstention, reset and restart. D67 governs dispatch after a non-author FIT and the lab's pre-run review. [deepseek-63 report](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/loop/tasks/object-store-consumer-m3/deepseek-63/REPORT.md) · [Issue #2858](https://github.com/UOR-Foundation/uor-r4/issues/2858)

The object store (D19) keeps bytes exactly in a hashed tree and has a client conformance pass. The durable-memory consumer and its frozen 384-item host-control panel are delivered (`4797ca883`, `#2960`), including `export_scope`/`import_scope`, and a cross-crate interop contract pins the integer session bytes against the store (`#3106`). The on-model durable-memory qualification remains **NOT_RUN**: it needs an M1-or-M2 model selection, a non-author FIT and the lab's D67 pre-run review. M1 is complete and M2 is in progress; M1's completion alone is not M3 qualification. The six per-sub-panel thresholds are registered in the [consumer record](../../labs/object-store-consumer-m3/README.md).

## Done when

A consumer reads and writes through the object store on the model path, and the durable-memory bars run with recorded results. The M3 bar in [`docs/integration/project-track.md`](../../integration/project-track.md) is: on the selected M1/M2 artifact, `m3-memory` passes provenance, contradiction and correction, abstention on unstated or retracted facts, explicit reset and process restart. Saved and restored sessions must continue identically token-for-token (`IntegerStackSession` + `UR4SESS1`), and non-memory prose must not silently overwrite stored facts (#1718).

**Current status:** the consumer and host controls pass; the selected-model run has not happened. The six thresholds are registered in the consumer record. **Original item-start status, retained only as history:** the thresholds were unregistered, M1 was unmet and M2 was held.

## How to start

The consumer design, implementation and finite controls in steps 2–4 are complete. Current work is the selected-model qualification in steps 5–6, not another consumer build.

1. Read [`docs/mechanisms/object-store.md`](../../mechanisms/object-store.md) (especially "What has never been tested" and "What would make it ACTIVE"), [`docs/mechanisms/exact-memory.md`](../../mechanisms/exact-memory.md), the [D19 record](../../../archive/labs/d19-object-store-2026-10-15/README.md), the [conformance record](../../labs/uorobj01-conformance-2026-10-18/README.md), the [consumer record](../../labs/object-store-consumer-m3/README.md) and [`docs/research/notebook/m3-durable-memory.md`](../../research/notebook/m3-durable-memory.md).
2. **Completed:** the design note chose the consumer and kept exact token, occurrence and version identity separate from geometric addresses, with no softmax, matmul or float on this path.
3. **Completed:** finite CPU controls cover byte-identical write/read and restart/restore, refusal of retracted or unstated facts, explicit reset and corrupt-record refusal. The consumer record holds the current test counts and frozen 384-item host-control result.
4. **Completed:** the consumer uses `UR4SESS1`, `export_scope`/`import_scope` and occurrence bindings. The saved-session format lives in `crates/uor-r4-integer/src/session_codec.rs`; the interop contract pins its bytes against the object store.
5. **Current next step:** select an M1 or M2 artifact by hash and confirm the registered pre-registration: frozen panel, the six existing per-sub-panel thresholds, stop rules and cost. The on-model run needs a non-author FIT review and the lab's D67 pre-run review, not a blanket owner release.
6. Open a PR that says `References #<this issue>` for partial work and `Closes` only after the on-model bars have run with recorded results. Record the result as one row in [`docs/research/notebook/m3-durable-memory.md`](../../research/notebook/m3-durable-memory.md).

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:22`](../../../docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md)
- [`docs/mechanisms/README.md:104`](../../../docs/mechanisms/README.md)
- [`docs/start/transformer-vs-geometry.md:34`](../../../docs/start/transformer-vs-geometry.md)
- [`docs/mechanisms/object-store.md`](../../mechanisms/object-store.md)
- [`docs/integration/project-track.md`](../../integration/project-track.md)
