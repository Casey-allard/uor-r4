# REPORT: one-model-end-to-end, claude-324

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Commits: `dcb8fa74f` (ORDER), `df7741d7b` (generator), `9f49e43dc` (manifest fields, two-word minimum).

## What did not work, first

- **No model was trained or scored in this unit.** It builds TRAIN data only. Nothing here says whether the bare-reply corpus moves `complete`; that is the next unit's run.
- **The first corpus would have lost 2,545 of its 40,000 rows silently.** With values of one word allowed, a one-token value gives a reply of 3 scored targets (value, `.`, EOS). The trainer keeps a window only with at least 4 (`build_document_windows`). The minimum is now two words, like the panel's shortest literals. The manifest records the per-row census, and the store test pins `under_4 = 0`; it fails with a minimum of one.
- **`scale-probe mix` refused the corpus, then claude-322's mix, as a root.**
  - The corpus manifest lacked a top-level `bos_id` and the four provenance fields (`source`, `revision`, `licence`, `attribution`); the generator now writes them.
  - A mix's manifest has no `source`, so claude-322's mix cannot be a root (claude-322 hit the same with D3). The new mix is built from claude-322's ten roots again, plus the corpus.
- **Two builder subagents were refused every write** before the third built the crate. The guard binds a subagent to its suborder from the session's current folder, and the session was in the main checkout. Recorded in `local/claude/FRICTION.md`.
- **No notebook row, against the ORDER's done-when 3.** The notebook's rules (`docs/research/notebook/README.md:34`, and its five verdict words) give a tooling or data step no row, since it measures no model. The row comes with claude-325's run of this mix.
- **The ORDER's disjointness from "any word of the sealed dev512 inputs" was not checked.** The dev512 inputs file (sha `b9661606…`) is not in the repository; only its audit summaries are, under `archive/labs/m2-fresh-437-2026-10-10/`. The check covers what is on disk:
  - the `m2-fresh256` panel's own literals (`FRESH256_*`), the panel this item is read on;
  - `HISTORICAL_EXPOSED_LITERALS`;
  - the repository's loaded historical literals (74). That loader (`load_historical_exposed_literals_from_repo`, `lib.rs:1356-1381`) binds the opened-fresh128 audit's input and tokenizer shas, and checks the 4 literals of the 2 banks the audit lists in literal support.
  - **opened-fresh128 is only partly covered.** Its audit counts 12 frozen banks and 15 fresh full literals but holds no text for the other 10 banks or the 15, so nothing on disk checks them against this corpus.
- **The mix was built with `scale-probe mix`, not `mix-chat-corpus`, and no `mix-chat-corpus` round trip was run.** The store test reads back every field both tools check, and the trainer's own loader reads the mix in claude-325.
- **The first disjointness test caught only a trailing `s`, and the overlap test asserted `≥ 256` panel messages.** After the fresh review it also checks `-es` and `-y`/`-ies` both ways (no new clash), and it pins the count at exactly 320. Only tests changed; the corpus and mix bytes are as sealed.
- **One curriculum template reads badly:** "I am a {v} by trade." gives "I am a opera …" for vowel-initial values. It is noise in the user text, not in the scored reply.

## What changed

- **A generator,** `docs/labs/one-model-end-to-end/current-state-corpus/` (standalone, detached from the workspace like the panel's instrument). `current-state-corpus out=DIR rows=N seed=S` writes a `uor-r4-chat-corpus/v1` store through the repository's own `CorpusWriter` and dialogue encoder.
- **The episodes follow the panel's served layout** (`served-reply-panel`'s default `one-user-turn/v1`, `render` at `served-reply-panel.rs:431-475`): one user message with the statements and then the question joined by single spaces, and one assistant reply. There are no assistant acknowledgments, as in the panel.
  - **Even rows, the panel's shape:** one statement per relation (job and home), from an assert or an update template, in random order. The served panel prompt also keeps only each relation's latest statement (`lib.rs:1068-1110`).
  - **Odd rows, update histories:** 1 to 3 statements per relation, at least 3 in all, each relation's first an assertion and the rest updates, interleaved keeping each relation's order. These teach that the latest statement wins.
  - **The reply** is exactly the asked relation's current value and `.`, then EOS. The mask covers the reply and EOS only.
  - **Phrasings:** the panel's `supported_wire` templates and queries, plus `milestone_world.rs`'s train phrasings for job and home (never its development ones).
  - **Values:** 2 to 8 words from a 423-word list in the crate, distinct within an episode.
- **Tests (6, all pass):**
  - the value words share no word, singular or plural, with the panel's `FRESH256_*` literals, `HISTORICAL_EXPOSED_LITERALS` or the repository's loaded historical literals (13 real clashes were found and removed; adding `basalt` makes it fail);
  - the tokenizer is the pinned `d36d3e87…`;
  - none of the 320 panel user messages, in the served form or as a join of the history's turns, equals a generated one;
  - the form and mask over 4,000 rows: the served prompt is a token prefix of the document, and the mask starts there and stays 1 through EOS;
  - determinism;
  - the store reads back, with every field `mix-chat-corpus` and `scale-probe mix` check.

