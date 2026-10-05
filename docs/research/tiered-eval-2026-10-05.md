# Tiered chat re-score of the existing chat models (2026-10-05)

References #820. Follows Step 0a (#1708) and uses the tier instrument from #1724 (`codex/tiered-eval` at `f3737c28`). This is a measurement only. No model was trained or changed.

## Summary

- **Tier C does not yet separate any model from a fixed reply.** The constant reply "I'm not sure. Can you tell me more about what you mean?" is acceptable on 42/160 tier C rows. Every model scores 25–35/160. Five of the seven models are significantly *below* that constant (paired exact McNemar p < 0.05), and the other two do not differ from it (29M p = 0.35, 100m-B p = 0.40). The constant gets its score from `clarify_or_on_topic` (18/24) and `unknowable_or_impossible` (15/24). The models score 0–1/24 on `unknowable`: they make up an answer instead of abstaining.
- **Every model beats the derangement control and constants 2 and 3** on tier C (p ≤ 3e-4). So the replies are about the request. They are just not better than "I'm not sure".
- **Memory is close to zero.** `multi_turn_memory` is 0–3/30 for every model. Constants score 0/30 there, so any memory pass is real, but there is almost none. The models pass the recall checks on 11–20 of the 84 checked rows overall.
- **No pairwise differences.** Against chat-29m-B-lr5e-4 and against chat-100m-C, no model differs on any tier at p < 0.05. The smallest p is 0.052 (100m-Cx vs 100m-C on K-ill-posed). The ~96M models are not measurably better than the 29M model on any tier. On tier C the 29M model is nominally ahead of 100m-C (34 vs 27, p = 0.19).
- **The models score highest on `smalltalk_feelings`** (10–18/26, against a best constant of 8/26). This is the only category where the models are clearly ahead of every constant.

## Method

- **Binary:** `chat-grade` built in release mode from `codex/tiered-eval` `f3737c28` (sha256 `a058872d…f229d`, at `~/.local/share/uor-r4/bin/chat-grade-f3737c28`).
- **Tier C:** `grade` on `conversational-v2-a.json,conversational-v2-b.json`, with `max_new_tokens=64`, `grader=qwen2.5:7b` (ollama digest `845dbda0…`, temperature 0, seed 1), greedy decoding, default constants and the embedded checks (sha256 `5589462f…`). Panel sha256 values match `data/panels/MANIFEST.sha256` (a `853770c4…`, b `8a8cfff7…`). `chat-grade check` passed, with a worst case of 323/384 positions. The settings are the same as the existing `*-powered-7b` reports. The three constant controls gave the same tallies (42, 7, 2) in all seven runs.
- **Tier K and everyday:** `tiers` recomputed from the existing graded reports `~/uor-r4-local/ladder/grades/<model>-powered-7b/report.json`, with no regeneration, using the embedded text-only ill-posed list (90 ill-posed, 110 clean). Those reports have no constant controls, so the K and everyday columns show the derangement control only.
- **Pairs:** `compare tier=C` on the new reports, and `compare tier=all` on the existing reports. Each pair is matched by row id and tested with a two-sided exact McNemar.
- **Models** (read-only, under `~/uor-r4-local/ladder/runs/<model>/model`): chat-29m-B-lr5e-4 (28.96M parameters) and chat-100m-{B,C,L,Cx,L12,C12} (96.02M each). Tokenizer: `inputs/claude-t4-1433-resume/bundle-learned-1/tokenizer.json`.
- **Cost:** 7 sequential tier-C runs from 07:47 to 10:38 UTC (2 h 51 min), 22–30 min each, with 256–612 s of generation in each run. The rest of each run was grader calls: 5 × 160 rows × 2 questions. Jobs ran one at a time with the lock `claude-tiered-rescore` (threads 2, gpu true), and the lock has been removed. The build took 7 min 49 s at -j 2. New storage is 2.4 MiB of report roots. No external compute was used.

### Model x tier

