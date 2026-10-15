# #1420 — feat(training): recover retained dialogue artifact output

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T06:10:15Z
- **Updated:** 2026-10-09T13:17:14Z
- **Closed:** 2026-09-27T06:12:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1420
- **Merged:** 2026-09-27T06:12:15Z
- **Merge commit:** 662147435a768e9e5a5fe787887942653f11d728
- **Base / Head:** main / UOR-Foundation:codex/dialogue-artifact-replay-20260927
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained R1d dialogue study saved learned width-576 parameters and likelihood curves but had no usable reload-to-generation path. This adds a training-only importer for its sealed continuous Full/Dot/Quaternion artifact and an exact-token entry sharing the existing generation loop. The ordinary checkpoint and integer-serving width contracts are unchanged.

The new offline example replays 38 existing development requests (58 actual assistant responses), using shared segmented role encoding and exact generated intermediate history. It preserves raw IDs/bytes, every stop and prediction record, and original-versus-replay provenance. The published result is weak dialogue: conversational phrasing and some topic reuse, but repetition, missing answers, 11 model EOS / 4 cycle / 43 cap stops. No fresh-panel qualification, further training or serving promotion follows. Documentation also corrects the old report's duplicated supervision count and separates a bounded no-longer-fit decision from an unsupported capacity-floor conclusion.

Validation at `e3fc0a587115d34b41a0fc9edacc09745ccc0746`:

- Four focused optimized Rust checks pass: complete named F32 mapping, malformed/nonfinite rejection, strict configuration/capacity boundaries, and old text-versus-exact-ID generation decisions. The initial 3/4 attempt failed before model construction because its synthetic tokenizer lacked a ByteLevel declaration; one fixture line fixed it. Both attempts are retained.
- Actual retained artifact loaded and completed all 58 responses. Full-forward versus incremental probabilities on 15 prefix positions differ by at most 9.69e-7, with zero argmax disagreements. No historical backend bitwise claim.
- Total build/check work 1067.794 s; actual offline replay 18.132 s, 58,572,800 B peak child RSS. Full cumulative accounting, input/source/executable hashes and all outputs are in `docs/evidence/dialogue-r1d-replay-2026-09-27.json`; the sealed full prediction packet remains at its recorded local path.
- Independent source and all-output review found no remaining blocker. Touched Rust formatting, diff whitespace and claim-wording checks pass. Queue compatibility acknowledgements execute no tests.

The active frozen Dot/Lorentz/Affine study and other labs' source/artifacts remain unchanged. References #973. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

