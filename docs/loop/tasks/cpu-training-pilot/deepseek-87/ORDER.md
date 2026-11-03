# ORDER: cpu-training-pilot, deepseek-87 (CPU copy key 2 parity)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `1499c09ed`.

## Goal

Make the independent CPU FBS Hamming port execute copy key 2 with the same
ordered two-token query and key as the integer serving session, while
preserving the historical copy-key-1 path.

## Cancel probe

Claude's merged copy-key unit widened `StackHamming` to accept `copy_key=2`,
but `crates/uor-r4-cpu-train/src/fbs.rs` still keys every `CopyHeads::choose`
call on one token. The CPU audit can therefore accept copy-key-2 artifacts and
silently compute the old read. The unit stands.

## Files in scope

- `crates/uor-r4-cpu-train/src/fbs.rs`
- `crates/uor-r4-cpu-train/src/hamming.rs`
- `crates/uor-r4-cpu-train/src/served_forward.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-87/**`
- `docs/research/notebook/cpu-training.md`
- `docs/research/cpu-training-program.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/harness/claims.json`

## Done when

1. Copy key 1 keeps its current CPU query, key, candidate order and artifact
   behavior byte for byte.
2. Copy key 2 builds each copy-head query from `(x_t, x_{t-1})` and each key
   from `(x_{s-1}, x_{s-2})`, with the same low/high half order as serving.
3. A copy-key-2 synthetic artifact matches `IntegerStackSession` on every Q12
   query coordinate, token decision and kept-cell vector over a multi-step
   sequence.
4. A semantic control distinguishes two facts after the same last token and
   proves key 2 selects the ordered fact that key 1 ties.
5. The served-forward audit accepts the copy-key-2 artifact only with exact
   CPU/serving parity.
6. CPU all-target tests, strict Clippy, formatting and evidence checks pass.

## Stop and ask if

The integer and float halves disagree, or an exact copy-key-2 parity requires
changing serving semantics rather than completing the CPU port.
