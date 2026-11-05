# REPORT: real-text-corpus, claude-293 (the corpus and its census)

Lab: claude. Order: [ORDER.md](ORDER.md). Item: [real-text-corpus](../../../../roadmap/items/real-text-corpus.md), done-when 2–5. Base: `43e2d3206`. PR #3738.

**Result: the real-text corpus is built, screened, counted and stored.**
- **Four roots** in the private store under `results/claude/real-text-corpus-2026-11-05/`, beside `raw/`:
  - **`oasst2/`:** OASST2 in full, 5,071 TRAIN conversations;
  - **`wiki-00000/`, `wiki-00020/`, `wiki-00040/`:** Wikipedia, 12,945 TRAIN articles across the three.
  - Together that is **27,968,113 TRAIN tokens** and 1,491,618 held out, with a `README.md`, a `SHA256SUMS` and a full read-back.
- **The census** against P-dev, p-long and p-long4k reads an exact prompt share of 0.0 on every split.
- **DEV and Fresh V2 share six generic phrases with OASST2,** each under four words ("Thank you!", "Hello!", …). The census reads 0.0351 and 0.0407 from them. The phrases are kept, and recorded below.
- **The frozen panels' user turns** are read back from their splits by a new mode, `prepare-real-text panel-turns`. Its turns equal the files p-long's and p-long4k's generator stored.

