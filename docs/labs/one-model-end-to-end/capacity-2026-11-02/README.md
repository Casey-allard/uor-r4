# claude-259: the capacity sweep's served evidence (2 Nov 2026)

What the laptop served from the capacity sweep: the [REPORT](../../../loop/tasks/one-model-end-to-end/claude-259/REPORT.md) reads it, under [its sheet](../../../harness/prep/one-model-end-to-end-capacity-2026-11-02.md). Six exports were served: (b) 256/4 and (h) 256/8 at seeds 1–3. (w) 512/8 refused to export at every seed (the REPORT's negative 9). The exports themselves are in the HF store at `results/claude/one-model-end-to-end-capacity-2026-11-02/served/`, with `SHA256SUMS`; their hashes are at the end of this page.

## What each folder holds

- **`conversations/`:** `conv-a.txt`, `conv-b.txt` and `conv-c.txt`, the text fed to `uor-chat` on standard input. `/tokens` prints the turn's ids after each reply.
- **`transcripts/EXPORT-a.txt`, `-b.txt`, `-c.txt`:** `uor-chat --stack EXPORT --tokenizer TOKENIZER --protocol 2 --max-tokens 64`, given one conversation, standard output and error together. Three edits only: terminal escapes and carriage returns are removed, uor-chat's four-line title block is cut, and the home folder's path is written `~/`.
- **`decode/EXPORT-tps.txt`:** ten timed runs of `uor-chat --prompt-ids 0,55,2728,28,223,2485,2144,2471,435,223,22,25,16,201 --max-tokens N`, N alternating 1 and 177, wall time from zsh's `EPOCHREALTIME`. Decode tok/s is 176 / (the median 177-run − the median 1-run). `h-s1-tps-run1.txt` is h-s1's first reading; neither of h-s1's is usable (the REPORT's negative 13).
- **`parity/EXPORT-toks.out`:** the summary line of `fbs-toks EXPORT --replies PROMPTS --out TOKS --reply-tokens 64` over DEV's 310 prompts, taken with `jq` from the arm's own `samples-memory.json`. The first-token and whole-reply counts in the REPORT compare those files' replies with the float model's. b-s2's line is from a re-run whose output is byte-identical to the first run's (the REPORT's negative 15).
- **`trace/conv-a-t3.ids.json`:** the 86 ids uor-chat served in conv-a's turn 3 on b-s2. **`trace/b-s2-conv-a-t3.trace.json`:** `fbs-toks` with `--hamming-trace` on `b-s2.uorlut` over those ids (schema `uor-r4.hamming-trace/1`, the integer side): at depths 2 and 6, each of the 4 heads' key and query codes, and the up to 16 earlier positions it keeps at every position. Heads 0 and 1 are the copy heads.

## The exports

| Export | sha256 |
|---|---|
| b-s1 | `f04f3039c7ee09c38534bbe082b10759c6d62646642a479fad7d2ab23946322c` |
| b-s2 | `4d411f35c8ce49d227a897b40931da9f25a32a3b527a12995a47bcd4586f8fa5` |
| b-s3 | `05c0a9544a46c7a54f1997142378c722abbca0282970465b9a118c9d012a0fb3` |
| h-s1 | `1ba010f7ff7158782ad63ed717b3abf2a6901fd6cacebdd9821c64de63a23926` |
| h-s2 | `e80c15d0620d40b8d112b052f0a4896e2d57dbfca3be0ea2e9bf7a43f8009672` |
| h-s3 | `7fc63395932a873d2202f67e3f41aede2816dd1e0f7f8ed6bd1cdefa412768a2` |
