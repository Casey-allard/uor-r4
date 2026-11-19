# [codex] Actual-feedback gate verdict

**The learned permission gate fails the registered retention bar.** On the sealed mixed panel, the original role-only selector copied 48/48 positive answers exactly through EOS. The gate allowed 24 and withheld 24. Every withheld positive changed from exact to wrong at output token 0. An oracle that always copied the annotated value recovered all 48, and the no-copy control recovered zero. The two seeds have the same class counts on the same prompts. [Sheet](SHEET.md) · [Mixed panel and FIT](PANEL-SHEET.md) · [Device and case summaries](writer/summaries/seed1.json)

| Class, mixed held | Rows | Actual copies, role-only | Actual copies, action | Ordinary generation, action | Exact reference, role → action |
|---|---:|---:|---:|---:|---:|
| Requested value exists | 48 | 48 | 24 | 24 | 48 → 24 |
| Person absent | 48 | 48 | 0 | 48 | No unique answer graded |
| Person ambiguous | 48 | 48 | 0 | 48 | No unique answer graded |
| Unrelated or ordinary question | 50 | 26 | 0 | 50 | 0 → 0 |

The gate sent every one of the 146 unsupported reads to ordinary generation. Across those rows, 122 of the role-only outputs had actually copied. The remaining 24 were already running ordinary generation. Thus the non-copy criterion passed, while the fresh positive drop of 50 percentage points exceeded the allowed five points. “Regarding Ben, what did the entry name?” after “Sue held blue. Ben once held gold.” illustrates a false veto: the expected source would emit gold (ID 2551) followed by EOS, but the native fallback began at ID 1155. The missing and ambiguous factual controls have no unique expected response. Their figures evaluate permission to copy.

| Previously sealed panel | Exact through EOS, role → action | Change in first wrong token |
|---|---:|---|
| Focal transfer | 1,610 → 1,610 | Four moved 1→0. |
| Renamed challenge | 1,598 → 1,598 | Four moved 1→0. |
| Earlier primary | 981 → 598 | 383 moved exact→0 and 111 moved 1→0. |
| Earlier fresh | 1,056 → 552 | 504 moved exact→0 and 20 moved 1→0. |
| Previously unopened family | 1,406 → 794 | 612 moved exact→0 and 104 moved 1→0. |
| Original 48 unsupported prompts | No unique answer graded | Role-only copied on all 48. Gate copied on zero. |

**What ordinary generation said.** The two no-fact prompts produced the same decoded results at both seeds. For “What color is fresh snow?” the model emitted “Got it, I'll remember all� that.” and EOS, against “white” and EOS. For “How many days are in a week?” it emitted “Got it, I'll remember that.” and EOS, against “seven” and EOS. The replacement character is the tokenizer's rendering of the emitted sequence. Each of the other 48 ordinary/prose rows asks one of two questions against controlled fact histories. The 50 rows cover four distinct questions in total and supplied zero literal exact replies. These samples assess a bounded set of fallback behaviors. [Literal decoded ordinary outputs](writer/summaries/seed1-ordinary.json) · [Seed 2](writer/summaries/seed2-ordinary.json)

Wick ran 20 arm/seed/panel files twice and compared all 96,670 native/serving transitions per pass. The observed primary and repeat passes took 79 seconds each. Every allowed trajectory, including full per-token state and release, matched always-role; every withheld trajectory matched the independently executed no-copy control. Only the gate switches between those paths. The mixed panel uses separate held wording, names, values and ordinary questions, with 388 distinct visible prefixes across TRAIN and held. Its 48 rows per main class reuse two phrasings and controlled variations, which limits transfer claims. [Replay checks](writer/grade.jq) · [Repeat digests](writer/repeat.sha256) · [Panel census](panel/CENSUS.json)

A fresh download of the [action-writer evidence archive](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/e26093069b1f5c01ef458777bca1208bed7c5073/results/codex/role-boundary-count-20261118/action-writer-wick.tar.gz) matched SHA-256 `6befd6e1cde59367c7b7fc79279d09a06cc8bf3acffbd1347cf93b377a8edf83`. It includes the fresh TRAIN/held data, actual gate decisions, role handoffs and base hooks, bound writer executable, all primary and repeat raw outputs, case grades and first-error matrices. Its referenced [crossed writer archive](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/746913b113e99a515fdd3273492ec4656aed07cd/results/codex/role-boundary-count-20261118/crossed-writer-wick.tar.gz) supplies original models, tokenizer and historical control trajectories.
