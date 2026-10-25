# Prose panel on the lawful sieve compiler (`m3-nonrelation-prose-panel`, #2856, claude-94)

- **Status:** pre-registration, written before any panel output exists. A fresh FIT and a pre-run review follow on #2856. Lab: claude.
- **Supersedes** the run in [`preregistration-DRAFT-2026-10-22.md`](preregistration-DRAFT-2026-10-22.md). That run used the saved softmax compiler, which CHARTER §3.2 bans (owner log on #2856, comment `6394013509`). The panel, grader and owner bar are unchanged.

## 1. Question

Does ordinary prose compile to a store write in the world=v2 schema when the compiler is lawful? And does that compiler still compile true relation statements, so that a clean panel cannot come from refusing everything?

## 2. The compiler (`crates/uor-r4-training/src/sieve_frame_compiler.rs`, kernel `src/sieve_frame.cu`)

- **Words are primes.** Each distinct word of the world=v2 training phrasings gets the next prime in sorted word order. A word set is the squarefree product of its primes, which the kernel holds as its prime-factor bitset (gcd is AND, divisibility is subset).
- **Frames are exact memory.** Each training phrasing of the 10 relations (assert, update, query) is one frame. Its routing expert is `p_relation · q_act`. Development phrasings and all values stay out of the table.
- **Decision, on the GPU, one thread per turn:**
  1. **sieve:** keep the frames whose signature divides the turn's;
  2. **align:** the turn must be exactly prefix, then a value of 1 to 4 words, then suffix (for a query, exactly the frame);
  3. **agree:** the turn compiles only if every surviving frame names one expert and one span. Otherwise it is `Unresolved`.

  There is no float, score, softmax or matrix product in the decision.
- **Word rule (fixed in the ORDER before building):** a word is a maximal run of alphanumerics and apostrophes (`’` read as `'`), lowercased. Each word keeps its byte span.
- **Host steps:** tokenizing and building the tables are data preparation. Decoding and grading read the GPU's stored decisions. No compile decision runs on the host (§3.5).

## 3. Arms, in one launch

| Arm | Turns | Scored as |
|---|---|---|
| **P, the panel** | the 50 frozen turns | `grade_prose_panel`, a fresh seeded store per turn, writes on |
| **R1, training phrasings** | every training phrasing filled with 8 development values (`Rel::draw`, seed 2856, the constants `REGISTERED_VALUES_PER_PHRASING` and `REGISTERED_SEED`, passed explicitly by the job), and each query phrasing once | the exact act, relation and value span |
| **R2, development phrasings** | the same, on the unseen development phrasings | the same |

## 4. Bars and what each outcome means

- **Registered bar (owner, #2028 comment `6356034555`):** false_overwrite 0/50, false_assert 0/50, clean 50/50.
- **Recall floor (decision review):** R1 at least 95% exact. The panel bar counts as met only together with the floor.
- **R2 has no bar.** It is reported as it comes, as the size of the generalization gap.

| Outcome | Meaning | Unlocks |
|---|---|---|
| An instrument check fails | an instrument fault, never a result | repair, then rerun |
| R1 below 95% | the compiler refuses true statements, so the panel reading proves nothing | fix the frames or alignment from R1's misses alone, then re-register |
| Floor met, panel 0/0/50 | the bar is met for the sieve compiler on world=v2, reported beside R2 (the floor replays the training frames, so only R2 shows how far the compiler reaches beyond them) | close #2856 with the rates; `m3-memory-turn-discriminator-gate` stays open (a broader compiler must keep 0) |
| Floor met, false counts above 0 | the baseline is the counts | close #2856 with the rates; they are the discriminator gate's target |

Any result is scoped to this configuration (D22). It is never a mechanism verdict, and never a pass of the discriminator gate.

## 5. Instrument checks

1. **Finite CUDA controls (`prose-panel-sieve controls`):** 12 hand-built turns over 6 hand-built frames, each with a literal expected decision. They cover the value cap, an empty value, a sieve reject, an order fail, a query exact and over-long, agreeing duplicates, disagreeing experts, frame words inside a value, and the empty turn. All 12 must match.
2. **Determinism:** the run launches twice in-process, and the decisions must be identical. The run also writes `decisions.json`: every turn's source and decision, in launch order, with no timings. Its sha256 is the record's `headline.decisions_sha256`. A second build (the bootstrap `cuda` build against the lean `sieve-cuda` build) must write a byte-identical `decisions.json`.
3. **D29 interlock:** `cargo test --features sieve-cuda --test d29_compiled_modules` on the pod. The built `sieve_frame.ptx` must carry no `ex2`, `lg2`, `tanh`, `mma` or `wmma` instruction.
4. **Host tests:** `cargo test -p uor-r4-training --lib` (72 at this sheet's commit), including the digest pin and the squarefree-signature identity.

## 6. Pins

| Object | Digest |
|---|---|
| Panel `data/panels/nonrelation-prose-v1.txt` | `7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7` (checked by `load_frozen`) |
| Frame table (`FrameTable::world_v2`, the compiler artifact) | `379c2ecc4f1d04b49168d76e43ac93f3d07730e1016e0a281608740efb46604f` (pinned by a test, and passed as `--expect-table`) |
| `sieve_frame.ptx` | recorded by the controls step, then passed as `--expect-ptx`, which the run requires (it refuses to start without it) |
| Control draw | seed 2856 and 8 values per phrasing. The host test `the_registered_controls_carry_exact_spans_and_fit_the_slot` checks exactly these controls: every value is 1 to 4 words and starts and ends on a word |
| Binaries | their sha256s are recorded beside the outputs |

## 7. Cost and stop rule

- One RTX 5090 on Vast, at most 1 hour of lease, about $1 or less.
- The probe ladder: (a) the controls and the D29 test, (b) the run's own launch time as the throughput check (about 900 turns, well under a second), (c) the run.
- **Stops:** a pin mismatch stops before any launch. A failed control stops the run, and the fault is repaired. Outputs go to the HF store (`results/claude/prose-panel-sieve-2026-10-25/`, with a SHA256SUMS) before the pod comes down.
- **Lead metric:** tokens/s over all turns in the launch, upload to download.
