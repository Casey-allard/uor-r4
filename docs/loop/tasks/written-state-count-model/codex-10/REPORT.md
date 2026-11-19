# [codex] Crossed role and boundary decisions, native count replay and client integration

## What did not work, first

The learned value boundary regressed on the earlier fresh panel, from 979 to 850 correct sources per seed. On the new wording family it added zero final correct addresses beyond the role-only arm. The non-conditional selectors also proposed a source for every one of 48 absent or ambiguous requests. A released M1 seed-1 artifact invoked through the ordinary text CLI gave the same generic reply to two opposite-role requests and an unrelated question. The reference count codec preserved four of 3,989 observed unigram successors. Predictive scaling requires a full-support expert and fitted weights. [Selector](SELECTOR-RESULT.md) · [Count replay](COUNT-RESULT.md) · [Baseline client](CLIENT.md)

## What changed

The two main experiments followed the [closure plan](../codex-9/CLOSURE-PLAN.md). On Wick a fitted occurrence score and a fitted value-start score were varied independently over the same inputs. The old continuation, token-ID writer, cursor feedback and stopping rule were fixed. The additional action probe asks whether a selected source warrants copying. The discrete CLI now accepts one ordinary text turn, binds its tokenizer, frames the complete prompt including BOS and returns text and the actual stop token. Raw-ID parity mode continues to report its length-bounded trajectory.

P1(a) gained a device-written table in the D11 trunk, a canonical stream oracle and a reader that checks each retained lookup. Both client and training code use exact integer IDs and lossless artifact packing. [Count sheet](COUNT-SHEET.md) · [Count FIT](COUNT-FIT.md) · [Selector sheet](SELECTOR-SHEET.md) · [Selector FIT](SELECTOR-FIT.md) · [Writer sheet](WRITER-SHEET.md) · [Writer FIT](WRITER-FIT.md)

## Measurements

On 5,196 whole D87 TRAIN records, 20,475,957 native tokens, the CPU/GPU exact streams agreed at each order from zero to ten. The GPU and CPU pruners agreed on all 14,634,359 retained triples. Reading back the actual 379,223,660-byte D11 artifact recovered every retained count. The end-to-end run including the CPU oracle and lookup checks took 68.48 seconds. The Mac text client loaded this file and reproduced all ten original seed-1 emitted IDs on the registered focal question. The new S2 rows are available for lookup. The fitted prediction mixer remains open. [Measurements and artifact](COUNT-RESULT.md) · [Text runs](CLIENT-TURNS.json)

Across the 1,728 focal prompts per seed, the baseline, role-only, boundary-only and joint arms produced **1,509 / 1,610 / 1,524 / 1,628** exact replies through EOS. Role alone moved 105 misses to exact and four exacts to misses. First wrong at token 0 fell from 165 to 114 and at token 1 from 54 to four. On the unseen wording family, role moved 282 misses to exact and lost 66, lifting total exact from 1,190 to 1,406 and both-role exact from 332 to 580 of 864. All 322 remaining wrong addresses bound the excluded mention to its fact. Boundary added to role changed 134 proposed word starts here and **zero** final addresses. For 122 of those the existing continuation already corrected the earlier proposal. On the earlier fresh panel role scored 1,056, and adding boundary took that down to 928. The two seeds use the same linguistic examples. [Selector diagnosis](SELECTOR-RESULT.md) · [Writer outcomes and transition tables](WRITER-RESULT.md)

The saved role-only artifact reproduced the operator in the ordinary client. It selected different source positions for the two Tava/Tavi questions, delivered the requested IDs and EOS, and changed its answer to a fixed Ben question when the stated values swapped. Read-off restored the ordinary model's output. A missing-entity question still triggered a false read, and the candidate copied part of an unrelated question after matching repeated punctuation. Raw CUDA/client writer words agreed on all 148 checked steps. Keep this an experimental reader. [Client integration, artifacts and replay](CLIENT-INTEGRATION.md)

The action gate selected ordinary generation on all 146 unsupported held requests, including 48 absent, 48 ambiguous and 50 unrelated questions. It suppressed 122 actual role-only copies; the other 24 already lacked a source. Held positive exacts fell from 48/48 to 24/48. Earlier unseen-family exacts fell from 1,406 to 794/1,728, while focal exacts stayed at 1,610. The 24 fresh false vetoes share one unseen descriptor: on “Regarding [person], what did the entry name?”, the unconstrained nominee picked the word “name” (token 1412) and found no prior occurrence. Generate won strictly, at distance 224 versus 404 for copy. Ordinary generation on the snow and days-per-week questions produced acknowledgments that did not answer either. This configuration failed positive preservation. Retain the role-only reader as an experimental artifact. [Copy decision and collision census](action/RESULT.md) · [Actual-feedback measurements](action/writer/summaries)

## Checks run at the head

Compare the actual CUDA outputs with independent native replay and repeat feeds. Compare raw digests of the whole D11 file, the declared count stream and the original base payload in their respective domains. The first count staging run performed pruning on the host. The qualifying count result applies that rule on CUDA. Early client testing exposed an omitted BOS in the text prompt. Switching to the complete-prefix encoder matched the archived role-swap tapes exactly. [Count controls](count/device-finite.json) · [Selector finite cases](selector/finite.txt) · [Writer finite cases](writer/finite)

The sealed selector, writer, action and count results have separately verified identities. See the [count file](COUNT-RESULT.md), [selector archive](SELECTOR-RESULT.md), [crossed writer archive](WRITER-RESULT.md), [action selector archive](action/RESULT.md), [action writer archive](action/WRITER-RESULT.md), and [two packaged role-only clients](CLIENT-INTEGRATION.md). These units reuse the same benchmark histories across seeds.

The full 1% CUDA count and device freeze were repeated from a separate build with the final production reader. The artifact was byte-identical to the earlier file, and all eleven order receipts matched. [Final reader and device replay](count/final-loader-replay.txt)

## Skills used

UOR research architect and native space cycle. Independent FITs, immutable handoffs and reviewed device/reader comparisons accompanied the model and artifact work.

## Still open

The 24 false vetoes share one descriptor unseen in TRAIN, with no contradictory action labels for that descriptor on held data. The next role experiment should distinguish a requested entity from a nearby question word and an explicitly excluded occurrence, while learning evidence for a missing or multiply specified referent. Train counterfactual neighborhoods and hold entire paraphrase families for evaluation. The ordinary generator also needs a measured full-support prediction path. P1(b–d), the compact deployable count predictor and its held-out scoring, the buffered 2k context horizon, multi-turn state and M2–M8. The open count-ceiling work in [PR #4097](https://github.com/UOR-Foundation/uor-r4/pull/4097) uses DEV-tune as a diagnostic. Formal and milestone promotion still depend on the registered whole-model tests.

## Plain summary

Count training now reaches a loadable integer artifact whose retained rows replay exactly. Choosing the requested occurrence improved recall across wording families. The evidence gives weaker support to the new value boundary. Decision and generation behavior on positive and unsupported text are reported separately.
