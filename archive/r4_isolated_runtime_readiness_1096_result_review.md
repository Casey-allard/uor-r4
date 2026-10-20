# Independent readiness result review — #1096

**2026-09-03 — `INDEPENDENT_RESULT_REVIEW_PASS`: the sole attempt satisfies
`ISOLATED_RUNTIME_READY`.** The independent `/root/readiness_review` agent
reviewed the frozen source and raw evidence after the parent's one execution.
The reviewer executed no readiness, model, comparison or replay and opened no
research corpus or model assets.

## Evidence and source binding

The admitted source is `79c674c8f6179a68878a12ee86e664f1435c3ebf`; current package,
policy and lockfile bytes remain unchanged from that revision. All 169 source
records were rehashed, along with the 18 bound runtime files, profile, compact
binding, four harmless probes and the original #1094 stop. The interpreter
symlinks still have their bound targets. The five artifact identity records
remain equal to the retained original binding. The reviewer compared those
records without reopening model assets; the accepted worker path performed its
required source/asset-byte checks before emitting readiness.

The raw stdout contains exactly two JSON objects, `ready` followed by `done`,
and they equal the result receipt's embedded events. Stderr is empty. The
coordinator's reviewed success path requires child exit zero before admitting
this terminal. The start receipt binds the exact pre-execution manifest and
independent approval; the resource receipt binds the exact result bytes.

| Retained original | Bytes | SHA256 |
|---|---:|---|
| `manifest.json` | 9,395 | `4acd2b7ec00ac8874573e2d6e52e5087b376bc0ebcd52aedd4464aa28979c644` |
| `review.json` | 2,452 | `e092e4bc3d3a989d37868c5530ff3483d80f4a64e53b3cf609aa1774a5c43759` |
| `started.json` | 737 | `2da6bab8bc6812b4dcec55da98650fe996768c9ff365fbcdde5896f31c9d38b9` |
| `worker.stdout.jsonl` | 3,769 | `cbe9436ab15bf207c362b7863685c524ec95f8f20b2402f3294920289af6561c` |
| `worker.stderr.txt` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `result.json` | 4,756 | `439aa149d6f128844490c4a9002bfe2ffb52fdeeaad067e8e3cb16447b24b930` |
| `resources.json` | 394 | `c3a5005f661cdd2e42a18e85eea066d0a7c9dadf3f7b6f391c48e8757398ba42` |

These originals reside at
`/Users/casey.allard/.codex/uor/issue-1096-readiness-freeze02/`.
The profile remains SHA256
`d508d3ec50b78e3bbd3cb1fe9468788f457f4e47f713f9a793f03079df48416e`;
the compact binding remains SHA256
`23977918c1685148f6cbc61ebab87507e2ce153e40d9b15c2634a4d68e0c2353`.

## Observed criterion and resources

**Measured behavior.** The actual isolated worker started, verified the bound
interpreter/source/artifact/runtime identities, configured the fixed CPU plan,
and completed with the expected readiness-only events. Both events agree on
Python 3.12.14, Torch 2.7.1, CPU/Accelerate, four intra-op threads, one inter-op
thread, deterministic algorithms, one worker, Apple M1/MacBookPro17,1, eight
logical CPUs and 16 GiB memory. Both bind the same manifest/profile and record
all 18 runtime files verified.

All four harmless corpus/reference/history/results probes report permission
denial. The reviewed probe implementation accepts only `PermissionError`;
successful open, missing file or a different I/O failure cannot satisfy it.
The parent verified their existence and exact harmless bytes before launch;
this review verified the same bytes remain afterward.

Both events report zero model loads, zero row forwards and zero batch forwards.
The initial model state and final before/after states are null. The final audit
records zero optimizer updates and oracle/label file reads, with isolation
confirmed. Static control flow skips model construction/deserialization and the
input batch loop. Withheld comparison and model replay remain `NOT_RUN`.

Elapsed coordinator time was **2.058100333 seconds**, inside the 60-second cap.
The conservative combined parent/child peak-RSS bound was **704,806,912 bytes**,
inside 3 GiB; the worker's own peak was 426,901,504 bytes. The second freeze and
its completed receipts contain 88,996 bytes across 13 files. The superseded
unattempted freeze contains 76,744 bytes. Their aggregate **165,740 bytes** is
inside 16 MiB. The resource receipt's 88,602 pre-receipt bytes plus its own
394 bytes reconciles exactly. Repository evidence copies and documentation are
additional retained delivery material, to be counted in the parent's final
storage review.

Exactly one `started.json` exists across the two declared freeze directories,
in the second freeze. The first manifest remains SHA256
`704b78e285a1ba5b1ee24f5c9e3a82205ac15c3d31081fc942c1a0dcbcb140ea`,
with neither start nor result receipt. Its static asset-ordering correction was
made before this sole attempt. The original #1094 stop still hashes to
`87bd3082ce9b4da5e5227a3b82f6515773cf5f113de689ff6251a2a45340fad5`.
No evidence was overwritten or deleted by this review.

## Verdict, limits and handoff

No actionable finding remains in the observed result or its source/evidence
binding for the named readiness decision. #1096 is eligible for closure after
protected delivery of its source, contract, reviews and exact receipts. Its
completion returns a runtime-readiness handoff to still-open #1094; it does
not execute or release that comparison.

The successful combined correction does not establish which missing alias or
ancestor-metadata operation caused the original `execvp` denial. This is one
measured startup/isolation check on the bound machine and files, not a general
sandbox-security guarantee, a mathematical proof, parser correctness, raw-text
model preservation or general language capability. The NEMESIS/W33/UOR source
context supplies none of those claims. Supplied segmentation remains the
qualified model entry; #1079's `LANGUAGE_R4_PRESERVED_CONTROL_WEAK`, #1082's
descriptive findings, #973 open and #954 blocked remain unchanged.

The concrete next action is a separately frozen #1094 preparation decision
using the qualified readiness handoff and independent release review. The
original #1094 unavailable result and withheld population remain preserved;
this review authorizes no new fit, withheld access, evaluation or replay.