## Measurements

**The corpus** (seed 324), sealed at `results/claude/one-model-end-to-end-currentstate-2026-11-08/corpus/`:

| Rows | Tokens | Scored (reply + EOS) | Scored per row | Longest document | Panel prompts matched |
|---|---|---|---|---|---|
| 40,000 (20,000 per shape) | 3,323,879 | 474,325 | 4 to 32 (none under 4) | 188 | 0 of 320 |

Shas: tokens `85641489ddc9f3af966545667cb558215dcbf3f36049205a139ca3e2146a79c2`, mask `febada2f1ed021c8553607a80b5fc09729e234f4070fa9cdb48717c95448b443`.

A row (row 0, panel shape):

```
<|bos|>User: I am a bowl mall round teacup turtle snowy by trade. I just moved to shrimp donut hairbrush coffee. What is my job?
Assistant: bowl mall round teacup turtle snowy.<|eos|>
```

**The mix**, sealed at `results/claude/one-model-end-to-end-currentstate-2026-11-08/train/`:

- **Built by** `scale-probe mix fraction=1/1` (bin `31610a00…`, the one claude-322 used) over claude-322's ten roots in its order (D3's four from the store, then claude-321's k = 8 capped split six times), then the corpus as `currentstate`. The script is `local/claude/claude-324/mix.sh`.
- **The reproduction check:** the ten roots alone rebuild claude-322's mix byte for byte (tokens `0273173e…`, mask `da506e85…`, manifest `442ead2c…`).
- **Shas:** tokens `b34e72542273f3ac1b040f7ec5c940b07b132898d6862e6df5155b2b9962c191`, mask `a8aaa73bbd6c8ee6759cf6ea2a2e46a8eb620885274c1ba75f962d0535b1f50c`, manifest `5efff97971db6a05148f01153c626b4f7ef8177d084bee0da096a93542e6ec13`.
- **Totals:** 76,368,774 tokens, 418,016 documents, 33,839,081 scored.

| | claude-322's mix | + currentstate | Its share |
|---|---|---|---|
| Documents | 378,016 | 418,016 | 9.6% |
| Tokens | 73,044,895 | 76,368,774 | 4.4% |
| Scored targets | 33,364,756 | 33,839,081 | 1.4% |

- **Windows (expected):** claude-322's mix has 417,421. Each current-state document (at most 188 tokens) should add one window, so about 457,421, and about 8.7% of the windows a 16,384 × 64 run sees. The run's own log line is the count; the next sheet's I-3 checks it.
- **Store readback:** all six files read back from the store with equal sha256.

## Checks run at the head

- `cargo test --offline --manifest-path docs/labs/one-model-end-to-end/current-state-corpus/Cargo.toml`: 6 passed, 0 failed.
- Negative gates: with `basalt`, or the plural `basalts` (caught only by the round-2 stem forms), in the value list, the disjointness test fails; with a one-word minimum, the store test fails on `under_4`.
- The crate is outside the workspace, so the workspace's own builds and suites are unchanged.

## Decision (for the next unit)

**Next is claude-325:** (b64L) at claude-322's recipe, pin, seeds and steps on this mix, under its own prep sheet, FIT and pre-run review. It is read like claude-322: `m2-fresh256` served replies, the first-line count, `complete`, stops and collapse, and the held-out pair. Its comparison is claude-322, the same recipe without the corpus.

- **What a lift in `complete` could show:** the trunk binds a fresh value, stated in the chat, into the bare reply, for job and home.
- **What it could not show:** the same for other relations, or for a reply form that was never trained. The panel's templates are ones it calls supported; its freshness is in its values, and none of those values' words is in this corpus.
- **claude-323 (mixed copy keys) runs in parallel.** If it lifts recall, the keys and this corpus can be combined in a later sheet.

## Skills used

- None beyond the loop's own steps.

## Still open

- The vowel-initial "I am a …" template could take "an" (a cosmetic change to the user text; it would change the corpus bytes).
- `scale-probe mix` cannot take a mix as a root. Each mix is rebuilt from its roots, which works and is reproducible, but needs every root on the laptop.
- mw-balp's licence and generator are still not recorded (claude-314).

## Plain summary

The panel asks "What is my current job?" after the user has said it, and only the bare answer counts. Our model had never been trained on that kind of chat, and it answered with another question's sentence. This unit wrote a generator for 40,000 such chats, using made-up job and home values whose words appear in no test panel, with the bare answer as the reply. It added them to the last training mix and saved both to the store. The next unit trains on it and scores it.
