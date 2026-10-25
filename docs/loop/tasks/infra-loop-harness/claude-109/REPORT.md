# REPORT: infra-loop-harness, claude-109

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2904. References #2836.

## What did not work, first

- **The harness offered a unit its own item said was not warranted.** Stage T's warrant clause ("only warranted if W2 blames the learning law") was prose. `loop next` reads only `depends_on` and status, so it ranked Stage T first after W2 had ruled H_law out. claude-107 claimed it, held the discrete crate for about an hour, and then cancelled at its cancel probe. This edit puts the decision where the harness reads it.
- **A candidate harness change, for the harness's owner:** an item could carry a machine-readable `warrant:` (for example a required W2 verdict) that `loop next` checks. This is not done here, because the harness's own files are outside this unit.

## What changed

- **`docs/roadmap/items/m1-bvg-k-stage-t-training.md`:**
  - `depends_on` gains `m1-address-key-full-vector`;
  - one note gives the reason (claude-107's D68 card) and the instrument work owed before Stage T's arms can run.
- **`docs/roadmap/README.md`:** re-rendered by `roadmap render` (+2/−1).
- **`PLAN.md:61`,** Stage T's "Needs" cell. It read "steps 1 and 3". It now lists the item's real `depends_on`:
  - steps 1, 3 and 5;
  - `m1-w2-code-utilization` (#3179);
  - `m1-address-key-full-vector` (#2834).

  The A8 and code-utilization dependencies were added by claude-91/-92 (#3185) without a PLAN.md edit, so the row had already drifted (L-001). Codex's open #3169 also edits PLAN.md, but only lines 21–27, so the two do not overlap.

## Measurements

None. This is a records unit, so there is no run, no GPU and no model.

## Checks run at the head

- `cargo run -p xtask -- roadmap check`: 92 items OK.
- `loop report --check` and `loop pr-check --local`: PASS (the review-record and required-checks rules report N/A in a local run).
- **A fresh exact-head review** of this diff, together with round 2 of claude-107's decision review: APPROVE, no Required findings. It checked that there is no cycle (#2834 depends only on `m1-w2-capacity-capture`) and that the render matches, and it re-ran `roadmap check`. Its advisory on the stale PLAN.md row is taken; see What changed.

## Skills used

- scope-guard: the edit is the one line that carries out the decision, plus its note.

## Still open

- **After the merge,** `loop next` must not offer Stage T while #2834 is open. The next `loop next` in this session checks it.
- **#2834** is the next M1 mechanism unit.

## Plain summary

The plan's next training step now waits, in the project's task list, for the address fix it depends on, so no lab starts it too early.
