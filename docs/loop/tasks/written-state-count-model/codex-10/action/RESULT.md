# [codex] A learned gate can veto unsupported sources, but also suppresses grounded replies

**Decision: keep this gate diagnostic.** It withheld every unsupported request on the separately constructed mixed family, yet lost 24 of 48 already exact positive replies. Each fell from exact through EOS to a wrong first token at position zero. It also lost 612 of 1,406 previously exact replies on the earlier unseen-family panel. The preregistered maximum loss was five percentage points on these positives. Both checkpoints reproduced the counts on the same prompts. [Decision rule](SHEET.md) · [Mixed-family action words](gate/HELD-ACTIONS.json)

| Panel / class | Rows | Role-only correct through EOS | Gated correct through EOS | What happened |
|---|---:|---:|---:|---|
| New held, positive | 48 | 48 | 24 | All 48 requested sources were already correct. The 24 vetoes generated an incorrect first ID. |
| New held, missing / ambiguous / prose | 48 / 48 / 50 | — | — | The gate issued zero copies across all 146. Twenty-four prose rows already had a null role candidate. Actual generation was identical to no-read on all withheld rows. |
| Original focal | 1,728 | 1,610 | 1,610 | Four errors at token 1 moved earlier to token 0. |
| Original renamed | 1,728 | 1,598 | 1,598 | Four errors at token 1 moved earlier to token 0. |
| Earlier primary / fresh | 1,728 each | 981 / 1,056 | 598 / 552 | 383 / 504 exact replies lost. |
| Previous unseen names/wording | 1,728 | 1,406 | 794 | 612 exact replies fell to token 0, and 104 prior token-1 errors moved to token 0. |
| Historical missing and ambiguous | 48 | — | — | The prior role-only path issued 48 sources. The gate issued none. |

Ordinary generation still requires separate language work. With factual evidence retained, withheld replies matched the actual ordinary writer across the entire generated prefix, as the control requires. The normal generation path failed the new prose references. For example, “What color is fresh snow?” began with native ID 1155 against expected ID 1605. Preventing the use of an unsupported source did not produce the correct answer. [Writer instrument and reproduction](writer/grade-wick.sh)

## Why the 24 positive replies disappeared

All 24 vetoed positives had the held question form “Regarding [person], what did the entry name?” The frozen role selector found the requested person and the correct value. The gate then nominated the word *name* (token 1412) as the best independent query word. This word had zero earlier identical occurrences and disagreed with the bound entity. Its nearest positive distance was 404 against 224 for the nearest negative. No tie occurred. All 24 accepted positives used the other wording family; their chosen and nominated words agreed on a single earlier match, and their distances were 304 and 376 respectively.

The 24 vetoes share **one** 14-word descriptor. It occurs in 24 held copy targets and zero held generate targets and is absent from both TRAIN banks. An independent post-seal Rust census reconstituted each full descriptor and verified the recorded nominee and nearest-class distances. Across the 194 held rows it found 53 distinct descriptors including null, zero contradictory target classes at a common descriptor, and zero nonnull rows whose full descriptor occurred exactly in TRAIN. A table fitted retrospectively to these 194 labels could separate them. The new context's nearest TRAIN exemplar belonged to the wrong class; the data do not establish that the same feature suffices on another language family. The remaining scientific question is how an unrestricted query-word scorer identifies the phrase's referent during training on genuinely different constructions. [Sealed collision census](gate/HELD-COLLISIONS.json) · [Independent census code](gate/audit/driver.rs)

## Implementation and evidence

A single native bit forwards the original role-only source or clears it before ordinary generation. TRAIN wrote two positive and eighteen negative unique prototypes from 125 eligible rows of 194. Zero eligible TRAIN descriptors had conflicting target classes. Bank size was 2,109 serialized JSON bytes per seed; the twenty 14-word descriptors contain 1,120 raw bytes in total. The forced-read, no-read and ungated controls use the identical model and writer. No threshold was changed after the preregistration. [Native reference operator](gate/operator.rs) · [Finite parity and release](gate/finite/release-receipt.txt)

The two complete fits and repeats across seven panels on each seed took 23.98 elapsed seconds on the shared Wick RTX 4090, including transport, host checks and lock waits. GNU time reported a peak 772,204 KiB host RSS. This is the action selector phase alone; writer rollout and panel generation are separate. Both fits and all 14 blind handoffs reproduced byte-for-byte. The device, host and separate Rust reference compared all 20 native words, including action, nominee, match count, features and class distances. The archive was downloaded from the pinned revision and compared byte-for-byte. [Wick gate archive, SHA-256 701f534cd5af11014dd639ee7b2e52ca9d23a8023a11f24be42b585db8e03258](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/40f22774c0f22f0b95d1fcd89277523ca32ff329/results/codex/role-boundary-count-20261118/action-gate-wick.tar.gz)
