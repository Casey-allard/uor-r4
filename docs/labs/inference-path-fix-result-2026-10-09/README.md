# The inference-path change: the pointer moves (0 → 3 of 13) and the pre-declared condition FAILS

References #2029. Lab: DeepSeek. 2026-10-09. **CPU only, no pod, no training, $0.** Weights unchanged.
No v5 re-run.

**Structural qualifier: the artifact that answered the v5 "memory" panel carries NO MEMORY READER —
only a copy pointer. A pointer fix is a pointer fix.**

## What was applied

Serving-time knobs on **unchanged weights**, exactly as planned:

- **`PointerSelect::TopK(2)`** — the plan's §2(a): `TopK(1)` is the single-source pointer, so `k ≥ 2`
  makes a two-token run reachable inside one step's copy mixture.
- **`set_pointer_gate_floor(0.9)`** — the plan's §2(b): `p_copy` is exactly 0 when no kept source holds
  the target and there is no floor, so flooring the gate lets `p_copy` win.

`crates/uor-r4-training/src/bin/numeric-pointer-trace.rs` gained `select=` and `gate_floor=`, applied
after load. **No weight was changed and no training was run.**

## The two arms

| | **ARM A — control (default knobs)** | **ARM B — TopK(2), gate floor 0.9** |
|---|---|---|
| **both stored digits selected** | **0 of 13** | **3 of 13** |
| class split | **EMITTER 2, PARTIAL 1, READER 10** | **DELIVERED 3, PARTIAL 6, READER 4** |
| byte-for-byte vs the sealed replies | **13 of 13** ✔ | **0 of 13** |
| `digit_ids` exactly two tokens (hard assertion) | held | held |

**Arm A is the instrument's validity control and it holds: the same binary, unchanged, still
reproduces the sealed replies 13 of 13 byte for byte.** Arm B's `0 of 13` is expected and is not a
defect: the knobs change decoding by construction, so the treatment arm cannot and should not
reproduce replies produced without them. **The instrument's own conditions are satisfied in the arm
where they are meaningful.**

## The pre-declared condition: FAILED

| condition | pre-declared | measured | |
|---|---|---|---|
| all 13 rows show both stored digits selected | 13 of 13 | **3 of 13** | **FAIL** |
| `mem-040` still emits the stored `84` | yes | **yes** — emitted ids contain `26, 22` in order, the tokens for `8`,`4` | pass |
| `mem-008` and `mem-024` no longer emit `19` | yes | **yes, both** — see the separate effect below | pass |
| instrument conditions hold | yes | control arm 13/13 byte-for-byte; assertion held | pass |

**The primary condition fails: the change moves the mechanism from 0 to 3 of 13 and does not reach
13.** This is reported as a miss, not averaged into a story: **the setting is insufficient on its
own.**

## The falsification: did it fire?

**The pre-declared falsification was: if both digits are selected and the reply STILL lacks the value,
the fix is in the wrong place and the emitter is the real problem.** **It did not fire in that form.**
Where both digits were selected — the **3 DELIVERED rows** — the value **was** emitted. **So on these
13 rows the failure is coverage, not emission: selection and emission agree wherever selection
happens.** That is the one thing this run adds to the diagnosis, and it is a real addition.

It does **not** license widening the same change: the pass condition failed, and per the plan's own
terms the next step is **not** a third knob.

## The EMITTER rows: a SEPARATE effect, with its own evidence

`mem-008` (stored `68`) and `mem-024` (stored `54`) both emitted `19` in arm A. **In arm B neither
emits `19`.** That is the **gate knob acting on the mechanism that implicated it** — a
**selected-then-dropped** failure — and it is reported **separately from the reader fix's success**,
which it does not constitute:

- `mem-008` arm B emits `[223, 27, 26, 2605, 2605, 2605]` — the distractor's `9` (`27`) and the
  stored `8` (`26`), **neither of the stored value's two digits as a run**;
- `mem-024` arm B emits `[223, 23, 22, 2605, 2605, 2605]` — `23, 22` in order, the **stored `54`**.

So the emitter rows still have their own piece and their own pass condition, exactly as the plan said.

## What this does not settle

- **Arm B's replies are a different distribution and their quality was not measured.** The gate floor
  of 0.9 forces copying at 90 % weight and the emitted id sequences show heavy repetition
  (`2605` repeated). **No judge was run**, so this piece says nothing about whether the change would
  help or hurt `check_pass` — and it must not be read as a capability improvement.
- It classifies these 13 rows on this artifact only.
- It says nothing about the reply half of criterion 1.
- **This artifact has no memory reader**, so nothing here transfers to an addressed-memory path.

## The ledger

| reading | status |
|---|---|
| Token count explains the cluster | **REFUTED** |
| Minimal pairs / digit order | **REFUTED** |
| Value addressability | **REFUTED** |
| The pointer's single-source shape is the binding constraint | **NOT REFUTED, NOT CONFIRMED — the knobs move it 0 → 3 of 13 and do not reach the pre-declared 13 of 13.** The shape is *a* constraint; at this setting it is not the whole one |
| The learned read/emit path | **SPLIT (arm A: READER 10, EMITTER 2, PARTIAL 1)**, and where selection succeeds, emission succeeds |

**Criterion 1 remains NOT MET on both halves and 43/232 is unchanged.** v5 was not re-run.

## Next

**Do not widen this change and do not try a third knob.** The measured state is: selection coverage
rises from 0 to 3 of 13 under a strong gate floor, and where both digits are selected the value is
emitted. The next question is therefore **why coverage stops at 3** — whether the pointer's kept
sources reach the value's second digit at all on the other 10 rows — which the trace can answer
directly by reporting **which window positions the kept sources occupy** at the step where the value
should be emitted. That is a read, not a knob. **A training-path change stays with the owner** and
still requires a timed calibration run before any estimate.
