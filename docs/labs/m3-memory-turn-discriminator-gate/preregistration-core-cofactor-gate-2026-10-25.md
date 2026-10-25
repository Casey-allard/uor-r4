# Core-cofactor marker gate on the lawful sieve compiler (`m3-memory-turn-discriminator-gate`, #2855, antigravity-63)

- **Status:** executed on GPU (`vast-57652110`, 1x RTX 5090, ref `67852848263f7df71362efaa411b2e7f50ebc6b7`) after fresh-context `TEST FITNESS: FIT` (`#2855` comment `6396634979`) and fresh-context `D67 PRE-RUN REVIEW: GO` (`#2855` comment `6396643627`). Registered bar met (`0/50`, `0/50`, `50/50`, `R1 = 462/462`, `R2 = 8/220`; `record.json` sha256 `6b9b7dd4704e5596cb2597aeed16a9b11ed9baa2b76f7b5c39e7f985ddb69c22`, HF `results/antigravity/prose-panel-gate-2026-10-25/`). Lab: antigravity.
- **Extends** [`preregistration-marker-gate-2026-10-25.md`](preregistration-marker-gate-2026-10-25.md) (`claude-102`) after the cancel probe showed that one-sided prefix markers alone (`MarkerTable`, 6 markers) leave every `Split::Development` phrasing (`R2`) uncompiled because development phrasings recombine two-sided markers (`please ... now`, `i meant`, `really`), shared cross-relation update prefixes (`i changed my mind`), contraction pairs (`what's` / `what is`, `i'm` / `i am`), and nominal/entity/preference shells across relations that already appear within `Split::Train`.

## 1. Question

Can the lawful prime-sieve compiler compile `Split::Development` wordings (`R2 > 0/220`) while keeping `R1 >= 95%` (`>= 439/462`) and achieving `false_overwrite = 0/50`, `false_assert = 0/50`, `clean_unresolved = 50/50` on the frozen non-relation prose panel (`P`), using only integer squarefree prime-signature divisibility, exact non-slot token alignment (`1..=79`), and two-sided cofactor markers plus fourth-proportional core/shell completions derived strictly from `Split::Train` (`FrameTable::world_v2()`)?

## 2. The mechanism (`sieve_marker_gate.rs`; kernel `sieve_frame_compile_marked` in `sieve_frame.cu`)

- **Base one-sided markers (`MarkerTable`):** unchanged at sha256 `cd79399490c0abcfc14253b5680a282fc840672c5932e925d62a3b95375a125d` (6 markers: `actually`, `people`, `please note that`, `remember`, `sorry`, `sorry i meant`).
- **Core-cofactor table (`CoreCofactorTable::derive(&table)`):** derived strictly from `table.frames[0..84]` (`Split::Train` only, zero hardcoded English words, no development phrasing or panel row read):
  - **Two-sided cofactors, left-division closure, pre-slot cofactors, and cross-relation shared update prefixes:** discovers 12 prefix markers (`actually`, `i changed my mind`, `i meant`, `instead`, `note that`, `now`, `people`, `please`, `please note that`, `remember`, `sorry`, `sorry i meant`), suffix update markers (`now`, `instead`), and pre-slot update marker (`really`).
  - **Fourth-proportional core/shell completions (`derived_frames`, 1095 frames, mapped back to canonical `0..84` via `frame_base`):** closes contraction pairs (`what is <-> what's`, `i am <-> i'm`), single-word update infixes (`new` after determiners `my`/`the`, `just` after `i`), nominal cores (`name`, `dog's name`, `friend`, `best friend`, `friend's name`, `best friend's name`, `hometown`, `lucky number`, `code word`, `job`, `home`, `favorite food`, `favorite color`), entity cores (`dog`, `friend`, `best friend`) across `called`/`named`/`renamed`/`who is my`, preference category cores (`food`, `color`) across `what`/`which` and `love`/`like`, location/occupation/personal completions, and suffix/pre-slot/`just` closures.
  - Pinned at `WORLD_V2_CORE_COFACTOR_TABLE_SHA256 = b5bd388410a82f14ca6a855f0e420b159f99ad1dd77467e46f33b390ead829d4` by unit test `core_cofactor_table_is_derived_from_training_frames`.
- **Order-independent exact-explanation dominance (`sieve_marked_reading` in `sieve_frame.cu`):**
  - Every non-slot token in prefix, suffix, and marker must be an exact `1..=79` lexicon match (`0u32` out-of-lexicon tokens never match outside `{v}`).
  - Uses a 2-bit `disagree` state (`bit 0`: equal-hole expert disagreement; `bit 1`: non-nested hole or slot-vs-query disagreement). When two slot readings have strictly nested value holes ($H_{\text{inner}} \subsetneq H_{\text{outer}}$), the inner reading explains a strict superset of the turn's words as marker + frame words (`hl_inner < hl_outer`) and refines the outer reading regardless of frame iteration order (`*disagree &= ~1u`). Equal holes (`hs == *hole_start && hl == *hole_len`) require identical experts (`if (e != *expert) *disagree |= 1u`). Non-nested holes set `*disagree |= 2u` monotonically (`STATUS_DISAGREE`).
  - When `sig_words > SIEVE_MAX_SIG_WORDS`, `sieve_frame_compile_marked` zeros all six output arrays (`out_status`, `out_frame`, `out_act`, `out_hole_start`, `out_hole_len`, `out_survivors`).
  - In `decode_gated`, `(ACT_QUERY, Act::Query)` additionally requires `hole_start == 0 && hole_len == 0`.

