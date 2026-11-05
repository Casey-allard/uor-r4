# REPORT: integer-long-context, claude-288 (the integer session serves carry past the trained window)

Lab: claude. [ORDER](./ORDER.md). PR #3717, item #3471. Closes #3471. Plan step: P4 in Casey's plan of 4 November (long context served), unit 3 of 3. Base `03f220688`.

**In one paragraph.**
- **What changed in the model that serves:** an a7r artifact now serves a conversation past its trained 192 tokens under **carry**, the rule claude-287's decision review chose (provisionally). `fbs-export --context C` writes the carry artifact: the shape's context is C, the turn memory's key window is C (one block, the whole served window, as the float decode keys it under carry), and the header records the trained window W. The payload bytes are unchanged; the file grows by 64 bytes (the JSON header by 24, and the payload's 64-byte alignment moves its start from 8,768 to 8,832).
- **Done-whens 3 to 5 are met:** the session serves carry, tested against the float forward's carry rule past 192, with a negative gate; tok/s and resident memory are read to 8,256 (and 32,768); this REPORT, one notebook row and the pipeline map's gap 6.
- **What the numbers do not say:** that carry serves a better conversation. On p-long's 609 long queries the served model and the float model are equally weak against the reference: both start 3–5% of replies with the right token, and almost none match it whole. The served carry reproduces the float carry's reply on 62% (seed 1) and 60% (seed 2), against 88% and 75% on DEV's 16 short prompts: the whole reply on 364 and 349 of the 588 queries inside the float's 2,048 window, and up to 2,048 on 14 and 15 of the 21 past it.
- **What changed in the model that trains:** nothing.

## Tokens per second first