**Claim check.** This is unit 2, the last, of the real-text corpus. The decided order (#2028 6634218223) puts it before P5 on real text. Nothing that trains or serves changes: this unit makes the human-written data P5 trains and scores on, instead of the templated M-world, whose replies are 75–78% verbatim in TRAIN (claude-202).

## What did not work, first

- **The ORDER's size estimate for Wikipedia was three times too low,** so decision 4 did not survive.
  - It planned 15,000 kept articles a shard, at about 15M tokens each. Shard 00000's first 15,000 articles hold 43,283,854 TRAIN tokens. The shard opens on the oldest articles: counting TRAIN and held out, its first 15,000 average about 3,050 tokens and its first 900 about 9,340.
  - The record is kept in the store's `superseded/`.
  - **Shard 00000 was built three times:** at `max_docs=15000` (`superseded/`); at `max_docs=2000`, whose held-out split the memory probe read (88 articles, 700,665 tokens), with only the probe's JSON as its record; and at 900, kept.
- **The census costs 0.67 to 0.75 GB per million TRAIN tokens on real text, not 150 bytes a token.**
  - Peak footprint over TRAIN tokens: 0.67 for `oasst2/` (2.51 GB at 3.77M); 0.70, 0.74 and 0.75 for the Wikipedia roots (5.64–6.08 GB at 7.9–8.3M).
  - The probe peaked at 0.61 GB on 0.70M tokens. It read an earlier build of `wiki-00000/heldout` (88 articles, tokens `3b5869ca…`, in its JSON), since rebuilt at 396,081 tokens.
  - The 150 bytes came from claude-202's run on the templated M-world. The census's M1/M2 reply-memory probe keeps a row for every reply target (`reply_rows`, then six `reply_table`s, `in_space_long.rs:4961-4993`). On templated data those collapse; on real text almost every row is distinct.
  - At 43.3M tokens the census would need about 29 to 32 GB; the laptop has 16 GB, about 8 GB of it free.
  - So the ORDER's stop rule applied ("shrink `max_docs`, and record it"). Each Wikipedia root was cut to about 8M TRAIN tokens: `max_docs` 900, 8,400 and 4,350, because article length differs by shard.
  - Wikipedia is 24.2M TRAIN tokens, not the planned ~45M. The corpus is 3.7 times today's 7.5M-token M-world corpus, not 7 times.
- **The exact prompt share is not 0.0 for DEV and Fresh V2 against `oasst2/`.**
  - The census reads 0.0351 for DEV (43 of 1,226 prompts) and 0.0407 for Fresh V2 (52 of 1,279).
  - Reading the turns back names every one of them: six phrases, all under the screen's four-word floor.
  - They are held by 33 of OASST2's 5,071 TRAIN threads. My decision is below; the phrases are kept.
- **For Wikipedia, the census's prompt checks measure nothing.**
  - Every token after `<|bos|>` has mask 1, so a Wikipedia split has no prompt runs. Its exact and n-gram prompt shares are 0.0 by construction.
  - Whether a panel's prompt appears inside article text is screened only by the builder's exclusion, which dropped 5 articles. No census counts panel prompt n-grams against TRAIN reply text.
- **Panel replies verbatim in OASST2's replies.** These are recorded, not inspected, since the bar is on prompts:
  - DEV 18 (0.0147), Fresh V2 28 (0.0219), P-dev 15 (0.0104), p-long 63 (0.0074), p-long4k 141 (0.0082).
  - The census gives only shares. Naming these replies needs a reader of assistant turns, which `panel-turns` is not.
- **Two held-out OASST2 prompts are also in TRAIN:** "Thank you" and "Can you write stories?", in different trees (held-out documents 32 and 257). The split is by tree, and people repeat openers.
- **Done-when 4 asked the REPORT to list every uncovered short turn; it lists them by count.**
  - There are 3,219, and 3,218 of them are the five frozen panels' turns, the sealed Fresh V2's among them.
  - That panel text stays in the private store. Each manifest's `exclusion.uncovered` holds the full list, under `SHA256SUMS`. The counts are under Measurements.
- **The in-unit fresh review found four items in the code and two in this REPORT.** Each is fixed at the head:
  - **The bin test never covered a skipped document,** so the ids' document index was not pinned: with `enumerate` moved after the filter, every test still passed. A new test puts an article first, and NG4 below shows it failing under that change.
  - **Two comments in `real_text.rs` were wrong.** One claimed the bin compares turns on real data; the other gave a wrong reason for the `<|unk|>` refusal.
  - **`panel-turns` hashed the split after writing `out`,** so a hashing failure left a file behind. It now hashes first.
  - **This REPORT quoted the census at 0.62 GB per million tokens and 27 GB for 43.3M,** a slope through the probe that understates the measured 0.67–0.75. It also gave the probe's 0.70M tokens without saying that the folder was since rebuilt.
  - **The second round** found the first rewording of the `<|unk|>` comment still wrong, and four record gaps: the harness checks were not listed, no file records the build commit, the `max_docs=2000` build went unrecorded, and an "8,800 tokens" figure had no stored source. Each is fixed or stated below.
- **The ORDER's cancel probe was stamped "about 16:20Z";** it ran before the ORDER commit at 16:00Z. Corrected in the ORDER.
- **Small slips, each caught at once:**
  - The first push failed with a transient error; the retry pushed.
  - The loop guard refused one command whose write target was a shell variable; I re-ran it with literal paths.
  - `uor-store put` printed "0/0 uploaded" for an upload of 62 files that did happen. The read-back, not that line, establishes storage.
- **Main's roadmap check is red for a reason outside this unit:** `infra-desktop-client-tauri.md` has `status: in_progress`, and `issues.json` names it. So this item's render is owed, as the ORDER said.

**Decision: the six shared phrases are kept** (ORDER stop-and-decide 3; mine, easy to reverse):
- **They are under the screen's four-word floor,** which claude-292's merged rule counts as uncovered rather than excluding, with a test.
- **They carry nothing panel-specific:** no invented name or word, no fact.
- **The panels that test memory** (P-dev, p-long, p-long4k) read 0.0 exactly.
- **Excluding them would cut real conversations** for a "Thanks!", and greetings and closings are what a chatbot most needs.
- **For any scorer on DEV or Fresh V2:** report apart the rows whose user turn is one of the six (53 DEV turns and 66 Fresh V2 turns by string match).
  - The census's 43 and 52 are fewer, because it matches whole runs with their position: a later turn's run opens on `\n`.
- **The 33 threads are named by index in the store's README,** so P5 can drop them if its sheet decides to.

## What changed

- **`crates/uor-r4-training/src/real_text.rs`:** `split_user_turns`.
  - It returns each document's user turns from a dialogue split's tokens and mask, for protocols 1 and 2.
  - It reads the mask-0 runs and strips the role markers. It skips system turns, `<|bos|>` and a prompt-side `<|eos|>`.
  - It refuses three things as a defect: a run that opens on no role marker, an `<|unk|>` in a prompt run, and a token and mask of unequal length.
  - Five tests cover it: a round trip under both protocols, with a system turn, a two-line turn and a document that ends on a user turn; then one test per refusal.
- **`crates/uor-r4-training/src/bin/prepare-real-text.rs`:** the `panel-turns` mode, `panel-turns split=DIR tokenizer=FILE name=NAME out=NEW_FILE`.
  - It checks the tokenizer pin first and refuses an existing `out`.
  - It writes `[{id: "NAME:<doc index>", user_turns}]`, skipping a document with no user turn, and prints the counts.
  - Argument parsing is shared with the build modes (`key_values`).
  - One test runs a two-tree build, reads its turns back, has `parse_panel` read them, and checks both refusals.
  - A second writes a split whose first document is an article, and checks the ids `t:1` and `t:2` and every printed count.
- **`docs/research/notebook/data-tokenizer.md`:** the line's first result row.
- **`docs/roadmap/items/real-text-corpus.md`:** `status: done`.
- **`docs/loop/tasks/real-text-corpus/claude-293/`:** the ORDER, its probe time corrected, and this REPORT.
- **Data, in the store, not committed:**
  - the four roots;
  - `panel-turns/`, the five frozen panels' user turns;
  - `census/`, one JSON per TRAIN split with its `/usr/bin/time -l` stderr, the OASST2 turn reads and the memory probe;
  - `logs/`, `superseded/`, `README.md` and `SHA256SUMS`, with 61 files under the sums.

## Measurements

All on the laptop, with the release binaries at `9911a2e1c`.

**The panels' user turns, read back** by `panel-turns`:

| Panel | Documents | User turns | Tokens SHA-256 | Against its stored turns |
|---|---|---|---|---|
| DEV (`ft/dev`) | 400 | 1,226 | `0919e90f…` | none stored |
| Fresh V2 | 400 | 1,279 | `0b7e754c…` | none stored |
| P-dev | 400 | 1,443 | `790b7e12…` | none stored |
| p-long | 160 | 8,474 | `8b2529b1…` | **equal**: `[.[].user_turns]` SHA-256 `4e5c524b…` on both |
| p-long4k | 160 | 17,180 | `ffefe084…` | **equal**: `21f52874…` on both |

**The corpus.** Every root uses the pinned tokenizer `d36d3e87…`, protocol 2, `heldout_per_mille=50`, `max_tokens=1048576` and the nine panels.

| Root | `max_docs` | Considered | TRAIN docs | TRAIN tokens | Held-out docs | Held-out tokens | Excluded (8-gram, short) | Dropped | Bytes per token |
|---|---|---|---|---|---|---|---|---|---|
| `oasst2/` | none | 5,400 | 5,071 | 3,773,419 | 278 | 214,758 | 51 (3, 48) | 0 | 2.612 |
| `wiki-00000/` | 900 | 901 | 864 | 8,011,538 | 36 | 396,081 | 1 (0, 1) | 0 | 2.491 |
| `wiki-00020/` | 8,400 | 8,402 | 7,970 | 8,255,877 | 430 | 443,682 | 2 (0, 2) | 0 | 2.351 |
| `wiki-00040/` | 4,350 | 4,352 | 4,111 | 7,927,279 | 239 | 437,097 | 2 (0, 2) | 0 | 2.427 |

- **OASST2's accounting:**
  - 13,854 trees read;
  - roots dropped: 8,451 not English, 2 deleted;
  - 1 tree with no assistant reply;
  - 5,400 threads considered. That is 5,349 kept plus 51 excluded.
  - Child messages dropped: 953 deleted, 616 not passed by review, 4 not English, 0 synthetic.
- **claude-292's smoke** screened against four panels: 5,391 kept, 9 excluded, of the same 5,400.
- **The build took 2.98 s** for OASST2, and 12.15 s for Wikipedia shard 00000 at 15,000 articles.

**The census** (`train-in-space --census 1`): the exact prompt share, panel by TRAIN split.

| Panel (prompts) | `oasst2/` | `wiki-00000/` | `wiki-00020/` | `wiki-00040/` |
|---|---|---|---|---|
| DEV (1,226) | **0.0351** | 0.0 | 0.0 | 0.0 |
| Fresh V2 (1,279) | **0.0407** | 0.0 | 0.0 | 0.0 |
| P-dev (1,443) | 0.0 | 0.0 | 0.0 | 0.0 |
| p-long (8,474) | 0.0 | 0.0 | 0.0 | 0.0 |
| p-long4k (17,180) | 0.0 | 0.0 | 0.0 | 0.0 |
| Its own `heldout/` | 0.0039 (2 of 511) | no prompts | no prompts | no prompts |

- **The 8-word prompt n-gram share against `oasst2/`:** DEV 0.078, Fresh V2 0.078, P-dev 0.055, p-long 0.051, p-long4k 0.051.
- **The 13-word share:** 0.0062, 0.0057, 0.00018, 0.000026 and 0.000087.
- **The six shared phrases** (DEV / Fresh V2 user turns, by string match):

  | Phrase | DEV | Fresh V2 |
  |---|---|---|
  | "Thank you!" | 16 | 23 |
  | "Hello!" | 10 | 11 |
  | "Thanks!" | 9 | 12 |
  | "What are you?" | 7 | 10 |
  | "Who are you?" | 6 | 6 |
  | "How are you?" | 5 | 4 |

  P-dev's intersection with OASST2 TRAIN's 9,390 user turns is empty.
- **Census memory and time** (`/usr/bin/time -l`), peak footprint and RSS:

  | TRAIN split | Peak footprint | RSS | Time |
  |---|---|---|---|
  | `oasst2/` | 2.51 GB | 2.11 GB | 12.7 s |
  | `wiki-00000/` | 5.64 GB | 3.72 GB | 30.5 s |
  | `wiki-00020/` | 6.08 GB | 4.10 GB | 29.9 s |
  | `wiki-00040/` | 5.93 GB | 3.92 GB | 28.7 s |
  | the probe (an earlier `wiki-00000/heldout`, 0.70M tokens, as TRAIN) | 0.61 GB | — | 1.6 s |

**Uncovered short panel turns** (under four words, counted in every manifest): 3,219.

| Panel | Turns | Distinct |
|---|---|---|
| DEV | 257 | 80 |
| Fresh V2 | 266 | 78 |
| P-dev | 166 | 34 |
| p-long | 770 | 49 |
| p-long4k | 1,759 | 65 |
| conversational-v2 | 1 | 1 |

**Every excluded document** (56). The rule is `panel_short_request` (a 4-to-7-word panel turn found whole) or `panel_8gram`. The panel request names the file and the document index. The matched words are in each root's `excluded.jsonl`, in the store.

<details><summary>The 56 excluded documents</summary>

| Root | Source id | Rule | Panel request |
|---|---|---|---|
| `oasst2` | `01b5b826-a921-46e0-ba13-57d00c635125` | panel_short_request | `dev.json:dev:39` |
| `oasst2` | `089b3ba6-38b9-4c30-bfa6-871e1329217a` | panel_short_request | `dev.json:dev:36` |
| `oasst2` | `25daedff-8afa-473a-9253-ce7f84db6199` | panel_short_request | `dev.json:dev:10` |
| `oasst2` | `26867825-858c-4783-8c49-496b2b6c7e20` | panel_short_request | `dev.json:dev:36` |
| `oasst2` | `2c6ce434-ba80-4920-8831-5f75e771ac1c` | panel_short_request | `dev.json:dev:303` |
| `oasst2` | `2fe57f95-cf24-415f-8db7-797850ae1616` | panel_short_request | `conversational-v2.json:conv-clar-24` |
| `oasst2` | `3924c53a-a2ac-4b9f-a835-8c6af8e543aa` | panel_short_request | `dev.json:dev:32` |
| `oasst2` | `3d9dea2e-9e1b-4764-9671-c48b3bcba27c` | panel_short_request | `dev.json:dev:32` |
| `oasst2` | `47e53461-8d48-4943-9caa-804c3212308b` | panel_short_request | `dev.json:dev:32` |
| `oasst2` | `4992f0f6-fe1b-44b8-ac61-e6d284715275` | panel_short_request | `dev.json:dev:50` |
| `oasst2` | `530b7114-37ec-46e3-ab0b-7551ec8dbc08` | panel_short_request | `dev.json:dev:32` |
| `oasst2` | `5437b447-d21e-49c3-9728-25ff2b509c65` | panel_short_request | `dev.json:dev:10` |
| `oasst2` | `57b77191-7a2c-4296-8439-043fd4c9c040` | panel_short_request | `dev.json:dev:32` |
| `oasst2` | `587fcd9e-5d93-4778-a3e9-cb5a0852ed48` | panel_short_request | `dev.json:dev:216` |
| `oasst2` | `57e26ccb-5530-49fc-af6b-fe07dd7a8012` | panel_short_request | `dev.json:dev:32` |
| `oasst2` | `63724395-8562-4d89-94db-48ea3e0d2073` | panel_short_request | `dev.json:dev:32` |
| `oasst2` | `67025322-3b68-40d7-b144-bb759e1b300b` | panel_short_request | `dev.json:dev:10` |
| `oasst2` | `6d5d4d84-8a1c-46db-9940-eaf6f342e6d1` | panel_short_request | `dev.json:dev:50` |
| `oasst2` | `78f51778-84c4-425c-b9b5-93ff595e6d7d` | panel_short_request | `dev.json:dev:10` |
| `oasst2` | `7a0e008f-9932-4914-b91d-172d28096db8` | panel_short_request | `dev.json:dev:10` |
| `oasst2` | `82ad4a52-aeb6-4405-96d3-207811b548a5` | panel_short_request | `dev.json:dev:10` |
| `oasst2` | `847fed6e-de69-45b3-8a9c-653e8ddcc2ac` | panel_8gram | `conversational-v2.json:conv-story-01` |
| `oasst2` | `857e2a63-25e0-4c28-876e-cd82c159182c` | panel_short_request | `conversational-v2.json:conv-clar-10` |
| `oasst2` | `89ab5d1d-0115-44f6-8a0f-306aa0ce3b3a` | panel_short_request | `dev.json:dev:10` |
| `oasst2` | `8b200a81-993b-4627-985b-c6aba803382e` | panel_short_request | `dev.json:dev:39` |
| `oasst2` | `8c3b1dde-f0b7-4967-b925-a811a2d81970` | panel_short_request | `dev.json:dev:10` |
| `oasst2` | `94d536a6-04c1-4593-9337-bd6e3cdc487b` | panel_short_request | `dev.json:dev:191` |
| `oasst2` | `817a6d0c-8354-4553-8c77-9d464f45cf66` | panel_short_request | `dev.json:dev:10` |
| `oasst2` | `9e7a830e-1ac8-4cbf-9f15-173fb614eb6a` | panel_short_request | `conversational-v2.json:conv-clar-24` |
| `oasst2` | `9de2e936-c012-4879-9ce6-6d167d2790c2` | panel_short_request | `dev.json:dev:195` |
| `oasst2` | `9e2fbdac-e4f2-45ba-8ed8-40f7ee68cc79` | panel_short_request | `dev.json:dev:10` |
| `oasst2` | `9f300c87-a0e1-4d1d-956d-0f04c94a580e` | panel_short_request | `conversational-v4.json:conv-v4-unk-12` |
| `oasst2` | `ab7ebdce-e041-4b46-b935-9c93aa92e697` | panel_short_request | `conversational-v2.json:conv-clar-10` |
| `oasst2` | `aee0d7cd-12da-4c0e-9e98-6f778c23f95c` | panel_short_request | `dev.json:dev:10` |
| `oasst2` | `b545ab59-55a4-4eb6-b771-4e4d0f4624c7` | panel_short_request | `dev.json:dev:10` |
| `oasst2` | `c55433f8-6bf6-45ab-aed8-e36340fb233b` | panel_short_request | `dev.json:dev:303` |
| `oasst2` | `c8eb1678-6703-47b3-ab0b-cc39e6de6337` | panel_short_request | `dev.json:dev:32` |
| `oasst2` | `c9842af1-9a0d-422d-bb72-e701ad27d7c5` | panel_short_request | `dev.json:dev:10` |
| `oasst2` | `c9f2fdf1-6504-45e1-aefb-44a13b073324` | panel_short_request | `dev.json:dev:303` |
| `oasst2` | `d7b27842-6e10-4532-9f98-0ac584943bfd` | panel_short_request | `dev.json:dev:32` |
| `oasst2` | `db00e78d-7648-4968-be89-26b0beccc8c8` | panel_8gram | `conversational-v2.json:conv-story-01` |
| `oasst2` | `e5bb23b3-5b62-4fb8-a7b8-fae81fd3bc05` | panel_short_request | `fresh-v2.json:fresh-v2:134` |
| `oasst2` | `ea265789-670b-4e48-942b-adfaf75bd038` | panel_short_request | `dev.json:dev:32` |
| `oasst2` | `f01e079a-3133-40a9-9767-8ff8589f3ed6` | panel_short_request | `dev.json:dev:216` |
| `oasst2` | `f0844f52-f67b-4160-8ff0-666dee61e98f` | panel_short_request | `dev.json:dev:10` |
| `oasst2` | `f330a702-4542-41b6-aee5-d0c78dc81bd5` | panel_8gram | `conversational-v2.json:conv-story-01` |
| `oasst2` | `f632ba52-f29a-41e6-ae0d-e5d23a9166fc` | panel_short_request | `dev.json:dev:10` |
| `oasst2` | `f715d53c-5c9c-411c-89cf-dc2d78eee93d` | panel_short_request | `dev.json:dev:32` |
| `oasst2` | `f6cbcb1a-6ea8-46d7-8945-e5f923f93626` | panel_short_request | `conversational-v2.json:conv-unk-14` |
| `oasst2` | `f74e9a78-4a17-4fbd-99fa-a0b13924755f` | panel_short_request | `dev.json:dev:10` |
| `oasst2` | `f8dfa424-c795-40e8-b601-a14477723fc1` | panel_short_request | `dev.json:dev:32` |
| `wiki-00000` | `1715` | panel_short_request | `fresh-v2.json:fresh-v2:115` |
| `wiki-00020` | `37492570` | panel_short_request | `dev.json:dev:16` |
| `wiki-00020` | `37507111` | panel_short_request | `dev.json:dev:276` |
| `wiki-00040` | `2986412` | panel_short_request | `p-dev.json:p-dev:9` |
| `wiki-00040` | `3005924` | panel_short_request | `dev.json:dev:276` |
</details>

## Checks run at the head

- **The training crate's suite on the review-fix tree** (`--features parquet-input --no-fail-fast`): **170 passed, 0 failed, 1 ignored, over 8 targets.**
  - The targets: lib 123, `chat-grade` 26, `m-world-recall-corpus` 8 (1 ignored), `prepare-real-text` 11, `prepare-chat-parquet` 1, `prepare-text-parquet` 1, and `mix-chat-corpus` and the doc tests 0 each.
  - At the base it was 163/0/1 over the same 8 targets. The difference is the 5 new library tests and the 2 new bin tests.
  - At `9911a2e1c`, the commit that built the corpus, it was 169/0/1, before the second bin test.
- **The review fixes change no output.** `panel-turns` at the fix tree, re-run on the five frozen panels, writes files byte-identical to the stored `panel-turns/`.
- **The stored corpus is what this code builds.** `oasst2/` was rebuilt at `a8d07553e` from the stored raw file, with its `attempt.json` argv. Its tokens, masks, both manifests and `excluded.jsonl` are byte-identical to the stored root.
  - No file records the build's commit. `9911a2e1c` is the commit its release binary was built at.
  - Since that commit the build modes changed in no line; `real_text.rs` changed only in comments.
  - The Wikipedia roots were not rebuilt, since that needs their shards fetched again.
- **The harness checks, run on the commit that carries this REPORT:** `loop report --check` PASS, `loop pr-check --local` PASS, `uor-harness check` exit 0, and `loop gates --full` exit 0. The gates also passed at `088d225` and `a8d07553e`.
- **Negative gates.** Each was set aside with an edit, its test seen to fail, and the code restored. Then the suite was rebuilt and seen green again.
  - **NG1, the leading space after the role marker kept:** the round-trip test fails.
  - **NG2, the marker scan inside a run removed:** the round-trip test and the bin's `panel_turns_reads_back_the_user_turns_a_build_wrote` both fail.
  - **NG3, the `<|unk|>` refusal removed:** `user_turns_refuse_an_unk_in_a_prompt_run` fails.
  - **NG4, `enumerate` moved after the filter in `panel_turns`:** `panel_turns_skips_and_counts_a_document_with_no_user_turn` fails, with ids `t:0` and `t:1`. The earlier bin test passes under it, which is the gap the review found.
- **On real data:** p-long's and p-long4k's turns equal their stored files, by the SHA-256 of `[.[].user_turns]` (`4e5c524b…` and `21f52874…`).
- **Style:** `rustfmt` on both changed files, no line over 100 characters, and no banned word. Clippy reports nothing in either file, in the normal and the test profile, at `9911a2e1c` and again on the fix tree.
  - The crate's only clippy error is the earlier one at `chat-grade.rs:4893`, which claude-292 recorded.
- **Storage:** all 61 files read back from the store into a fresh folder. `shasum -c` passes every one, and the stored `SHA256SUMS` is byte-identical to the local one.
  - The review fixes changed the store's `README.md`, so it and `SHA256SUMS` were uploaded again and the whole root read back a second time. All 61 files pass, the sums are byte-identical again, and `raw/SHA256SUMS` passes for the raw files beside them.
  - The second round changed `README.md` once more. It and `SHA256SUMS` were uploaded a third time, and each read back byte-identical. The other 60 files were unchanged since the second full read-back.
- **Raw inputs:**
  - each Wikipedia shard's SHA-256 matches `raw/SHA256SUMS` before its build;
  - `raw/SHA256SUMS` passes for the stored OASST2 file and both READMEs;
  - the shards were fetched one at a time, 00000, then 00020, then 00040, and each was deleted after its build.

## What it unlocks

- **P5's sheet unit** can move P5's Data line to real text, through a decision card. It reads this folder's `README.md` and picks the mix and the mask.
- **For that sheet:**
  - report the six-phrase rows apart on DEV and Fresh V2;
  - a census of a larger real-text split needs up to 0.75 GB per million TRAIN tokens, so a machine with that much memory.
- **A larger Wikipedia build** comes from the same raw shards with a larger `max_docs`.

## Skills used

No skill applied. The loop's ORDER and REPORT procedure was used.

## Still open

- **The roadmap render:** owed until main's roadmap check is green.
- **No census reads panel prompts against TRAIN reply text,** the only place a panel prompt could sit in Wikipedia. Only the builder's exclusion screens it; P5's sheet decides whether that is needed.
- **The panel replies verbatim in OASST2** (DEV 18, …) are counted, not named.
- **`split_user_turns` skips a mask-0 `<|eos|>` wherever it falls.** One inside a prompt run would join the text around it. The builder admits no document holding special text, so no split it writes has one; another generator's split could. The fresh review rated it low.
- **No issue exists for this item yet,** so the PR closes none; the render that makes one is the owed one above.

## Plain summary

The project now has real human-written training text: about 28 million tokens of OASST2 conversations and Wikipedia articles, with held-out parts that no training run sees, stored privately with checksums. Every document was screened against the frozen test panels, and the count of shared prompts reads zero for the three panels that test memory. The two older panels share six greetings with real conversations, such as "Thank you!" and "Who are you?"; they are kept and named, so a score can set them apart. Wikipedia came out smaller than planned, because the counting tool needs about five times more memory per token on real text than on the old templated data.
