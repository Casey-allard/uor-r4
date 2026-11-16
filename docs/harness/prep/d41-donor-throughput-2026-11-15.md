# Prep sheet: the donor throughput probe on wick (m2-donor-corpus-generation, claude-398, 15 Nov 2026)

Lab: claude. Item: [m2-donor-corpus-generation](../../roadmap/items/m2-donor-corpus-generation.md) (#2848). Order: [claude-398](../../loop/tasks/m2-donor-corpus-generation/claude-398/ORDER.md). Code: branch `claude/claude-398-donor-throughput`, PR #4025. Release cited at dispatch: D86 (§3.10).

TEST FITNESS: FIT - round 2 (fresh-context non-author, at `25f13c283`) found no blocker; its two should-fix findings and four notes are folded in below.
- Round 2 (at `25f13c283`): FIT. (1) Busy slots and GPU utilization now average only the samples inside the 300 s window, so an arm's drain does not lower them. (2) "Under-occupied" now applies only to A4, A5 and A9. A6 and A7 wait for `n` free slots by design, so a loss there is reported as "lost at 16 workers" with its mean busy slots. Notes: (3) the window cuts off about half a prompt per worker, which counts more against the slot arms (about 2% at A0, about 8% at A3), so the reading names this bias and gives each slot arm's generated-token window rate beside it; (4) the job compares `smoke-temp` with the earlier `smoke-seq` as evidence for A8a's "same distribution", and an A8a win points to the length of the CPU chain, not to the GPU; (5) A9 never sizes the plan-keeping number; (6) the summary survives a malformed `metrics.json`.
- Round 1 (fresh-context non-author, at `f53631e76`): NOT FIT. One blocker and five should-fix findings, all folded in: (1, blocker) A6 and A7 ran fewer sequences than A0, so they are oversubscribed at 16 workers, busy slots are polled from `--metrics`, and an under-occupied arm is not read as a negative; (2) the bar uses a window rate that leaves out the in-flight tail; (3) a failing `count` or empty smoke no longer stops the job; (4) a server that never answers `/health` is `server-failed`; (5) C takes only levers that co-exist and keep the plan; (6) production is sized only from an arm that won. Its notes are kept: `smoke-n4` gave 802 generated tokens against `smoke-seq`'s 789, so `usage.completion_tokens` counts every choice; the backend-sampler check is now automated; arm A9 was added after round 1.

## The question

How many kept-answer tokens per second can `openai/gpt-oss-20b` give on wick, and with which settings? The pilot (claude-390) measured 220.7 kept and 503.6 generated tokens/s at one setting, so 100M kept tokens would take about 126 hours. The production pass `distill-d1` is sized from this probe's best measured rate. Each lever is a hypothesis, measured on its own against the pilot's setting.

The pilot's server log suggests a cause. With 32 slots busy, a decode step took about 55 ms and the GPU averaged 47%. Its requests send `top_k 0`, so the engine's CPU sampler handles the full 201k-token vocabulary for every slot at every step. Arms A8a and A8b test this. `b12000` runs on the GPU only the leading samplers of the chain that have a GPU version, and the default chain starts `penalties;dry;…`, where `dry` has none. So `--backend-sampling` alone changes nothing; the first smoke confirmed this, giving shards byte-identical to the CPU run. At the request's values (`top_k 0`, `top_p 1`, `min_p 0`, no penalties, DRY or XTC), every sampler but `temperature` is a no-op. So `--samplers temperature` samples the same distribution, and with it `--backend-sampling` can run the whole chain on the GPU (greedy at temperature 0, `dist` at 1).

## Steps

1. **Smoke** (instrument check, not corpus): `wick-throughput.sh throughput/arms-smoke.txt smoke <commit>` runs the head `generate` on the set's first 4 prompts at `--parallel 1`, once per mode (`seq`, `n4`, `g-n3`), once with `--tokenize 0` then `count`, once with the short system message, and once with `--backend-sampling`. Each smoke must keep ≥ 3 records, with 0 markup answers, 0 errors and 16 answers returned. This is the first time the head client's integer sampling request reaches `llama-server`. **Done at `9d7c62536`: all six passed** (16 answers each, 0 markup, 0 errors, 0 tokenize errors; `count` gave 514 kept tokens for the `--tokenize 0` smoke). Two more smokes run inside the probe job, before A8a and A8b, with the trimmed chain on the CPU and on the GPU.
2. **Probe** (`throughput/arms-probe.txt`): one job, eleven timed arms and A0's repeat. Each is a 300 s dispatch window from the start of the pilot's prompt set, and the prompts in flight then finish. **The rate the bar uses is the window rate:** tokens from requests, and kept-answer tokens from prompts, that finished within the 300 s, divided by 300 s. The in-flight tail after the window (`tail_ms`) is recorded and never enters it. For `--tokenize 0`, `count` tokenizes the same window's records afterwards, from `window.json`. A0 runs first and again last, so its spread is measured. Every arm uses `--max-tokens 1024`, seed 20261115 and the pilot's sampling values. The server runs with `--metrics`, and every 5 s the job records GPU memory, GPU utilization and busy slots (`llamacpp:requests_processing`).

   | Arm | Server (`llama-server -ngl 99 -fa on --jinja --metrics` plus) | Client | Lever |
   |---|---|---|---|
   | A0, A0-repeat | `-np 32 -c 131072` | `seq`, 32 workers | the pilot's settings, re-measured with the head client |
   | A1 | `-np 64 -c 131072` | `seq`, 64 | more slots |
   | A2 | `-np 64 -c 131072 -b 4096 -ub 1024` | `seq`, 64 | more slots, larger batches |
   | A3 | `-np 128 -c 262144 -b 8192 -ub 2048` | `seq`, 128 | most slots, largest batches |
   | A4 | A0's | `seq`, 32, `--system-file short-system.txt` | a shorter system message |
   | A5 | A0's | `seq`, 32, `--tokenize 0`, then `count` | no per-answer `/tokenize` |
   | A6 | A0's | `n4`, 16 (oversubscribed: 64 asked, 32 slots) | four answers in one request (all sampled) |
   | A7 | A0's | `g-n3`, 16 (oversubscribed) | one greedy request, then `n` = 3 (the pilot's seed rule) |
   | A9 | A0's | `final`, 32 | the analysis channel skipped: `/apply-template` renders the prompt, the client checks it ends at `<\|start\|>assistant`, appends `<\|channel\|>final<\|message\|>` and sends the pilot's four requests raw to `/completion` (a smoke, `smoke-final`, runs first) |
   | A8a | A0's plus `--samplers temperature` | `seq`, 32 | the same distribution through a shorter CPU chain (diagnostic) |
   | A8b | A0's plus `--samplers temperature --backend-sampling` | `seq`, 32 | the same chain on the GPU (diagnostic) |