## 3. Arms, in one launch

| Arm | Turns | Scored as |
|---|---|---|
| **P, the panel** | the 50 frozen turns (`7c29c665…`) | `grade_prose_panel`, a fresh seeded store per turn (`seed_all_relations = true`, `allow_writes = true`) |
| **R1** | training phrasings with 8 development values (seed 2856, 462 turns), and each query phrasing once | exact act, relation and span |
| **R2** | development phrasings with 8 development values (seed 2856, 220 turns), and each query phrasing once | exact act, relation and span |

## 4. Bar and outcomes

- **Registered bar.** All four must hold:
  - the owner's panel bar: `false_overwrite = 0/50`, `false_assert = 0/50`, `clean_unresolved = 50/50` (#2028 `6356034555`);
  - `R1 >= 95%` (`>= 439/462` exact);
  - `R2 > 0/220` (strictly above `claude-94`'s `0/220` baseline);
  - the exact `R2` count is reported (`compiles R2 x of 220`, never `"generalizes"`).

| Outcome | Meaning | Unlocks |
|---|---|---|
| An instrument check fails | an instrument fault | repair, then rerun |
| Panel false count above 0 | a derived frame or marker lets non-relation prose write | negative for this configuration (D22); narrow the completion rule |
| Panel clean (`0/0/50`), `R1 >= 95%`, `R2 > 0/220` | registered bar met for this configuration | `m3-memory-turn-discriminator-gate` -> `status: done`, `Closes #2855` |
| Panel clean (`0/0/50`), `R2 == 0/220` | no development phrasing compiled | negative for this configuration (D22); item stays open (`References #2855`) |
| `R1 < 95%` | derived frames or markers conflict with training phrasings | negative for this configuration (D22); narrow the completion rule |

## 5. Instrument checks

1. **Finite CUDA controls (`prose-panel-gate controls`):** 16 hand-built turns over 8 frames and 2 markers, including the 14 base cases plus case 15 (`nested slot span refines outer span`, `status = 1`) and case 16 (`non-nested slot spans disagree`, `status = 2`). All 16 must match (`failed == 0`).
2. **`claude-94`'s 12 sieve controls (`prose-panel-sieve controls`):** all 12 must match (`failed == 0`) on the same compiled `sieve_frame.ptx`.
3. **Determinism:** two in-process GPU launches in `prose-panel-gate run` must produce identical decisions (`deterministic_second_launch == true`), and a second build (`--features cuda,sieve-cuda` vs `--no-default-features --features sieve-cuda`) must produce byte-identical `decisions.json`.
4. **D29 PTX interlock** on both feature sets (`sieve-cuda` and `cuda`): no `ex2`, `lg2`, `tanh`, `mma`, or `wmma` instructions.
5. **Host checks:** `cargo test --no-fail-fast -p uor-r4-training --lib` passes 76/76 tests (including both `WORLD_V2_MARKER_TABLE_SHA256` and `WORLD_V2_CORE_COFACTOR_TABLE_SHA256` pins), and `ratchet d23`, `d25`, `d32` are at baseline.

## 6. Pins

| Object | Digest |
|---|---|
| Panel (`data/panels/nonrelation-prose-v1.txt`) | `7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7` (checked by `load_frozen`) |
| Frame table (`FrameTable::world_v2()`) | `379c2ecc4f1d04b49168d76e43ac93f3d07730e1016e0a281608740efb46604f` (`--expect-table`) |
| Base marker table (`MarkerTable::derive`) | `cd79399490c0abcfc14253b5680a282fc840672c5932e925d62a3b95375a125d` (`WORLD_V2_MARKER_TABLE_SHA256`) |
| Core-cofactor table (`CoreCofactorTable::derive`) | `b5bd388410a82f14ca6a855f0e420b159f99ad1dd77467e46f33b390ead829d4` (`--expect-markers`, `WORLD_V2_CORE_COFACTOR_TABLE_SHA256`) |
| `sieve_frame.ptx` | recorded by `prose-panel-gate controls`, then passed as required `--expect-ptx` |
| Binaries and job script | recorded in `SHA256SUMS` beside the outputs |

## 7. Cost and stops

- One RTX 5090 on Vast, at most 1 hour of lease, budget cap $1.50 (expected wall time ~5 minutes).
- **Probe ladder:** (a) `prose-panel-sieve controls` (12/12), `prose-panel-gate controls` (16/16), and D29 PTX checks; (b) `prose-panel-gate run` under `sieve-cuda`; (c) `prose-panel-gate run` under `cuda,sieve-cuda` and byte-identical `decisions.json` comparison.
- **Stops:** any pin mismatch or control failure stops before the panel run; outputs (`controls.json`, `sieve-controls.json`, `record.json`, `decisions.json`, `cuda-record.json`, `cuda-decisions.json`, `SHA256SUMS`) are uploaded to `caseyallard/uor-r4-store:results/antigravity/prose-panel-gate-2026-10-25` before `uor-pod down`.
- **Lead metric:** tokens/s, upload to download.
