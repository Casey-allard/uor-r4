# ORDER: one-model-end-to-end, claude-356

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `de3a44c45aa1729ccb85dbf908f6ab9fe1ac810f`.

## Goal

**Serve the three place-read models with the longest-match copy rule everywhere except a reply's first token, on prompts no earlier reading used, and read whether their recall replies get better.**

**Why now.** [claude-355](../claude-355/REPORT.md) served `longest-exact` and read "not better". Its lost recall replies sat on the rows whose first reply token changed (21 of 26, 16 of 18, 17 of 20), where the rule's two tokens are the end of `Assistant` and `:`. On the rows whose opening did not change, recall replies rose (214 to 233, 244 to 268, 199 to 230). The counts pointed at the opening; no run left it alone. This unit is that run.

On 12 Nov 2026 (UTC), in chat, the lab asked "Run the laptop test of the rule that keeps the one-token match at a reply's first token?" and Casey answered **"Yes, test it"**.

## What runs

Laptop only: saved integer artifacts and a generated panel. Nothing is trained and no pod is launched.

| Step | What |
|---|---|
| 1 | `CopyRule::LongestAfterOpening` (`longest-after-opening`) in the integer reader: at the row that chooses a reply's first token (the row the session's reply mark names) the latest exact match; at every other row the `longest-exact` choice. With no reply mark it is `longest-exact`. Served only when a caller sets it. |
| 2 | A fresh panel from the generator that made the old one: `m-world-recall-corpus conversations=1000 seed=15` (the old panel is seed 14, TRAIN seed 13), its `recall=oracle` twin, the places kit by `train-in-space --turn-census --places-out`, and prompts at the 256 and 192 windows by `--panel-prompts`. The same programs first remake seed 14. |
| 3 | The four stage 3 artifacts of claude-352 (p-s1, p-s2, p-s3, control n-s2) read on the fresh and the old prompts under three rules: their own, `longest-exact`, `longest-after-opening`; 48 tokens a reply. |
| 4 | Counts as in claude-355 (recall replies that state the value, repeat-a-string replies equal to the wanted string, other replies equal to the wanted reply, replies at the cap), and the changed replies word for word. |
| 5 | One scripted conversation for p-s2 under the three rules. Its nine user turns, fixed here: "Keep these in mind: druvalk is 73, mibtorn is felsharp, quonzet is 18." / "What is mibtorn?" / "What does a cow say?" / "Say exactly this: trelvon dask pimmerholt zane" / "What is druvalk?" / "I'm tired today." / "Tell me what quonzet was." / "What is two plus three?" / "What is my name?" |

## Decisions of this unit (claude's own decision review; logged for Casey, who can reverse them)

1. **The reading is fixed here, before any reply to a fresh prompt exists under any rule.** It is read at the 256 window, on the fresh prompts that equal no prompt of the old panel, on each of the three place-read seeds, the new rule against the model's own: (a) at least 10 more recall replies state the value; (b) the repeat-a-string replies equal to the wanted string are not fewer; (c) the other replies equal to the wanted reply fall by no more than 10. All three on all three seeds reads **"better on recall"**. Anything else reads "not better", with the part and the seed named. The control, the old panel and `longest-exact` are reported and are no part of the reading.
2. **Part (b) is narrower than claude-355's ("more"), and it is chosen knowing that result.** The new rule equals `longest-exact` on every row but a reply's opening, and no repeat row's first token changed in claude-355, so this unit tests nothing new on repeat rows. claude-355's reading of them stands whatever is found here: a copied string seldom stops (2, 2 and 0 of 96 replies equal the string). "Better on recall" will not say that repeat replies are right.
3. **The rule and the margin of 10 were chosen on the old panel's replies.** That is why the reading is on fresh prompts, and why the old panel is description only.
4. **The fresh panel is accepted only if the same programs remake the old one:** seed 14's tokens with the old panel's SHA-256, its places kit and its 256 prompts byte for byte.
5. **Under either new rule the laptop copy reads differently from its trained model on purpose.** No float-against-integer agreement is claimed. Nothing served by default changes; the float side, every bar and status, and claude-352's NOT_RUN reading are untouched.
6. **No pod and no training.** A retrain with a rule waits for Casey's word.

## What a pass is

| # | Gate | Check |
|---|---|---|
| G1 | The rule's tests fail when its lines are broken, each form on its own, the session's passing of the reply mark among them; the integer crate's stack tests pass at the head. | the test runs |
| G2 | On the old prompts, the reply files with no rule set and under `longest-exact` are byte-identical to claude-355's. | `cmp` |
| G3 | Decision 4's three equalities. | SHA-256, `cmp` |
| G4 | The counts of step 4 for the four artifacts, both windows, both panels and three rules are in the REPORT with the changed replies word for word, and the REPORT says plainly how decision 1 reads. | the REPORT |

## Not established by this unit

- How a model trained with either rule would reply; none is trained.
- Stopping after a copied string, the value's first token, questions about things never told. No milestone moves.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a rule that already spares the opening, or this run already made.
- **Result (at claim):** none. `CopyRule` holds `mean`, `latest-exact`, `latest-nearest` and `longest-exact`; no copy rule reads the session's reply mark.

## Claim

Written by hand, as claude-347's to claude-355's were: claude-346 (PR #3913, a draft under the hold) holds the item for `uor-harness loop claim`.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-356/**`
- `crates/uor-r4-integer/src/stack/**`
- `crates/uor-r4-integer/src/bin/uor-chat.rs`
- `docs/labs/one-model-end-to-end/instrument/**`
- `docs/research/notebook/**` (one row a result)

## Inputs

- claude-352's kit (artifacts, old prompts, pinned `train-in-space` and scorer) and claude-355's reply files and kit, on the laptop: `local/claude/claude-352/`, `local/claude/claude-355/`.
