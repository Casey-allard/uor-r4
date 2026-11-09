# REPORT: one-model-end-to-end, claude-338

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Code head `f9ea2eccdccfce09669a3c7e255f7892f5f0dcad`, on base `f75725d3071abb9488e058ac158a04d8d093014d`.

**In one line:** the reply-panel scorer can now score an artifact with ask heads and a question carry. `--engine integer` decodes through the integer session, and the default CPU-port path refuses such an artifact by name instead of scoring it as if it had none. No model was trained: the language run is claude-339.

## What did not work, first

- **The first real run stopped at argument parsing.** The scorer keeps a fixed list of flags, and `--engine` was not on it. The unit tests call the decode functions directly, so they never went through `run()`'s parsing. Fixed in `f9ea2eccd`, and the refusal test now parses `--engine integer` as given; a mutation that drops the flag makes it fail.
- **The integer engine is slower on the laptop:** 561 reply tokens a second in decode against the CPU port's 1,326 on the same 256 rows (about 2.4×). Both are this scorer's single-threaded host loops, not a served-speed measurement. The panel takes under a second either way.
- **The integer engine refuses `--teacher-forced` (and so `--paired-first`).** The rank paths still build their own CPU-port forward from the artifact. On an asking artifact, teacher-forced ranks are therefore not available yet; the CPU path refuses them there too.
- **No asking artifact was scored for real.** None exists yet: claude-339's run makes the first. The ask path is shown on a synthetic artifact (below).
- **Deviation from the ORDER, named:** parity is "identical after removing the four timing fields and `settings.engine`", not byte-identical. The report holds wall-clock fields that differ run to run. The ORDER's text was corrected in this unit.

## What changed

All in `docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`:

- **`--engine cpu|integer`** (default `cpu`). `integer` decodes each row through a fresh `IntegerStackModel::session()` with `step_native`, which serves the Hamming reads with their ask heads and the readout's carry (claude-336). Both engines sit behind one `Decoder` trait (`context_length`, `mark_reply_start`, `step_nearest`), so `generate` and `prime_prompt` are the same code for both.
- **`refuse_engine`:**
  - The CPU port refuses an artifact whose Hamming image has any ask head (a carry needs an ask, so this covers the carry), naming `--engine integer`.
  - The integer engine refuses `--teacher-forced`.
- **The report:** an integer run adds `settings.engine = "integer"`. A CPU-port report is unchanged.
- **Tests:**
  - `each_engine_refuses_what_it_would_score_silently_wrong`
  - `the_integer_session_decodes_as_the_cpu_port_without_an_ask`: Hamming reads, the point-key memory with its reply table, and a prompt past the context.
  - `only_the_integer_session_reads_the_ask_heads`: the CPU port decodes the asking artifact exactly as the one without ask heads; the integer session decodes it differently.
  - `point_key_fixture` is split into `point_key_artifact` and a wrapper, for the parity test.

## Measurements

**Parity on claude-322's sealed seed-1 artifact** (`s1.uorlut`, `86e51d5e…`, no ask table), m2-fresh256, scorer binary `ceefbbc2…`:

| | CPU port (default) | integer session |
|---|---|---|
| rows / complete | 256 / 0 | 256 / 0 |
| EOS / cap stops | 109 / 147 | 109 / 147 |
| good pairs | 0 of 128 | 0 of 128 |
| decode tokens/s (laptop) | 1,326 | 561 |
| report without timing fields (sha256) | `5a27775c…` | `5a27775c…` |

- **The two engines give the same report** on all 256 rows once the four timing fields and `settings.engine` are removed.
- **The default path is unchanged:** claude-322's sealed `score/s1.json`, written by the old scorer (`be3d4cb0…`, #3828), strips to the same `5a27775c…`.

No GPU time and no training.

## What this does and does not show

- **It shows** that the scorer reads what an integer artifact serves. On an artifact without ask heads, both engines give the same replies, stops and totals. On an asking artifact, only the integer session decodes the ask heads; the CPU port can no longer score one silently.
- **It does not show** that the integer engine is right on a trained asking artifact. That rests on claude-336's match of the session against the float trace, and on claude-339's run.
- **It does not show the served model's score for an artifact with a content readout.** `step_native` returns before the content readout's vote, which `serve_step` (the browser path) applies. Both engines skip it, as the scorer did before this unit, so they agree with each other but not with `serve_step` on such an artifact (fresh review, finding 2).
- **It does not test turn memory under the integer engine.** The CPU port refuses an artifact with turn memory; the integer session accepts one, and no test here pins its reply-start moves against the panel (fresh review, finding 1).
- **It does not move the panel:** s1 still scores 0 complete and 0 good pairs, as in claude-322.

## Checks run at the head

- **Host, at `f9ea2eccd`:** `served-reply-panel` tests with `--features cpu-parity` and the pinned tokenizer: 23 passed, 0 failed. Clippy on the binary and its tests: no warnings in this file.
- **Mutations, each applied to the committed tree, its test failing, then restored** (`local/claude/claude-338/mutate-338.sh`):
  - the CPU-port refusal off;
  - the ask never seen in the Hamming image;
  - the teacher-forced refusal off;
  - `mark_reply_start` a no-op in the integer decoder;
  - the integer context misread by 64;
  - `--engine` dropped from the flag list.
- **The real parity run** above, and the old scorer's sealed report.
- **The in-unit fresh review**, round 1, one reviewer given the spec and the diff only: APPROVE, with two low findings (the content readout and turn memory above). Both are named in the module doc and here rather than refused; no second round for a doc-only delta.

## Skills used

- None beyond the loop itself: the ORDER, the host tests with their mutations, the real parity run and the fresh exact-head review.

## Still open

- **claude-339, the language run.** Ask heads, carry 4 and the found-copy pull (claude-337), on claude-322's d3mwcap6 configuration, two seeds, against a no-carry control (L-010) and the `−C` cut, scored with `served-reply-panel --engine integer`. It needs its own prep sheet, FIT and pre-run review, and must answer claude-335's seed-1 failure of the carry with the pull at the scored rows only.
- **Teacher-forced and paired-first ranks on the integer session.** They stay on the CPU port, so they are refused for an asking artifact. Moving `teacher_forced_nearest_ranks` and `first_position_ranks` onto the `Decoder` trait needs each step's full nearest ranking from the session, which `NativeStep` does not carry today.
- **Content readout and turn memory under `--engine integer`:** refuse them, or route the decode through `serve_step` and test turn memory, before scoring such an artifact.
- **The CPU port's Hamming reads** still ignore the ask and the carry (DeepSeek's lane, sent on #2028 by claude-336).

Closes nothing: #3407 stays open.

## Plain summary

The program that grades a model's replies ran the model through a second, CPU copy of its maths. That copy has no question reader and no carry, the two parts the next training run is meant to teach. So it would have graded the new model as if those parts were missing, and given no warning.

Now the grader can run the model through the same integer code that serves it, which has both parts, and the old path refuses such a model by name. On the last trained model, which has neither part, both paths give the same grade on all 256 test conversations, and the same grade the old grader sealed. No GPU was used.

## Records

- Local: `local/claude/claude-338/parity/` (both reports, the stripped forms, `SHA256SUMS`, the scorer's hash), `mutations-r2.txt`, `host-tests-r2.txt`.
- Notebook: one row in `docs/research/notebook/float-training.md`.
