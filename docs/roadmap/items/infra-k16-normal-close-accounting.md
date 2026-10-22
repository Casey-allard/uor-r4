---
id: infra-k16-normal-close-accounting
title: Record K16 normal close and final resource accounting
milestone: infra
area: tooling
status: done
open_to: lab
needs_gpu: false
depends_on: []
scope: [docs/harness/ledger/**, docs/labs/infra-k16-normal-close-accounting/**, docs/reference/storage.md]
---

## Why it matters

Closes custody and cost records for the last full experiment so later runs build on a clean ledger. It is a records task, not a model change. See [where we are](../../start/where-we-are.md) for the K16 result it belongs to.

## What is missing

NOW's historical K16 snapshot (`docs/integration/NOW.md:69-81`) says the same lease is retained and "Normal close and final resource accounting are not yet recorded". The registered outcome call itself is done (`OCCUPANCY_COLLAPSE_TRIGGERED`, 2026-10-19), so check whether closure is already recorded before starting; this line may be stale.

What the sources show today:

- NOW labels that section "Historical K16 redirect — 18 October 2026" and says its retained-lease statements describe that snapshot.
- The four-role record (`docs/labs/k16-four-role-execution-2026-10-18/README.md`, line 24) says: "Normal close: NOT_PERFORMED at the 18 October 23:54 UTC observation cutoff. Final resource accounting: PENDING actual cutoff." Its $4.308712222 proxy covers 15:01:18–19:46:01Z only, at the quoted $0.908 per hour. Storage and transfer are unmeasured, and no final total, deletion or release receipt is recorded.
- The outcome-call record (`docs/labs/k16-outcome-call-2026-10-19/README.md`) contains no close or accounting entry.
- Custody is done: NOW records that all four native jobs exited 0 and the archived shards passed cloud readback.

## Done when

Normal close and final resource accounting recorded for the K16 run, or the stale statement in NOW removed with a pointer to where it was recorded.

## How to start

1. Check whether closure is already recorded. A search of `docs` for "normal close" and "final resource accounting" finds only NOW and the four-role record. The lease or pod id is not in `docs/labs/k16-four-role-execution-2026-10-18/receipt.json`; look in the #2029 comments, in `docs/harness/ledger/*.jsonl`, and in the local compute ledger that `uor-harness budget show` reads (`docs/harness/README.md:81`). Then search for it: `git grep -n "<id>" -- docs archive/CATALOG.md`. Also read `docs/labs/k16-current-status-scope-2026-10-19/README.md` and `docs/reference/storage.md`.
2. From the repository root, read the live state without changing it: `scripts/pod/uor-pod status` and `uor-harness budget show`. If the lease is still retained, do not destroy anything. Post on the issue and ask the lease holder. Only the session that holds a lease closes it, and results must already be off the pod; NOW records that custody passed.
3. First deliverable, posted on the issue: a short comment with the lease id, create time, close time (or "still retained"), billed hours times the quoted $0.908 per hour as a labelled proxy, the storage and transfer figures if measured (otherwise "unmeasured"), and a link for each fact.
4. Record it. If the close happened, add a dated addendum under line 24 of the four-role record; do not rewrite its earlier wording, since nothing is deleted. If it was already recorded elsewhere, edit `docs/integration/NOW.md:69-81` to drop "not yet recorded" and point to that record. If the close is genuinely still pending, record that and leave NOW as it is.
5. Run `uor-harness check` and `~/.cargo/bin/cargo run -q -p xtask -- roadmap check`.
6. Open a PR that says `Closes #<this issue>`, or `References` if partial. Record the result as one row in `docs/research/notebook/m1-discrete-attention.md`, the line that holds the K16 rows.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/integration/NOW.md:69-81`](../../../docs/integration/NOW.md)
