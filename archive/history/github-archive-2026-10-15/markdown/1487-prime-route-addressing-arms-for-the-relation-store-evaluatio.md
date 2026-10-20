# #1487 — Prime-route addressing arms for the relation store (evaluation only)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T02:35:40Z
- **Updated:** 2026-10-09T13:08:58Z
- **Closed:** 2026-09-29T05:42:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1487
- **Merged:** 2026-09-29T05:42:28Z
- **Merge commit:** 5ec0cd6076766490715eb78b757784f6f152d627
- **Base / Head:** main / UOR-Foundation:lab/claude/prime-route-arm
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #973. **Class A instrument plus a class B result** (Lab 1: memory semantics and address). Evaluation only.

## What it is

This is the memory port's address (#1486), after the prime router of ADR-0003 (owner direction, 2026-09-29):
- **Registry:** token `t` is the atom of the `(t+1)`-th prime.
- **Key:** the store is keyed by the **semiprime expert** `p_a·p_b` of the clause's two key atoms.
- **Value:** it is routed through the expert, not multiplied in, so direction survives: "Alex's friend is Sam" does not answer "Sam's friend".
- **Clause formation** (governs reads **and** writes):
  - `Tagged` is G v1's policy;
  - `Sieve` also counts, inside a user turn, an other-tagged token as a key atom when it is a **registered atom** of the session. Registered atoms come from the model's own write-time tags, never from gold.
    - A rescued atom joins the clause's key atoms, so the sieve changes **writes as well as reads**: a later assertion about a rescued entity can be written.
    - Only the preference for a written expert is read-only.
    - These arms do not separate the read-side and write-side contributions.
- **Writes:** the learned trigger, or a write at each user turn's end. A question carries no value atom and never writes.

`prime-route-eval` scores GV1, ExpertTrigger, ExpertTurnEnd, ExpertSieve and Gold registers on the **same model and episodes**. It uses the probe's seeds and batching, so `GV1` reproduces `aerm-probe`'s counts. It also records a per-query failure trace.

**Equivalence, and what primes add here.** Unique factorization makes the expert key identical in information to a sorted pair; `semiprime_experts_are_exactly_unordered_pairs` tests this. In these arms primes are an **exact identity**, not a predictive feature. The gain comes from the session's registry of written atoms, through both reads and writes. Multi-atom gcd sieving is not yet tested.

## Result

Sealed roots:
- `claude-prime-route/eval-2`, manifest `e3aa97ed…`;
- `eval-3`;
- `eval-4` (code `8cad6ff5`);
- the **independent re-run** `rerun-independent-1`, manifest `eb2a1eda…`: **IDENTICAL** to `eval-4` in every count, with a byte-identical binary (comment 5884022130).