| model | C acceptable | C fluent | C relevant | C check pass /84 | C derangement acc. (p) | C vs const-1 "I'm not sure..." 42/160 (p) | C vs const-2 (p) | C vs const-3 (p) | K-clean acc. (derangement, p) | everyday acc. (derangement, p) |
|---|---|---|---|---|---|---|---|---|---|---|
| chat-29m-B-lr5e-4 | 34/160 | 60 | 45 | 11 | 3 (3.7e-08) | 0.35 (const ahead) | 4.6e-07 | 1.9e-08 | 10/110 (0, 0.002) | 9/32 (0, 0.0039) |
| chat-100m-B | 35/160 | 54 | 39 | 13 | 2 (1e-08) | 0.4 (const ahead) | 7.7e-07 | 1e-08 | 12/110 (1, 0.00098) | 9/32 (0, 0.0039) |
| chat-100m-C | 27/160 | 54 | 34 | 14 | 1 (2.2e-07) | 0.049 (const ahead) | 3.6e-05 | 1.6e-06 | 12/110 (0, 0.00049) | 8/32 (0, 0.0078) |
| chat-100m-L | 27/160 | 47 | 32 | 20 | 1 (2.2e-07) | 0.032 (const ahead) | 0.00018 | 1.6e-06 | 12/110 (0, 0.00049) | 5/32 (0, 0.062) |
| chat-100m-Cx | 25/160 | 43 | 28 | 15 | 3 (2.7e-05) | 0.014 (const ahead) | 0.00053 | 5.6e-06 | 8/110 (0, 0.0078) | 7/32 (0, 0.016) |
| chat-100m-L12 | 25/160 | 38 | 28 | 17 | 4 (0.0001) | 0.019 (const ahead) | 0.00028 | 5.6e-06 | 11/110 (0, 0.00098) | 4/32 (0, 0.12) |
| chat-100m-C12 | 33/160 | 51 | 40 | 14 | 2 (3.7e-08) | 0.24 (const ahead) | 8.7e-07 | 3.7e-08 | 10/110 (0, 0.002) | 8/32 (0, 0.0078) |

### Tier C acceptable by category (constant-1 in last row)

| model | multi_turn_memory | self_contained_instruction | clarify_or_on_topic | smalltalk_feelings | story_continuation | unknowable_or_impossible |
|---|---|---|---|---|---|---|
| chat-29m-B-lr5e-4 | 1/30 | 8/30 | 5/24 | 18/26 | 1/26 | 1/24 |
| chat-100m-B | 2/30 | 8/30 | 7/24 | 17/26 | 1/26 | 0/24 |
| chat-100m-C | 1/30 | 6/30 | 5/24 | 15/26 | 0/26 | 0/24 |
| chat-100m-L | 1/30 | 3/30 | 8/24 | 14/26 | 0/26 | 1/24 |
| chat-100m-Cx | 0/30 | 6/30 | 9/24 | 10/26 | 0/26 | 0/24 |
| chat-100m-L12 | 2/30 | 4/30 | 8/24 | 11/26 | 0/26 | 0/24 |
| chat-100m-C12 | 3/30 | 8/30 | 9/24 | 13/26 | 0/26 | 0/24 |
| constant-1 | 0/30 | 0/30 | 18/24 | 8/26 | 1/26 | 15/24 |
| constant-2 | 0/30 | 0/30 | 0/24 | 7/26 | 0/26 | 0/24 |
| constant-3 | 0/30 | 0/30 | 0/24 | 0/26 | 2/26 | 0/24 |

### Paired vs chat-29m-B-lr5e-4 (acceptable: ref / model, discordant ref-only / model-only, exact McNemar p)