3. **Combined arm C**, a second job, run twice. C takes only levers that can co-exist and keep the corpus plan: the fastest winning server setting (from A1–A3, A8a, A8b), plus A5 and A7 if they won. Levers that change the plan (A4, A6, A9) never enter C; they are reported on their own.
4. **Optional V (vLLM):** the same client against vLLM's OpenAI server on the donor's released safetensors, if it installs in Claude's container within 30 minutes. vLLM is Python and is named in the receipt with its version. If it does not install or serve gpt-oss on `sm_89`, V is reported as not run.
5. **Storage:** each arm's `metrics.json`, `run.json`, `window.json`, `count.json`, server and GPU logs, `backend-active.txt` and `summary.tsv` go to the HF store under `results/claude/d41-throughput-2026-11-15/`, fetched back and checked by sha256. Answer texts go there too, marked as probe output: not corpus, never training data.

## Bar

The probe is a measurement. Its bar decides which numbers may size production. Every rate below is the window rate of step 2.
- **B1 smoke:** all six smokes pass, as in step 1, and the probe job's three smokes (`smoke-final`, `smoke-temp`, `smoke-temp-bs`) pass the same check.
- **B2 arm quality:** an arm counts only if its truncated plus markup answers are ≤ 5% of answers returned, its request errors are ≤ 1% of prompts dispatched, and its tokenize errors (in `metrics.json` or `count.json`) are 0.
- **B3 baseline:** A0's kept-answer rate lies within ±15% of the pilot's 220.7 tok/s, and A0 and A0-repeat lie within 10% of each other. This shows the head client and a 300 s window measure what the pilot measured.

**A lever wins** when its arm passes B2 and its kept-answer rate is at least 1.10 × A0's and above the larger of A0 and A0-repeat by more than their difference.

**Under-occupied, not lost:** A4, A5 or A9, whose mean busy slots inside the window fall below A0's by more than 10%, has not tested its lever at A0's load. If it does not win, it is read as "under-occupied" and not as a negative. A6 and A7 are excepted: an `n` request waits for `n` free slots, so empty slots are part of that lever, and a loss there is reported as "lost at 16 workers, mean busy slots X".

**The window's cut-off bias:** about half a prompt per worker is in flight when the window closes and is left out of the kept rate. This costs about 2% at A0 and about 8% at A3, whose prompts are longer, so a slot arm needs a little more real gain to win. The REPORT names this bias, and gives each slot arm's generated-token window rate (per request, less affected) beside its kept rate.

