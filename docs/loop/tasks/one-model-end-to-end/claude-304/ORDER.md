# ORDER: one-model-end-to-end, claude-304

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `2578cd6b4` (`origin/main`, 7 Nov 2026; claude-303 merged).

## Goal

This serves Casey's one track (#2028 6675163438, "Yes, one track"): the trunk's no-memory top-1 must beat its TRAIN's unigram and its T2 on the human held-out panel, beyond the paired band, at every seed.

claude-303 (#3809, merged as `2578cd6b4d10d2d2beae24dcdc6692424d56e1f0`) fixed width 1,024's memory: I-m passed at micro-batch 16 at both seeds (38,621 and 38,571 of 97,887 MiB). Both seeds then stopped at I-t:
- 3.578 and 3.636 s a step project 66,641 and 67,618 s, against claude-302's bound of 57,600 s;
- no arm trained, so the width lever, the bar and the spread are all still unread.

This unit trains (w1024), under a sheet that amends claude-302's in two places:
1. **Faster sizes:** smokes at micro-batch 64 (the whole batch), 32 and 16. The fastest that passes I-m and I-f trains.
2. **A longer wall:** I-t's bound is raised from 57,600 s to 72,000 s, by the decision review in the sheet.

## Claim check

- **(a) The step:** Casey's one track. This is the run claude-302's outcome map sends a NOT_RUN back to: rerun, with the instrument that stopped it repaired.
- **(b) What it changes in the model:** this unit trains the first width-1,024 trunk on (b64L)'s recipe, then scores it on the human held-out panels against T2, the unigram and (b64L).

## Cancel probe (run first)

Run on 7 Nov, before any work. Class: instrument.
1. **No hold:** `loop next` shows the kill switch off.
2. **No running pod of mine:** `uor-pod status` shows 0 running pods.
3. **The trainer is claude-303's:** `git diff fd90aa1c16e2af35d7fa52d101726e7fe6927953 2578cd6b4d10d2d2beae24dcdc6692424d56e1f0 -- crates/` is empty.

Result: GO.

## Done when

1. A sheet amending claude-302's, with a non-author FIT and a D67 pre-run review of the job, chain, seal and watch scripts. The job's gate tests on the laptop must pass, with each mutation caught.
2. On two pods, one per seed:
   - I-0 and I-e pass, and the smokes choose a size;
   - (w1024) trains 16,384 steps and is scored on the four panels, or a registered gate stops it;
   - the run is sealed with a SHA256SUMS read back, and the pod is down.
3. If both seeds trained: on the laptop, I-r, then `grade` and `pair` against T2, the unigram and (b64L), as claude-302's Readings say.
4. A REPORT with the negatives first, one row in `docs/research/notebook/float-training.md`, `References #3407`, and a [claude, for Casey] note on #2028 that logs the wall-bound decision.

## Stop and ask if

"Ask" means decide, record it here and in the REPORT, and go on; nothing waits.
- **The cap:** the sheet's cost section sets it.
- **Casey says "hold",** or reverses the wall-bound decision.
- **The laptop's free disk falls under 5 GiB.**

## Files in scope

- `docs/labs/one-model-end-to-end/**`
- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`
- `crates/uor-r4-float-train/src/hamilton_sheet.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/flock_read_memory.rs`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `crates/uor-r4-training/examples/m-world.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-training/src/bin/m-world-recall-corpus.rs`
- `crates/uor-r4-training/src/bin/mix-chat-corpus.rs`
- `crates/uor-r4-training/Cargo.toml`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-training/src/milestone_world_v2.rs`