| model | C | K-clean | K-ill-posed | everyday |
|---|---|---|---|---|
| chat-100m-B | 34 / 35 (12/13, p=1) | 10 / 12 (7/9, p=0.8) | 24 / 24 (7/7, p=1) | 9 / 9 (3/3, p=1) |
| chat-100m-C | 34 / 27 (14/7, p=0.19) | 10 / 12 (7/9, p=0.8) | 24 / 26 (8/10, p=0.81) | 9 / 8 (4/3, p=1) |
| chat-100m-L | 34 / 27 (15/8, p=0.21) | 10 / 12 (9/11, p=0.82) | 24 / 21 (10/7, p=0.63) | 9 / 5 (5/1, p=0.22) |
| chat-100m-Cx | 34 / 25 (17/8, p=0.11) | 10 / 8 (8/6, p=0.79) | 24 / 16 (16/8, p=0.15) | 9 / 7 (4/2, p=0.69) |
| chat-100m-L12 | 34 / 25 (15/6, p=0.078) | 10 / 11 (9/10, p=1) | 24 / 25 (11/12, p=1) | 9 / 4 (5/0, p=0.062) |
| chat-100m-C12 | 34 / 33 (13/12, p=1) | 10 / 10 (7/7, p=1) | 24 / 21 (11/8, p=0.65) | 9 / 8 (3/2, p=1) |

### Paired vs chat-100m-C (acceptable: ref / model, discordant ref-only / model-only, exact McNemar p)

| model | C | K-clean | K-ill-posed | everyday |
|---|---|---|---|---|
| chat-29m-B-lr5e-4 | 27 / 34 (7/14, p=0.19) | 12 / 10 (9/7, p=0.8) | 26 / 24 (10/8, p=0.81) | 8 / 9 (3/4, p=1) |
| chat-100m-B | 27 / 35 (6/14, p=0.12) | 12 / 12 (10/10, p=1) | 26 / 24 (10/8, p=0.81) | 8 / 9 (2/3, p=1) |
| chat-100m-L | 27 / 27 (9/9, p=1) | 12 / 12 (10/10, p=1) | 26 / 21 (13/8, p=0.38) | 8 / 5 (4/1, p=0.38) |
| chat-100m-Cx | 27 / 25 (7/5, p=0.77) | 12 / 8 (9/5, p=0.42) | 26 / 16 (16/6, p=0.052) | 8 / 7 (2/1, p=1) |
| chat-100m-L12 | 27 / 25 (10/8, p=0.81) | 12 / 11 (10/9, p=1) | 26 / 25 (12/11, p=1) | 8 / 4 (4/0, p=0.12) |
| chat-100m-C12 | 27 / 33 (6/12, p=0.24) | 12 / 10 (9/7, p=0.8) | 26 / 21 (10/5, p=0.3) | 8 / 8 (2/2, p=1) |

In the first table, "(const ahead)" means the constant was acceptable on more discordant rows than the model. A p below 0.05 there means the model is significantly *worse* than that constant.

## What this means for the instrument

Tier C passes its derangement and content-free-constant controls. It does **not** pass the "I'm not sure" constant, which is the control #1724's README says the clarify and unknowable categories must be read against. As currently weighted, a model that answered every request with constant 1 would beat every chat model we have. The tier C total is therefore not usable as the headline score. The per-category rows that do carry signal are:

- `multi_turn_memory`: every constant scores 0, so this is a pure memory floor.
- `self_contained_instruction`: constants score 0.
- `smalltalk_feelings`: the models are ahead of every constant.