`fbs-toks ARTIFACT --context-sweep --tokens TOKENS_U16` on a7r's carry exports, this M1 (8 cores, 4 performance, 16 GiB) on AC power. The prompt is p-long4k's first ids (claude-287's `tokens.u16`, `ffefe084…`). Each point is one warm-up, then 20 timed runs of the prefill and 64 greedy decode steps, with the Hamming read and without it on the same artifact. Medians; the context is where the decode ends.

| context | decode, with read (s1 / s2) | decode, without read | prefill, with read |
|---|---|---|---|
| 192 | 1,340 / 1,323 tok/s | 1,447 / 1,445 | 1,380 / 1,366 |
| 576 | 1,254 / 1,254 | 1,441 / 1,445 | 1,328 / 1,323 |
| 2,112 | 1,000 / 1,003 | 1,441 / 1,448 | 1,164 / 1,165 |
| 8,256 | **527 / 529** | 1,445 / 1,444 | 755 / 766 |
| 32,768 (s1 only) | **176** | 1,449 | 314 |

- **Peak resident memory** for the whole sweep to 8,256: 276 MB at both seeds (275,808,256 and 275,578,880 bytes). To 32,768 at seed 1: 377 MB (376,635,392 bytes), in 3,117 s of wall time. The 32,768 artifact's replies at the four shared points equal the 8,256 artifact's, with the read and without.
- **The read's cost grows with the context:** without it decode stays at about 1,445 tok/s, and with it the extra time per token is about 146 ns per earlier position at 8,256 ((1/527 − 1/1,445) s over 8,256 positions). claude-242 measured about 167 ns on claude-240's artifact.
- **At short context the writer sets the speed,** not the read: the writer is 484 of 744 µs of a step at a 128-token prefill (`fbs-toks --parts` on the seed 1 carry artifact at 8,256: 484,083 of a 743,625 ns step median, `parts-s1-c8256.json`; the parts sum to 998 per mille of the step). That is why a7r's 1,340 tok/s at 192 sits below claude-242's 4,647 on claude-240's artifact.
- **What this is not:** a served conversation. The sweep's prefill marks no reply, so the turn memory does not move. P-1's 609 long queries, which do mark a reply, took 2.0 s each, a mean prompt of 1,905 tokens and 32 reply tokens (1,220 s per seed), with 192 MB resident.

## What did not work

- **Parity with the float model is lower on long prompts than on short ones.**
  - On p-long (prompts of 1,652 to 2,034 tokens), the served carry reply equals the float carry reply, up to and including the first `<|eos|>` (and, for the 21 queries that pass the float's 2,048 window, up to 2,048), on **378 of 609** at seed 1 and **364 of 609** at seed 2.
  - On DEV's 16 prompts (17 to 176 tokens), inside 192, the base artifact matches the float on 14 of 16 and 12 of 16; the carry artifact matches on the same counts.
  - Most disagreements start at the first token: 152 of 231 at seed 1 and 144 of 245 at seed 2. The first token agrees on 457 and 465 of 609.
  - **The class: the integer-to-float serving gap the base already has, not the carry rule.** Three readings point there:
    - inside 192, where both artifacts run, carry's replies are the base's (P-2), and the base already differs from the float on 2 and 4 of 16;
    - the turn memory's keys follow the float carry rule exactly (G2);
    - on a synthetic trunk the carry session equals a native export at the longer context at every position, and departs from the in-crate float reference only at snap near-ties (least transition, gate or rate margin under 10⁻⁶; on this seed the gate's), the integer's known rounding difference (G3).
  - **The cause on a7r is a candidate, not shown:** more prompt tokens give more such ties a chance to change a pick. No a7r trace was read to confirm it. The DEV rate rests on 16 prompts, so its own interval is wide (14 of 16 is about 64% to 97%).
  - **Neither side is better against the reference.** First token right: integer 20, float 19 at seed 1; integer 28, float 32 at seed 2. Reference prefix tokens summed: 40 / 38 and 65 / 67. Exact reference replies: 0 / 0 and 1 / 1 (the same query).
- **21 queries pass the float's 2,048 window.** claude-287's float carry ran at C = 2,048, so a reply past 2,048 tokens reads a slid window there, while the integer carry at 8,256 does not. Those 21 are compared up to 2,048 only: 14 of 21 and 15 of 21 agree that far.
- **The integer suite has 38 failures, none from this change.**
  - 35 tests in 7 binaries need local bundles or a `tokenizer.json` that this checkout does not hold (`load bundle: ... NotFound`).
  - 2 in `m5_full_path_cost_controls`: the step disassembly audit finds a `blr` in `core::ptr::drop_glue::<serde_json::error::ErrorCode>`. The same 2 fail the same way at the base `03f220688`, run in a separate worktree with the same toolchain and target folder, so they are not from this branch.
  - 1 in `m7_parity_library_cli_browser`: `wasm-pack` calls `cargo build`, and this Mac's `/opt/homebrew/bin/cargo` shadows Rust's cargo (`error: unknown command 'build'`).
- **The past-W reference test failed at first as written.** Its first form compared the carry session with the float reference to 2 LSB at every position, and failed at position 19 (Q12 5,362 against 5,410). A probe classed it as the instrument: the reference's gate snap is a near-tie there (margin 1.7 × 10⁻⁸), and the carry session equals a native-context export exactly at all 48 positions. The test now asserts the exact native equality, and allows a reference departure only at such a tie.
- **The parser accepted a malformed carry artifact.** DeepSeek's WIP audit on the PR found that `Container::parse` took a carry header (`source.trained_context` present) whose turn memory still keyed at the trained window: it would serve block-cleared keys under carry provenance. The exporter never writes one, but the parser did not refuse it. It now refuses that, and a point key on a carry stack (the code review's minor finding: only the writer refused one). `carry_export_refusals` pins both by message. **Negative gates:** with the parse lines set aside the test fails at the key-window assertion; with only the point check removed it fails at "a point key under carry provenance". Both pass restored.
- **One pushed commit broke the float crate.** A `perl` substitution in `10d287b63` dropped `forced_reading`'s `eos: u32` parameter, and the commit was chained after the build with `;`, not `&&`. `ed7788bd6` restores it; the float crate's diff is now the one test.

## What changed

- **`stack/format.rs`:** `export_fbs_trunk_uorlut01_carry(snap, C)`. The shape's context is C; the turn memory's `key_window` is C; `header.source.trained_context` records W. It refuses C < W and C past `MAX_CONTEXT`; the FBS export writes no read select, so the rank read's `MAX_FLOCK_CONTEXT` does not bind. Without carry the output is byte for byte as at the base, and no `trained_context` is written. `Container::parse` refuses a recorded trained context outside 1 to the context, a carry stack whose turn memory does not key over the served context, and a carry stack with a point key. `add_point_key_uorlut01` refuses a carry stack, because a point key is built on 192-token windows while the carry stack's turn memory keys over the whole context (found by the code review; a7r has no point key).
- **`bin/fbs-export.rs`:** `--context C`.
- **`bin/fbs-toks.rs`:** `--context-sweep --tokens` times every point of 128, 512, 2,048, 8,192 and 32,704 (each with 64 decode steps) that the artifact's context holds, prints each it skips, and records them as `skipped_prefills`. It no longer refuses an artifact that cannot hold 32,768.
- **`stack/mod.rs`:** the export's re-export.
- **`stack/tests.rs`:** the export's tests (`carry_export_serves_past_the_trained_window`, `base_export_records_no_trained_context`, `carry_export_refusals`) and the three session tests below.
- **`in_space_long.rs`:** one test, `the_carry_decode_keys_read_across_the_trained_window`, the float side of the key contract.

## Measurements

**The exports (G1),** from claude-227's stored snapshots and run configs (`results/claude/integer-retention-rates-parity-2026-10-31/`), with `fbs-export --raw-gates --raw-transition --run-config ...` as claude-243 ran it:
- **The base exports reproduce the a7r exports claude-243 served** (recorded in its local sums for `R/a7/a7r-sN.uorlut`, not in its REPORT; claude-252's `pulled/SHA256SUMS` agrees): seed 1 `f86a7856366680c687f5cb20d8214937f00f9d3a34379b169f71b753475be175`, seed 2 `0a69a009cc1bc8d3aeefc48c20508c5ef5db7be5fdc6dfce681113b596d105ed`, 51,310,784 bytes each.
- **Carry at 8,256:** seed 1 `49eb052ef14ae3f9feb96cbb55bc506a2ece08bbad8b9d33dc9e58d0f443aec5`, seed 2 `bde4295d64e2f3512be8c46f8d40d0bf5c27c629fabe62573e9c7cf1f7a5d31e`, 51,310,848 bytes each.
- **Carry at 32,768:** seed 1 `8499e6d8b68f26c44895df222c7f8e92fec997ccb8e455adf642a6d3dad4ad92`, seed 2 `9b71404d6286dc738f28c1110a3a7e98a2e9eb3e671a649a72993c706d87b2b0`.

**The tests (G2, G3), CPU, synthetic models:**
- **G2, keys:** `carry_session_keys_follow_the_float_rule_past_the_trained_window`. A 601-token script with three turns crossing 192 (180–230, 400–440, 470–600), trained window 192, carry at 640. At every keyed row the session's opening and reply keys equal the float carry rule, and the literal anchors (1, 231, 441) and the reply offset are pinned. The float crate's `the_carry_decode_keys_read_across_the_trained_window` pins the same anchors on the float decode's own `opening_slots` and `greedy_reply_key`, since the float crate cannot run the integer session (D29). It forms the decode's window and cut by hand, as the decode forms them, rather than calling the decode, so a later change to the decode's window would not fail it.
  - **Negative gate:** with `key_window` set back to the trained 192 (format.rs, the carry line set aside as a patch), the test fails at turn 180, row 192, where the windowed key is `[MAX, 1, MAX, 76]`. It passes with the line restored.
- **G3, within W:** `carry_session_serves_the_base_steps_within_the_trained_window`. With a Hamming read and a turn memory holding planted opening and reply keys, the base and carry sessions give equal steps and memory moves at every position below 192, and both stored keys are read. At 192 the base refuses (`ContextFull`) and the carry steps on.
- **G3, past W:** `carry_session_matches_the_float_reference_past_the_trained_window`. A trunk trained at W = 8, carried to 64, over 48 positions. The carry session equals a native export at context 64 at every position. Against the in-crate float reference, query drift past 2 LSB is allowed only where the reference meets a new snap near-tie at that position (a least transition, gate or rate margin under 10⁻⁶; this seed's one is the gate at position 19), and a different cell or token only under a reference margin of 1/1024; at most 4 such positions, all past W. The in-crate reference has no Hamming read, so the read is checked only as a trace: with one, the last step keeps positions more than W back. The read's own values against a float read belong to `integer-hamming-read`, and are not tested past W here.

**P-2, within W (DEV's 16 prompts, 32 reply tokens):** at both seeds 15 of 16 replies are byte-identical between the base and carry artifacts. The 16th (176-token prompt) is the base's 16 tokens, where it stops at 192 (`end: context`), and then the carry's 16 more.

**P-1, past W (p-long's 609 queries, memory on, 32 reply tokens):** each prompt prefilled in a fresh carry session at 8,256, then greedy. 1,220 s per seed, peak RSS 192 MB.

| | seed 1 | seed 2 |
|---|---|---|
| prompts | 1,652–2,034 tokens | the same |
| whole reply = float carry's (588 inside 2,048) | 364 | 349 |
| the same to 2,048 (21 past it) | 14 | 15 |
| first token = float's | 457 | 465 |
| first token = reference: integer / float | 20 / 19 | 28 / 32 |
| DEV base rate inside 192 (integer = float) | 14 of 16 | 12 of 16 |

The integer side's `keyed` column is 0 throughout: it names the point-key opening, and a7r has no point key (schema `/7`). The turn memory is read through the step, not that column.

**P-3, speed and memory:** the table under "Tokens per second first". P-3's release `fbs-toks` has sha256 `e44a4a0f836d06aca0aef01e8166825d1dd536f503ddc5b064e9abceff7816cc` (`p3-inputs.sha256`). It is the claude target folder's binary (`.worktrees/claude-i3-projection/local/claude/target/.../release/fbs-toks`), last written at 04:28 (−0500), before every run here: P-2 ran at 04:32 (four runs of about 1.2 s), P-1 from about 04:33 to 04:53, P-3 from 04:53 to 06:06, the `--parts` reading at 06:15. The binary still has that sha, so P-1 and P-2 ran it too, though their outputs do not name it. Its source commit is not recorded: it was built from this branch's tree between `84863db1a` (04:25) and `c389d77c4` (04:31), and `fbs-toks.rs` has not changed since `c389d77c4`. Load averages before and after each run are in `p3-*.load`: 4.2 to 6.0 (s1, 8,256), 6.0 to 3.3 (s2, 8,256), 3.3 to 4.2 (s1, 32,768). Other labs' work ran beside it. Wall time 450 s and 443 s for the sweeps to 8,256. The sweep at 8,256 printed that it skips the 32,704 prefill, and records it in `skipped_prefills`.

**Sealed** in the store under `results/claude/integer-long-context-2026-11-05/c288/` with a `SHA256SUMS`, sha256 `881ee16c45d06f07f216f468076ca7d328c7ee7b87e8c38a17d9ad22da32d2ac`, over 34 files: the two carry artifacts at 8,256 that P-1 read, P-1's and P-2's replies, P-3's sweeps with their load and `time -l` output, the sweep script `p3.sh`, and P-1's comparison `p1-compare.jq` with its output per seed. `uor-store ls` lists all 35; the stored `SHA256SUMS` and `p1-compare-s2.json` were read back and match. The float side is claude-287's sealed `a7r-sK-carry/samples-p-long-queries-memory.json` (`ff63faee…`, `f2da21b9…`) and `samples-memory.json` (`570ee161…`, `656d41ad…`), each matching that seal's `SHA256SUMS`. The base and 32,768 exports are not stored: their shas are above, and `fbs-export` rebuilds them from claude-227's stored snapshots. The `--parts` reading, taken after the seal, is stored beside it under `c288-parts/` (its JSON, `time -l` output and load, with its own `SHA256SUMS`, sha256 `06ea350833536dd6c91715f9ed72eafb5e1111f201f6f297c7a64935d4696d04`; that file and the JSON were read back and match).

## Checks run at the head

- `cargo test -p uor-r4-integer --features serving-wasm --no-fail-fast` at `5469a3278` (the parser fix): 698 passed, 38 failed, 4 ignored, over 52 test targets. The 38 are the same tests, by name, that failed at `c47c61796` and before, classified above; none is from this change. The library's own 469 tests pass.
- The float crate's suite at `ed7788bd6`: 305 passed, 0 failed. The float crate has not changed since.
- The stub-nvcc check of the float crate (`--features cuda --all-targets`): clean.
- G2's negative gate, run and recorded above.
- At `631eae201`, and again at the head this REPORT is committed in (those results are in the PR's review record): `uor-harness check` exit 0, warnings only (heartbeat receipts for M2, M3 and M7, and a merged local branch of another session); `loop gates --full` 11 of 11 GREEN; `loop report --check` PASS; `loop pr-check --local` PASS, with the advisory read-path and FILEMAP warnings.
- **The roadmap render did not run.** `cargo xtask roadmap render` refuses main's `infra-desktop-client-tauri.md` (Kimi's, from #3675), whose status `in_progress` is not one the check allows. So `docs/roadmap/README.md` and `issues.json` still show this item's old status; the first render after that file is fixed picks up `status: done`. The file is another lab's and was not touched; the note is on #2028.

## Skills used

None beyond the loop's own steps.

## Still open

- **Carry is provisional** (claude-287): no checkpoint yet recalls a far note, so no rule is shown to use earlier context. The row is re-read on the first checkpoint that does.
- **The chat binary** (`bin/uor-chat.rs`) is DeepSeek's lane (`cpu-training-pilot`), and was not run here. Read from the code: it serves the context the artifact declares (`shape.context`), and prints a notice when that is not the sealed bundle contract's 256 (`stack/chat.rs:40`). So a long conversation there needs only a carry export.
- **The Hamming read's cost** grows with the context, as claude-242 found: it is 64% of decode at 8,256 and 88% at 32,768 on a7r (from the with- and without-read rates). claude-242's next step, keeping the 16 nearest without a sort, is `integer-hamming-read`'s.
- **The m5 disassembly audit** fails at the base on this Mac (`serde_json` drop glue in the audited call graph). Noted for the item that owns `report_output.rs`; not touched here.

## Plain summary

The chatbot used to stop after 192 tokens. Now the same trained model, exported with a longer context, keeps going: it carries its running state and its memory through the whole conversation, as the float model does under carry. Within 192 tokens nothing changes. Past 192 its replies are the float model's on about six in ten long prompts, because the two round some close calls differently. It is no worse than the float model against the right answers, and both are still weak. On this laptop it writes about 1,340 tokens a second near the start of a conversation, about 530 at 8,000 tokens and 176 at 32,000, because each new token looks back over everything said so far.
