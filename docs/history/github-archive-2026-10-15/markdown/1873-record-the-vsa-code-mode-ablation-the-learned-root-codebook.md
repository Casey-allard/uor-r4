# #1873 — Record the VSA code-mode ablation: the learned-root codebook does not move this scorer (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T03:34:39Z
- **Updated:** 2026-10-09T13:23:00Z
- **Closed:** 2026-10-08T03:36:18Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1873
- **Merged:** 2026-10-08T03:36:18Z
- **Merge commit:** c10f625a1f0fbe80f8c5d416f52f533a9f3c4d7a
- **Base / Head:** main / UOR-Foundation:deepseek/vsa-codemode-evidence
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this records

Card P7 measured the VSA term as HARMFUL under the fixed token-id-hash codebook. `learner/vsa_codes.rs` documents that defect and its repair — codes derived from the **learned 120-root assignment** by LSH of the canonical icosian root quaternions, so `E[d_H] = D·θ/π` — and the September wiring record measured the repair beating the hash on `shortlist-recall` (recall 8.4–8.8 % → 10.7–11.2 %). The open question was whether the repaired codebook changes the **full-vocabulary discrete scorer's** verdict. This re-runs the sweep under both codebooks.

## Result

Artifact `native_geometric_prose_model.rgm`, sha256 `7023c450…` (the Card P7 identity), two disjoint held-out slices of 8,127 scored positions:

| slice | mode | baseline BPB | vsa delta | 95 % CI | band | dTop1 |
|---|---|---:|---:|---|---|---:|
| offset 0 | fixed | 1.8964 | −0.0010 | [−0.0011, −0.0009] | NEGLIGIBLE | −0.04 |
| offset 0 | root | 1.8969 | −0.0015 | [−0.0019, −0.0010] | NEGLIGIBLE | +0.00 |
| offset 100M | fixed | 1.8783 | −0.0015 | [−0.0016, −0.0013] | NEGLIGIBLE | −0.05 |
| offset 100M | root | 1.8788 | −0.0020 | [−0.0025, −0.0016] | NEGLIGIBLE | −0.12 |

**Switching to the learned-root codebook does not make the VSA term contribute to this scorer** — the band is NEGLIGIBLE in all four runs and the delta is slightly negative in every one, replicated on two slices. The mode switch is real and confirmed by the header (`root (120 distinct code vectors)` versus `fixed (0 distinct code vectors)`).

That is precisely the ceiling `vsa_codes.rs` states: codes are a function of the root, so 4,096 tokens collapse onto at most 120 distinct codes and two tokens in one root have `d_H = 0`. Its own remedy is Stage 3 — codes at finer resolution from the **continuous** learned embedding, which the exported artifact does not carry (it stores `token_to_root` only).

## Scope, stated to prevent misuse

- This measures the VSA term's contribution to **this artifact's** discrete scorer. It is not a verdict on the 120-root representation, the JEPA learner, or routing — the September record shows the repair improving shortlist routing. The repair is real; its effect does not reach this term.
- `ablate-prose` prints a **static** wiring note for the `vsa` row ("heads bind a fixed random codebook…"). It is not mode-aware and is therefore stale for the root runs; the header line is the mode evidence.
- Debug build, one artifact, one corpus, two 8,256-token slices, temperature 1.0. No training, no fitting, no serving change.
- Absolute BPB differs from the Card P7 record (vsa row 1.8048) because that record evaluated the stripped `_nocoarse` artifact (963,950 bytes).

The document is `docs/evidence/native_geometric_vsa_code_mode_ablation_2026-10-08.txt`; the raw tool output for all four runs is retained at `~/uor-r4-local/vsa-codemode/` on the laptop.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