Proposed instrument fixes (for #1724, not done here):
1. Report tier C as per-category results against the best constant *per category*, not one total.
2. Or score clarify and unknowable rows by whether the model abstains or clarifies *when it should* and answers *when it can*, so that a fixed abstention cannot pass both kinds of row.

## Sealed report roots

All roots are under `~/uor-r4-local/ladder/grades/tiered-2026-10-05/`. Each was claimed exclusively and sealed with `manifest.json`. The second column is the sha256 of each root's manifest.

| root | manifest sha256 (prefix) |
|---|---|
| `chat-100m-B-C-powered-7b` | `86d3bf514631ef91` |
| `chat-100m-B-K-tiers` | `d9f5679dbaaf5dac` |
| `chat-100m-C-C-powered-7b` | `c333d0cb933afe67` |
| `chat-100m-C-K-tiers` | `d68cbfabf7567411` |
| `chat-100m-C12-C-powered-7b` | `56d1e569b9de03c9` |
| `chat-100m-C12-K-tiers` | `4ac89b73c71f47d8` |
| `chat-100m-Cx-C-powered-7b` | `b9c7dbaa09a5bf25` |
| `chat-100m-Cx-K-tiers` | `e67c1cce1b6324ba` |
| `chat-100m-L-C-powered-7b` | `ad15191667e66ad6` |
| `chat-100m-L-K-tiers` | `9a221c28eba6b6e0` |
| `chat-100m-L12-C-powered-7b` | `369b0e9b2d2c5245` |
| `chat-100m-L12-K-tiers` | `969d2b9aa5bc2e42` |
| `chat-29m-B-lr5e-4-C-powered-7b` | `4dd98b19f0741787` |
| `chat-29m-B-lr5e-4-K-tiers` | `7170c3c6ca1762e1` |
| `cmpC-chat-100m-C-vs-chat-100m-B` | `c42fb4c63eed2436` |
| `cmpC-chat-100m-C-vs-chat-100m-C12` | `6eb790aa6d24279e` |
| `cmpC-chat-100m-C-vs-chat-100m-Cx` | `b665b5b76ca558e3` |
| `cmpC-chat-100m-C-vs-chat-100m-L` | `f7e6bd8e20028809` |
| `cmpC-chat-100m-C-vs-chat-100m-L12` | `c6ca68c70778ae79` |
| `cmpC-chat-100m-C-vs-chat-29m-B-lr5e-4` | `82fc7ca610878cdd` |
| `cmpC-chat-29m-B-lr5e-4-vs-chat-100m-B` | `7038fdf3e1535b28` |
| `cmpC-chat-29m-B-lr5e-4-vs-chat-100m-C` | `1419a038571943ea` |
| `cmpC-chat-29m-B-lr5e-4-vs-chat-100m-C12` | `5e57a55b3b4179fa` |
| `cmpC-chat-29m-B-lr5e-4-vs-chat-100m-Cx` | `ffbec64978c011cb` |
| `cmpC-chat-29m-B-lr5e-4-vs-chat-100m-L` | `e10105e66e696eb7` |
| `cmpC-chat-29m-B-lr5e-4-vs-chat-100m-L12` | `0856248999000929` |
| `cmpK-chat-100m-C-vs-chat-100m-B` | `11242c3f3128266b` |
| `cmpK-chat-100m-C-vs-chat-100m-C12` | `903570222dafac22` |
| `cmpK-chat-100m-C-vs-chat-100m-Cx` | `9c714467612a7e78` |
| `cmpK-chat-100m-C-vs-chat-100m-L` | `2037d86a24e3e4ea` |
| `cmpK-chat-100m-C-vs-chat-100m-L12` | `5ea847261ab2b646` |
| `cmpK-chat-100m-C-vs-chat-29m-B-lr5e-4` | `3fe4f05b6f82db14` |
| `cmpK-chat-29m-B-lr5e-4-vs-chat-100m-B` | `45c6c2400447f6b9` |
| `cmpK-chat-29m-B-lr5e-4-vs-chat-100m-C` | `f453746c92d4eb6a` |
| `cmpK-chat-29m-B-lr5e-4-vs-chat-100m-C12` | `531cec133dde40a8` |
| `cmpK-chat-29m-B-lr5e-4-vs-chat-100m-Cx` | `fdda48b4b40da5ef` |
| `cmpK-chat-29m-B-lr5e-4-vs-chat-100m-L` | `0fa5b27bd864a856` |
| `cmpK-chat-29m-B-lr5e-4-vs-chat-100m-L12` | `8c77b4ea00a8782e` |

## Limits

- One grader (qwen2.5:7b) and one greedy decode per model. No second annotator has checked the panel or the tier K labels.
- Tier K and everyday come from the earlier reports, which were generated by binary `a621514c`. Only the tier assignment is new.
- This re-score answers no capability question beyond these panels. It does not establish general conversation, memory or reasoning ability.
