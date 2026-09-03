# Independent pre-execution review — #1096

**2026-09-03 — `APPROVED_FOR_SINGLE_READINESS_ATTEMPT`; execution is `NOT_RUN`.**
The reviewer was the independent `/root/readiness_review` agent. No sandbox,
readiness worker, model, research corpus, fit, comparison or replay was executed
by this review. This is static admission of one empirical readiness attempt,
not an observed runtime result or mathematical proof.

## Exact admitted freeze

The approved manifest is
`/Users/casey.allard/.codex/uor/issue-1096-readiness-freeze02/manifest.json`,
9,395 bytes, SHA256
`4acd2b7ec00ac8874573e2d6e52e5087b376bc0ebcd52aedd4464aa28979c644`.
Its source commit is `79c674c8f6179a68878a12ee86e664f1435c3ebf`.
The independent `review.json` in that same directory has SHA256
`e092e4bc3d3a989d37868c5530ff3483d80f4a64e53b3cf609aa1774a5c43759` and binds only those exact manifest bytes.

| Bound record | SHA256 |
|---|---|
| Compact artifact/source binding | `23977918c1685148f6cbc61ebab87507e2ce153e40d9b15c2634a4d68e0c2353` |
| Actual sandbox profile | `d508d3ec50b78e3bbd3cb1fe9468788f457f4e47f713f9a793f03079df48416e` |
| Original #1094 stop | `87bd3082ce9b4da5e5227a3b82f6515773cf5f113de689ff6251a2a45340fad5` |
| Original #1094 binding | `baeb56a632c0e75c0574d9bad503e7849cf434039490e1fe92e3b8f0114b1f0b` |
| Original #1094 profile | `914b72856e0822c981b1295b8e96048fa43f35b8c1dedeea58f51321a781a72d` |

## Independent checks and resolved findings

The reviewer inspected the coordinator, actual worker, profile generator,
[contract](r4_isolated_runtime_readiness_1096.md),
[source audit](r4_isolated_runtime_readiness_1096_sources.md) and original
[#1094 terminal](r4_text_clause_adapter_1094.md). An independent standard-library
script verified all 169 source-closure files against both their recorded SHA256
values and exact Git blobs at the admitted commit. It rehashed 18 runtime files,
verified both interpreter symlinks and all four harmless sentinels, and
independently reconstructed the exact profile bytes without importing the
candidate package. The three changed Python sources parse successfully.

The five artifact identity records equal the original #1094 binding. The
reviewer did not open model or corpus assets; their byte verification belongs
to the parent and actual worker's declared checks. This distinguishes record
comparison from independent asset-byte verification.

The corrected profile retains the original source, venv and resolved runtime
trees and the five exact asset paths. It adds the literal interpreter aliases,
manifest/profile reads and exact ancestor metadata. The data and xattr rules
retain their restricted exceptions; ancestor metadata does not admit directory
contents. Network and home writes remain denied. Old editable source and
research corpus/reference/history paths receive no new content permission.
The permission effect still requires the declared runtime observation.

Three static review findings were resolved before any attempt: the 60-second
deadline now covers admitted identity checks and launch as well as the child;
captured output is retained after execution ends without interruption from
that deadline timer; and asset names are sorted before rendering the profile,
so canonical binding serialization preserves exact profile bytes. The first
identity-only freeze, SHA256
`704b78e285a1ba5b1ee24f5c9e3a82205ac15c3d31081fc942c1a0dcbcb140ea`,
remains intact in the original `issue-1096-readiness` directory. It was
superseded by the ordering correction and has no `started.json`. It is an
unattempted authoring artifact, not a second empirical attempt. The two freeze
directories total 153,632 bytes before this review receipt, leaving the declared
8 MiB evidence reserve inside the aggregate 16 MiB task cap.

## Admission and limits

The reviewed command uses the actual worker's `--readiness-only` branch with
closed stdin. Static control flow skips model deserialization/construction,
model-definition imports and the entire batch loop; no corpus preparation or
population generation is reachable from the coordinator. Byte hashing of the
accepted artifacts is permitted and does not construct an executable model.

Approval admits exactly one attempt with the frozen 60-second, 16 MiB and
3 GiB combined peak-RSS limits, zero model loads/forwards/optimizer updates,
and an exclusive irreversible `started.json` lock. The parent must publish the
exact run contract before launch. Success requires exit zero, exactly the
bound `ready`/`done` events, verified fixed CPU runtime, all four permission
refusals, null model states and the declared zero-work audit. Failure retains
raw partial output and the specific terminal; it admits no retry or revised
profile under this manifest.

No actionable static blocker remains for that named decision. The audit does
not prove Python startup, establish the original denial's cause, guarantee
general sandbox security, qualify raw-text model behavior or release withheld
inputs. #1094 remains open; #1079's weak-control and #1082's descriptive results,
#973 open and #954 blocked remain unchanged.