**Recorded, with no bar:** generated and kept tokens/s over the window and over the whole wall time, `tail_ms`, kept tokens per 1,000 generated, request p50/p95, prompts dispatched, mean GPU utilization, peak memory and mean busy slots.

**Reading map** (fixed before the run):
- **P:** B1–B3 hold. The REPORT sizes production as hours = 100,000,000 / (kept tok/s) / 3,600, from the best arm or C run that **won**, and from the mean of A0 and A0-repeat if none won. It gives two numbers: the best that keeps the corpus plan (the pilot's system message, one greedy and three sampled answers, the analysis channel left in), and the best overall. A6 has no greedy answer, A4 changes the prompt and A9 skips the analysis channel, so each changes the plan, and production's own sheet would have to adopt that change. C's two runs are both sizing numbers: the lower one is used. A9 may name the best overall number, never the plan-keeping one.
- **I (instrument):** B1 or B3 fails. The client or the window is repaired. No number sizes production until a new sheet's run passes.
- **Q:** an arm fails B2. That arm is reported and not used for sizing.
- **A9 (the final-channel pre-fill), reported on its own, beside the slot and batch arms and never merged into C:** the pilot kept 43.8% of generated tokens because gpt-oss writes its analysis channel even at `reasoning_effort` low. A9 records its kept tok/s, its kept tokens per 1,000 generated, and its truncated and markup rates separately. Skipping the analysis channel can change the answers, so A9 changes the corpus plan, and a win means only that production's own sheet may consider it, with an answer check of its own.
- **H-sampler (diagnostic):** if A8a or A8b wins by ≥ 1.5× with higher GPU utilization, the CPU sampler was the pilot's limit. The job writes both checks to `backend-active.txt`: the count of "setting backend sampler" lines in the server log, and whether the `smoke-temp` and `smoke-temp-bs` shards differ. A8b counts only if the count is above 0 and the shards differ; otherwise it is read as "not active", not as a negative. The job also compares `smoke-temp` with the earlier smoke job's `smoke-seq` (default chain): identical shards are evidence that the trimmed chain samples the same distribution, though different shards do not disprove it. An A8a win points to the length of the CPU sampler chain, not to the GPU. Backend sampling is experimental in `b12000`, so production's sheet must check it before adopting it: on the same prompts at `--parallel 1`, its greedy answers must equal the CPU sampler's.

## Base artifact

- **Donor:** `openai/gpt-oss-20b` revision `6cee5e81ee83917806bbde320786a8fb61efebee`, the pilot's GGUF (`gpt-oss-20b.gguf`, its sha256 in the pilot's receipt).
- **Engine:** llama.cpp `b12000` (`68d20b884c3878217de79eade0a249efde96ab38`), the pilot's build, outside the repository (§3.4).
- **Instrument:** `docs/labs/d41-driver-math-pilot/instrument` at the commit each job's command line names; every `run.json` records it as `instrument_commit`. It is a client with no model code.
- **Machine:** wick, NVIDIA GeForce RTX 4090, UUID `GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41`, in Claude's container (`local/claude/wick/wick.sh`).

## Data split

- Every prompt comes from the pilot's seeded, panel-guarded prompt set (sha256 `6a6630b2eb0078c3c795b04e005da47c810113cf41d15888c7632bb96d37cca0`), checked by the script before any arm. None comes from TRAIN, DEV or a panel.
- The probe's answers are probe output only. They are not part of `distill-d1`, not sealed and never used to train.

## Minimum test power

- No mechanism claim, so no seed requirement. One seed, 20261115.
- At the pilot's rate, a 300 s window holds about 3,000 requests, and each rate is reported with its request p50 and p95.
- Run-to-run spread is measured by A0 and A0-repeat. A lever must beat that spread to win.
- Every arm starts at the same prompt, so the arms see the same prompts in the same order, as far as each one gets.

## Cost

Wick is the owner's own GPU, so there is no rental cost (D86). The probe job takes about 12 × 5.5 minutes plus 7 server loads, about 75 minutes; the combined job about 15 minutes. No pod, and nothing near the §5.4 caps. `wick.sh gpu` is read before each job, and the job runs only if the GPU is free of other users' work.

## Stop rule

- Each arm stops dispatching after 300 s, and the prompts in flight finish.
- A server that does not start (for example, out of memory at `-np 128`) is recorded as `server-failed`, its arm is reported as not measured, and the job goes on to the next arm.
- Hard stop: if a job passes 2 hours, the lab stops it by its own PID and reports the arms finished.
- One pass per arm name. A failed arm is reported and is not rerun under this sheet.