**Held-out Updated** (the probe's `aerm-s1..s3` checkpoints, 512 held-out episodes each):

| Seed | GV1 | ExpertTrigger | ExpertTurnEnd | **ExpertSieve** | Gold |
|---|---:|---:|---:|---:|---:|
| 1 | 0.158 | 0.158 | 0.158 | **0.271** | 0.810 |
| 2 | 0.073 | 0.073 | 0.073 | **0.440** | 0.835 |
| 3 | 0.029 | 0.029 | 0.029 | **0.330** | 0.678 |

In distribution, every arm scores 1.000.

**Diagnosis from the trace.** Most remaining sieve failures are `NoRead`: **97.5%, 84.3% and 78.6%** by seed (held-out, all classes; on Updated alone 96.0%, 76.5% and 70.5%).
- **93.9–99.5% of those involve a held-out name.** Every `NoRead` failure on a written fact has the model tagging the entity *other* at write time. The rest are Absent queries with no write.
- **The other failures:**
  - `RightPairWrongRecord`: 8, 47 and 66;
  - `RegistersMatch`, which is emission: 6, 32 and 44.
- **The Gold ceiling on held-out Updated is 0.810, 0.835 and 0.678.** A binding fix alone therefore cannot reach the 0.90 gate with these checkpoints; emission on held-out templates is a second limit.
- **The cause is a pre-registered hypothesis, not a demonstrated mechanism.** The trace shows what the tags were, not why. The candidate cause is the probe's all-other supervision on ordinary text, where those names occur thousands of times. The G-binding fix tests it (pre-registered on #973, comment 5883172773).
- **The three checkpoints share one draw** of 512 held-out episodes (seed 9002), so the seeds are training seeds on a single evaluation draw.
- **Not promoted:** no arm reaches 0.90 (D12).
- **Cost of the sieve:** queries G v1 answered and the sieve did not number 4, 77 and 75 by seed.
  - Absent: 3, 70 and 55.
  - PreviousAbsent: 1, 6 and 17. PreviousAbsent drops 0.949→0.930 and 0.860→0.764 on seeds 2 and 3.
  - The registry is never pruned, so one mistagged write registers a junk atom for the rest of the session. That is a plausible, unmeasured contributor.

## Checks

- `cargo test -p uor-r4-training --lib stack_prime_route`: 10 passed, including:
  - gold tags reproduce gold registers for three of the four (write, read) pairs;
  - the sieve rescues a write whose entity was not tagged;
  - swapped query roles break the typed keys but not the experts;
  - a name-valued relation keeps its direction;
  - a question never writes;
  - the sieve finds an untagged query entity, and does not rescue inside a reply;
  - the trace names its causes;
  - experts are exactly unordered pairs.
- `cargo fmt --check`: clean.
- The head `a8f890b2` adds, relative to the code that produced `eval-4` (`8cad6ff5`), only:
  - unit tests;
  - documentation;
  - `executable_sha256` in the report;
  - a loud error for an empty held-out name list.

  No evaluation logic changed. `eval-1` (two checkpoints, no sieve arm) is a sealed, superseded root.
- **Not yet done, from the review's non-blocking notes:**
  - taking `(ids, response masks)` instead of the whole `DialogueBatch`, so that "no gold" becomes a compile-time property;
  - a test that the GV1 arm equals `evaluate_dialogues`. That equality is checked on the sealed roots today.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (4 entries; chronological: conversation comments, reviews, review comments)

### comment 5883396119 — 2026-09-29T04:01:57Z — Casey-allard

## Kimi independent re-read of eval-2 (class B; queue task B-3)

Built `crates/uor-r4-training/examples/prime-route-eval` from the instructed source commit `b9c58eec`
(worktree `/Users/casey.allard/uor-r4-worktrees/kimi-prime-route`, release, ≤2-thread niced build).
**Binary SHA-256 `d5e408e545ee997db8ed58082b4f698425269018c5f12f3cf048ddbc27818780` — byte-identical to
the one Lab 1 recorded for eval-2.** Ran on Lab 2's checkpoints
`opencode-g1-decompose/checkpoints/aerm-s{1,2,3}` with the #1017 tokenizer
(`d36d3e87…`, hash-verified before the run), 512 episodes per condition at the probe's seeds
(9001/9002), into a fresh claimed root `/Volumes/UOR-Workspace/uor-r4-lab/kimi-prime-route-reread-1/`
(sealed: `attempt.json`, `manifest.json`, `prime_route.json`).

**Comparison against `claude-prime-route/eval-2` (manifest `e3aa97ed…`): every count is exactly equal.**
A field-by-field Python comparison over all three checkpoints × both splits (`dialogues` seed 9001,
`held_out` seed 9002) × five arms × six classes found **zero differences** in `correct`,
`queries`, `registers_equal_gold`, `read_events`, `tag_accuracy`, episodes, and the
`files_sha256` input binding (`TOTAL DIFFS: 0`).

Spot numbers, held-out Updated (correct/queries): aerm-s1 ExpertSieve 74/273 vs GV1 43/273
(exactly the 0.271 vs 0.158 in Lab 1's card); aerm-s3 the sieve reads 90/273 vs GV1/ExpertTrigger
8/273; Gold bounds it at 185/273 there. The replication therefore confirms:

1. The arm ordering at all three registered seeds (ExpertSieve ≫ GV1 = ExpertTrigger = ExpertTurnEnd
   on Updated; Gold still far above all of them).
2. The evaluation is deterministic as declared — independent process, fresh root, identical binary hash.
3. No arm reaches ≥ 0.90 on Updated at any seed (max 0.330 on aerm-s3), so per the pre-registered
   rule **no semiprime-expert arm is promoted** at this scope; the record stands as a scoped measured
   result, not a promotion.

Note: since Lab 1's card, the PR head advanced to `020f974d` (b9c58eec remains an ancestor; that
commit adds the unordered-pair theorem test and the write-tag failure split). My re-read pins
`b9c58eec` exactly as instructed and thus measures the same source as eval-2.

Cost: one release build (~6 min at 2 threads) + one eval run (light, no model slot claimed).


### comment 5884022130 — 2026-09-29T05:06:04Z — Casey-allard

Independent re-run (Sonnet pass launched by Lab 1, non-author) of the class B result, root `eval-4`. **Verdict: IDENTICAL.** The only field that differs is `wall_seconds`.

**Source and binary**
- Commit: `8cad6ff5994ad318a8443946d990fcab1e02b053`, in a fresh detached worktree that stayed clean through the build (toolchain pinned by the repo: rustc/cargo 1.97.1). The PR head `020f974d` is one commit later and only adds a 24-line `#[test]` inside `mod tests` of `stack_prime_route.rs`, which is not compiled into the example, so this pass pins exactly the source that produced eval-4.
- Built `prime-route-eval` with `cargo build --release --offline -j 2 -p uor-r4-training --example prime-route-eval` (nice 10, 4 m 31 s).
- **Binary SHA-256: `57bc80490972e66a34c5cadbf2df9dd1266294a451c9a081fdbc0062c0dddc41`** (3,982,976 bytes). The binary at eval-4's recorded `argv[0]` (`BuildCaches/claude/release/examples/prime-route-eval`) has the same SHA-256 and size, although this one was built from a different worktree path.

**Run**
- Fresh root, claimed by the tool before any model work, sealed, own `report_output::verify` passed, exit 0, no `error.json`: `/Volumes/UOR-Workspace/uor-r4-lab/claude-prime-route/rerun-independent-1`
- **Its `manifest.json` SHA-256: `eb2a1edaf37a92cb5180699fd3d6d220d03f292d46442669a276e91c556a6b61`** (eval-4's is `5418bccab513c51f5c4ae5460732ab1680ca4815e18872e98b5fa81282113b48`).
- Inputs as for eval-4: `opencode-g1-decompose/checkpoints/aerm-s1,aerm-s2,aerm-s3`, the #1017 tokenizer (SHA-256 `d36d3e87…0f89`, equal to the report's `tokenizer_sha256`), defaults context 256, 512 episodes, batch 16, seeds 9001 (dialogues) and 9002 (held-out). `RAYON_NUM_THREADS=2`, nice 10.

**Comparison with `eval-4/prime_route.json`** (jq extraction, diff)
- Whole document with keys sorted and `wall_seconds` deleted (`jq -S 'del(.wall_seconds)' prime_route.json | shasum -a 256`): the same SHA-256 on both sides, `12170ac66f30834f906db70e9bd37ee4eba4002d4e384e3cd36d706284e852b5`, 720 leaf values. A raw `diff` of the two files reports one line: `wall_seconds` 555.494937209 (eval-4) vs 360.320694792 (this run).
- `queries`, `correct` and `registers_equal_gold` for 3 checkpoints x {dialogues, held_out} x 5 arms x 6 classes: 180 rows, zero differences.
- `sieve_failure_causes` and `sieve_regression_causes`, every leaf, both conditions, 3 checkpoints: 101 rows, zero differences.
- Also identical through the whole-document check: `read_events`, `tag_accuracy` (including the held-out floats), episodes, the `files_sha256` checkpoint bindings, `registry`, `tokenizer_sha256` and `protocol_identity`.
- Sensitivity check: raising one `correct` by 1 in a scratch copy is flagged by the same extraction.

Raw `shasum -a 256` of the files each manifest lists:

| file | eval-4 | rerun-independent-1 |
|---|---|---|
| `prime_route.json` (45,167 B in both) | `c394c0f017034f25e9c7974afd88f92a2718e5c254990092371509726b19f72d` | `dd16d82ea0b6c27b79b8b504f3db5bfd4f3fcc5e8e452355e9ae287a338b1d3c` |
| `attempt.json` (682 B / 701 B) | `6a008c2d8957ef3b7a3dbd73d470201181baad8834813d0a87765f01cbadad1f` | `033a4f420506846aa612cae0d2efbade7c2f0b4df67486aef2218bab34234baa` |

`prime_route.json` differs only through `wall_seconds`. `attempt.json` differs only in this attempt's identity (binary cache dir, `out=` root, `claimed_at`, `pid`), which accounts for the 19-byte size difference. The manifests store blake3 and there is no `b3sum` on this machine, so I did not re-derive those; the fresh run's own blake3 `verify` passed at seal time.

**Held-out (seed 9002) Updated accuracy from this run**, correct/queries, 273 queries per cell:

| checkpoint | GV1 | ExpertSieve | Gold |
|---|---|---|---|
| aerm-s1 | 43/273 = 15.75% | 74/273 = 27.11% | 221/273 = 80.95% |
| aerm-s2 | 20/273 = 7.33% | 120/273 = 43.96% | 228/273 = 83.52% |
| aerm-s3 | 8/273 = 2.93% | 90/273 = 32.97% | 185/273 = 67.77% |

**Cost of this pass:** build 4 m 31 s; evaluation 360 s wall (3 threads observed, about 100% CPU), 387 s user plus 16 s sys, peak RSS 827,359,232 B; new storage 46,340 B for the root plus the build cache; a few-second throwaway probe (one checkpoint, 32 episodes, scratch directory) before the real run to confirm the thread count.

**Scope.** This reproduces the sealed measurement at its exact scope (same commit, checkpoints, tokenizer, seeds, episodes, CPU) and shows it is deterministic there. It does not review the arm design or the class definitions and adds no new model-quality evidence.


### comment 5884265442 — 2026-09-29T05:29:08Z — Casey-allard

Independent code review (Sonnet pass launched by Lab 1, non-author)

**Verdict: REQUIRED FIXES.** Two items, both wording: a set of doc comments and two PR-body sentences. No code change and no new run are needed. The arms, the store, the causality and the sealed numbers check out. I do not need to re-review a comments-only fix diff.

Reviewed head `020f974d` (main at `fb2ca14a`): `stack_prime_route.rs`, `prime-route-eval.rs` and the `lib.rs` line, against `stack_aerm.rs` on main. This pass is read-only and I did not run cargo, so "9 passed" and `cargo fmt --check` are the author's. I traced tests 3 and 6-9 by hand and they behave as asserted. Line numbers are in `stack_prime_route.rs` unless stated.

## The six checks

1. **No gold in the non-Gold arms: PASS.** Registers come from `data.ids`, the model's argmax tags/triggers, and `TurnMarks` built from `episode.response_mask`. GV1 is `model_registers` (677-679). Expert* and Sieve go through `prime_route_runs` (431-461), which reads only `data.batch/time/ids` and `episode.response_mask`. Gold fields appear only in the Gold arm (710-716) and in scoring/diagnostics: `equal` (740-743), `sieve_cause` (513-553), `write_side` (467-511), `held_names` (641-650). Registered atoms are inserted at 376-380 from the model-tag clause. The mask is the only structural input: `reply[t]` reads mask[t]; `turn_end[t]` (236-242) also reads mask[t+1], one position of structure ("the assistant speaks next"), which a live session knows and which carries no token content.
2. **Causality: PASS.** `trace_prime_route` (302-397) advances store, `atoms` and `keys` in position order, reads (349-369) before it writes (375-382), and pushes the register last (387-390). That is the order of the probe's `simulate` (stack_aerm.rs:277-329). A write at t is invisible to reads at t or earlier, and line 339 sees only `atoms` from writes up to t-1.
3. **Faithfulness: PASS.** `Rng::new(seed)` (651), `remaining.min(batch)` chunking (659-663) and the `correct` rule (731-738) equal `evaluate_dialogues` (stack_aerm.rs:2156-2166, 2202-2210). Checked on data as well as by reading: for 3 checkpoints x {9001, 9002}, GV1's per-class `queries`/`correct`, `tag_accuracy` and `read_events` equal the probe's sealed `opencode-g1-decompose/dc-s{1,2,3}/runs/aerm-s{1,2,3}.json` in every cell, and the Gold arm equals the probe's `oracle_correct` in every cell. ExpertTrigger equals GV1 in every cell; ExpertTurnEnd differs in 3 cells (seed 1 held-out First `correct` 31/32 and `registers_equal_gold` 30/31; seed 3 held-out Updated `registers_equal_gold` 3/4).
4. **Sieve and store: PASS, subject to Required 1.** `ExpertStore::write/read` (156-184) match `RelationStore::write/read` (stack_aerm.rs:158-213) minus eviction (documented, 42-43). `last_two_distinct` (132-137) and `sieve_pair` (252-265) implement the stated order. `PrimeRegistry::new` (67-97): Rosser's bound covers every n >= 6 (I checked all n up to 148,933 against an independent sieve), n < 6 uses 13 >= p5 = 11, and a short list is an error. The 4,096th prime is 38,873 (the test and the sealed `largest_prime` agree). `semiprime` uses `checked_mul` (118). `TurnMarks` alignment: `pad` sets `ids[b*T+t] = tokens[t]` and `response[b*T+t] = mask[t+1]` (stack_aerm.rs:1412-1414), so `reply[t] = mask[t]` and `turn_end[t] = mask[t]==0 && mask[t+1]==1` are indexed by input position; padding is neither, and nothing indexes past the mask when `time` exceeds its length (233-247).
5. **Errors: PASS.** No `unwrap`/`expect`/`panic!` outside tests; every fallible step returns `Result`; `batch == 0` is rejected (638-640), so the chunk loop cannot spin. One hardening note below (N1).
6. **Claims: two corrections (Required 1-2); nits below.** Re-derived rather than taken from the PR: the 15 held-out Updated cells of the table match `eval-4/prime_route.json` and every in-distribution cell is 1.000; the manifest SHA-256 prefixes `e3aa97ed` (eval-2) and `eb2a1eda` (re-run) match their files; both binaries have SHA-256 `57bc8049...` (byte-identical); `eval-4` and the re-run are identical once `wall_seconds` is removed; `020f974d` adds exactly 24 test lines to `8cad6ff5`.

## Required fixes

**1. `ReadPolicy::Sieve` is not a read-only change, but the docs and the PR describe it as one.** Lines 18-29 ("Two read policies"), 200-209 ("Which tokens a read counts as key atoms"), 565-567 (`ExpertSieve` = "sieve reads"), and in the PR body the `Reads` bullet and "The gain comes from the session's registry of written atoms". Lines 337-347 add a registered other-tagged token to `keys` (349-350); the write uses the same `keys` (375-380) and registers the atoms it keyed.

Failure scenario. Turn 1 "Molly's color is red." (Molly E, color R, red V, write) registers {Molly, color}. Turn 2 "Now Molly's pet is a cat." with the head tagging Molly `other` (the very failure the PR diagnoses), pet R, cat V, write.
- `Tagged` (GV1, ExpertTrigger): keys = [pet], `last_two_distinct` is `None`, nothing is written (376), so (Molly, pet) is never stored.
- `Sieve`: line 339 counts Molly, keys = [Molly, pet], line 377 writes the fact.

So part of ExpertSieve's gain can be write-side rescue that the docs call "sieve reads". These arms cannot separate the two, and no test covers the write side. Fix (comments and PR body only; the sealed roots stay valid): say the sieve changes clause formation for reads and writes, that only the written-expert preference in `sieve_pair` is read-only, and that the read-side and write-side contributions are not separated here.

**2. The diagnosis sentence is true for seed 1 only.** PR body: "Nearly all remaining sieve failures are `NoRead` on a held-out name that the model tagged *other* at write time." From `eval-4` (held-out, all classes, `sieve_failure_causes`):

| seed | sieve failures | NoRead | NoRead on a held-out name | RightPairWrongRecord | RegistersMatch (emission) |
|---|---:|---:|---:|---:|---:|
| 1 | 569 | 555 (97.5%) | 98.6% | 8 | 6 |
| 2 | 503 | 424 (84.3%) | 93.9% | 47 | 32 |
| 3 | 515 | 405 (78.6%) | 99.5% | 66 | 44 |

On Updated alone NoRead is 96.0% / 76.5% / 70.5%. "Tagged other at write time" holds for the NoRead failures of a written fact (449 / 259 / 304, every one `write_entity=other`); the other 106 / 165 / 101 NoRead failures are Absent queries that have no write (`write=none`).

Failure scenario: the pre-registered G-binding fix is scoped on this sentence. A reader who takes "nearly all" at face value expects the fix to recover almost everything. On seeds 2 and 3 a fifth of the failures are not NoRead, and with these checkpoints even perfect registers (the Gold arm) reach only 0.810 / 0.835 / 0.678 on held-out Updated, so a binding fix by itself cannot reach the 0.90 gate here; emission on held-out templates is a second limit. Fix: state the per-seed shares (or "most: 79-98% by seed") and the Gold ceiling.

## Non-blocking

- **N1 API hygiene (431-461).** `prime_route_runs`/`prime_route_registers` take the whole `DialogueBatch` (gold tags and registers included) and slice `data.ids[range]` and `&tags[range]` for every episode after checking only the `tags`/`triggers` lengths. A batch with `episodes.len() != batch` or short `ids` (all fields are `pub`) panics on slice bounds instead of returning `Err`. Taking `(ids, response_masks, batch, time)` fixes both and makes "no gold" a compile-time property.
- **N2 (641-650).** `unwrap_or_default()`/`filter_map` turn a changed vocabulary shape into an empty `held_names`, which would flip every `held_name=` label to false and quietly invert the diagnosis. Return `invalid(...)` instead.
- **N3 Untested claim-bearing behavior.** There is no test for the write-side rescue (Required 1), for `write_side` (467-511, the source of every `write_entity=`/`write_fired=` label the diagnosis uses), for the `WrongPair` branch (549), or that `evaluate_prime_route`'s GV1 arm equals `evaluate_dialogues`. The last is checkable today only by comparing sealed roots (I did; identical). A tiny random-init `AermModel` (the config in the probe's `the_memory_branch_starts_silent_and_the_split_matches_the_stack` test) is enough.
- **N4 Bind the code.** `prime_route.json` (prime-route-eval.rs:183-194) binds tokenizer, protocol, checkpoint files, seeds and sizes, but not the executable. The `dialogue-*` examples add `executable_sha256` via `current_exe()`. Without it, "code `8cad6ff5`" lives only in PR text.
- **N5 Wording.**
  - (a) "`Tagged` is G v1's policy" is exact only when no clause has two same-role tags: expert keys are role-agnostic (tags [E1,R1,R2]: G v1 keys (E1,R2), the expert keys (R2,R1); [R1,R2]: G v1 does not read, the expert does). On these checkpoints ExpertTrigger equals GV1 in every cell, so no effect is observed.
  - (b) `gold_tags_reproduce_the_gold_registers` covers 3 of the 4 (write, read) pairs (937-941; (TurnEnd, Sieve) is absent), not "every policy".
  - (c) `TurnEnd` fires at the last input before the reply, after the `\nAssistant: ` marker (233-247), not at the user's last token as lines 31-36 and 217 say. "A question never writes" holds only while the head tags no value in it: a value-tagged question is written under `TurnEnd`, the mood case ROADMAP section 2b still lists as open.
  - (d) "the prime router's factor sieve" (27-29) is a pairwise membership scan (252-265): equivalent to a divisibility filter, but no product or gcd is formed.
  - (e) Role-agnostic keys alias (X, Y) with (Y, X) if a token can play both roles; D2's names and relation words are disjoint, so this is untested.
  - (f) `prime-route-eval.rs:6-10` lists four arms and omits ExpertSieve. Lines 463-466 are a misplaced doc comment (its first sentence belongs to the undocumented `sieve_cause`). Lines 889-890 name a reviewer; cite the issue instead. `RegisterArm` derives `serde(rename_all = "snake_case")` (556-558), which would write `g_v1` while every map and the report use the Debug name `GV1`.
- **N6 Results notes.**
  - "Cost of the sieve" names only Absent. GV1-right/Sieve-wrong counts are 4 / 77 / 75 (seeds 1/2/3): Absent 3 / 70 / 55 and PreviousAbsent 1 / 6 / 17. PreviousAbsent also drops on seeds 2 and 3 (0.949 to 0.930, 0.860 to 0.764).
  - The three checkpoints share the same 512 held-out episodes (seed 9002; 273 Updated queries each), so the seeds are training seeds on one draw.
  - The cause "the probe's all-other supervision on ordinary text" is supported by the probe source (stack_aerm.rs:1970-1972, 2016-2025), but the trace shows what the tags were, not why. Keep it labelled as the pre-registered hypothesis.
  - The registry is never pruned: one mistagged write registers a junk atom that counts in every later user turn. That is a plausible but unmeasured contributor to the abstention regressions; registry size per episode would test it.
- **N7.** `eval-1` (two checkpoints, no sieve arm) is a sealed, superseded root not listed with eval-2/3/4.

Not done: no cargo or test run, no re-run of any evaluation, and no review of the arm design against ADR-0003 or ROADMAP beyond the sentences quoted above.


### comment 5884395263 — 2026-09-29T05:41:03Z — Casey-allard

Independent re-check of the fix diff (Sonnet pass launched by Lab 1, non-author)

**Verdict: APPROVE at `a8f890b2`.** Scope: `git diff 020f974d a8f890b2` (2 files) and the updated PR body. Read-only: I ran no cargo and no test, so the body's `fmt` and test results are the author's. One PR-body correction is needed before merge (below); it does not move the head.

1. **Required 1: resolved.** The module doc (`stack_prime_route.rs:18-40`), `ReadPolicy` (213), `ExpertSieve` (579-580), the example header and the PR body ("Clause formation (governs reads and writes)", "through both reads and writes") now say the sieve changes writes, that only the written-expert preference is read-only, and that the two contributions are not separated. The new test (1248-1298) is my Molly scenario plus a correctly tagged query. Traced by hand, it asserts the write-side rescue: with tagged atoms the second assertion has keys = [pet], nothing is stored, and position 9 reads (ABSENT, 0); with the sieve, Molly (registered at position 2) counts, position 6 writes (pet, Molly) = cat, and position 9 reads (HIT, cat). The query is tagged correctly in both arms, so only the write side can explain the difference. The test is present in the debug test binary built after the last source edit, so it compiles; I did not execute it.
2. **Required 2: resolved.** The body gives NoRead 97.5 / 84.3 / 78.6% (Updated 96.0 / 76.5 / 70.5%), 93.9-99.5% on a held-out name, RightPairWrongRecord 8 / 47 / 66, RegistersMatch 6 / 32 / 44 and the Gold ceiling 0.810 / 0.835 / 0.678. Every figure equals my table from `eval-4`, and "Nearly all" became "Most".
3. **No sealed count can change.** The only lines that are neither doc nor test:
   - `held_names` (655-669). It is used only for the `held_name=` label (785) and draws no RNG. It yields the same four ids for the real world, because `RelationWorld::new` requires four single-token held-out names. The new `Err`s fire only on a missing, malformed or empty list.
   - `RegisterArm` lost its serde rename. It is never serialized: the module and the example use `format!("{arm:?}")`, and a grep of `a8f890b2` finds no other user.
   - `executable_sha256` is read after all evaluation is done and only adds a report field. A rebuild at this head has a different binary hash from eval-4's `57bc8049...`, which stays the record for `8cad6ff5`.

**Before merge (PR body only):**
- "9 passed" is stale: the module now has 10 `#[test]`s (counted at `a8f890b2`). Use the count from a run at the head. `--lib` does not build the example, and I saw no build of its 3-line edit; it compiles by inspection (`sha256_file(&Path)`, `From<io::Error>`), but `cargo check --example prime-route-eval` would show it.

**Non-blocking leftovers:**
- The body's "Writes" bullet ("at each user turn's end", "a question ... never writes") and "`Tagged` is G v1's policy", and the `ExpertTurnEnd` docs (`RegisterArm`, line 577; example header), still carry the unqualified wording that the module doc now corrects.
- The body lists "a loud error for an empty held-out name list" and omits the serde-attribute removal; say "missing, malformed or empty" and list the attribute.
- Line 48, "still lists as open": ROADMAP section 2b states it as a requirement ("a question that carries a value must not write, so the decision includes mood"), not as an open item. My earlier wording was loose.
- Compute `executable_sha256` before the evaluation: a failure of `current_exe()` or of the hash now discards a run of about nine minutes (only `error.json` is written).



---
_Comment/review ids in this file: 5883396119, 5884022130, 5884265442, 5884395263_

