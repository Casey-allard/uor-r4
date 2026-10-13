# The owner's arithmetic ruling, and moving the lab data out of every checkout

**Question.** Two threads from one owner session: (1) does the remaining ambiguity in the
constraint set resolve, and is a ternary/XOR arithmetic a viable replacement for the
prohibited matmul; (2) can the labs stop duplicating ~1 GB of tracked material per
workspace, and can the machine get its disk back.

**Line:** no model line - governance and workspace. · count 0/3 unchanged · headline
37/232 → 37/232 (unchanged). No model compute was run, and none is admitted to any of it.

## 1. The ruling

Asked to resolve the last reading under which the GPU training lane might still use dense
GEMM, the owner answered: *"no matmul at all anywhere, same with softmax - there is a
ternary alternative with xor that may work."*

Recorded as a bullet in [D25](../../integration/DECISIONS.md#d25---geometric-operator-phase-and-current-lab-direction).
It adds no fourth restriction: D23 and D25 already prohibited softmax and dense matrix
multiplication in the executed path, and this closes the training-lane exemption. No
result that needs either counts as progress on any line.

## 2. The ternary/XOR assessment

[The brief](../mechanisms/ternary-xor-arithmetic.md) is in `docs/mechanisms/` and states
what is true, what is not, and what a fair test would need. Summary:

- **Ternary weights do remove the multiplier.** `w ∈ {−1,0,+1}` makes `w·x` a conditional
  add, subtract or skip. This is elementary and old.
- **They do not remove the matmul.** The contraction stays dense — every weight touched
  every token. D25's own words are that "replacing a library call with a handwritten dense
  matrix contraction is not a geometric replacement". So ternary answers D11's
  multiplier-free serving contract, not D25's geometric requirement, and it collides with
  OP2 rather than complementing it: both are attempts to remove those dense projections.
- **XOR is not the arithmetic of a ternary alphabet.** XOR is addition in GF(2), whose only
  coefficients are {0,1}: `−1` and `+1` collapse, and `0` is not a third value with a group
  law. What *is* exact: against a **binary** operand, `Σ wᵢxᵢ = popcount(P & X) − popcount(N & X)`
  over disjoint positive/negative weight bitplanes. The operators there are AND and
  subtract; XOR supplies a Hamming *distance*, not a dot product. We re-verified this
  independently before shipping the brief: 5,000 random trials, exact every time for a
  binary operand; for multi-bit operands it needs a per-bitplane sum, which is bitslicing
  rather than XOR.
- **It is a quantisation lever, not a geometric mechanism**, and the project already has a
  better-justified multiplier-free representation: residual icosian pairs with a 2-bit
  mantissa norm, measured at 1.5148 BPB against dense's 1.5136. A ternary weight is
  ~1.58 bits and is not in `Z[φ]`.
- **Repo reality, named so nobody misreads it:** `TernaryLinear` in `lowbit.rs` is exact and
  multiplier-free, but every use is post-hoc quantisation of a float master in offline
  diagnostic binaries. No training-crate code mentions ternary, and there is no
  popcount/bitsliced contraction or GPU kernel anywhere. Its existence is not an
  implemented XOR path.

It enters as a **proposal under the D25 rule freeze**, not a rung, and changes nothing
until the first compliant rung0 result reports. If pursued, the brief names the control
(icosian-mantissa at equal steps, width and weight bytes — not an FP16 dense control), the
bar, and the conditions that would make a result uninterpretable.

## 3. The workspace change

Measured before the change: a fresh worktree cost **1.3 GB with no build output**, because
git materialises a second full checkout and `research/` alone is ~1.0 GB of mostly binary
material — a 53 MB `.npz` (twice), 20 MB CSVs, 14–23 MB PDFs. Every worktree carried an
identical copy.

**Space reclaimed on this machine, 31 → 45 GiB free:**

| action | freed |
|---|---|
| `kimi-lab` Rust `target/` (caught between builds; it has since rebuilt) | 11.8 GB |
| `~/uor-r4-local/target` build cache | 994 MB |
| two worktree `target/` caches | 1.4 GB |
| `.graph-indexer/` orphan SQLite (no generator on the machine, no repo reference) | 364 MB |

**Then, with the owner's approval, the research material moved out of git:**

| archive | path | size | files | md5 |
|---|---|---|---|---|
| `prime-analysis` | `research/prime-analysis` | 566 MB | 7,713 | `37ed6837…` |
| `ai-research` | `research/ai-research` | 378 MB | 3,337 | `316f79c6…` |
| `archives` | `research/archives` | 103 MB | 129 | `7af2d66d…` |

999 MB, MD5-verified in the store at `icloud:UOR-R4/results/owner-docs/`, indexed at
`icloud:UOR-R4/index/`. **Nothing was deleted before restorability was proven**: we
restored `prime-analysis` from iCloud and compared it against the local copy — same 7,713
files, byte-identical content digest. The only difference is that the tar round trip
normalises two non-ASCII filenames in `research/theoretical-mathematics/prime-number-theory/`;
contents are unaffected. That check is recorded in
[docs/harness/research-data.json](../../harness/research-data.json).

New tools: `scripts/storage/research-data` (status / manifest / restore, read-only by
default) and `scripts/storage/worktree-audit` (what every workspace costs, whether it is
removable, and the `format-patch` command to preserve anything unique first).

## 4. One checkout, and the guard that makes it safe

Worktrees were never the goal; isolation was, to stop two labs corrupting each other. With
the binary material archived a worktree costs ~330 MB instead of 1.3 GB, and one checkout
carries no duplicate at all.

[docs/labs/working-in-one-checkout.md](../../labs/working-in-one-checkout.md) sets out the
rules: one branch has one owner at a time, keep the tree clean between sessions, serialise
`cargo` builds, and use a worktree only when two branches are genuinely needed at once.
The guard is real, not advisory:

```
python3 scripts/harness/cli.py team claim --branch <name> --lab <lab>     # refuses a second owner
python3 scripts/harness/cli.py team claims                                # who holds what
python3 scripts/harness/cli.py check                                      # warns on an unclaimed branch
```

`status.branch_claimed` warns on every `check` when the checked-out branch is held by no
lab. The claims file is `.harness/claims.json`, deliberately gitignored: git cannot store
state about the worktree it is being committed from, and committing it would create the
churn it exists to prevent.

## 5. Also in this unit

- **M5–M8 are in the harness registry** (from issues #2033–#2036, with their carried-over
  acceptance and dependency text), and the registry now states the real statuses —
  M2 blocked on owner, M3 paused, M4 blocked on owner — instead of a blanket `in-progress`.
- Their panels exist and are deliberately **not frozen**, because an unstarted milestone has
  no pre-registered acceptance to encode. `prep.panel_frozen` was taught the difference: it
  is an error for a started milestone to lack a frozen panel and a warning for an unstarted
  one. The exemption is itself asserted by a test, so it cannot be used to smuggle in an
  incomplete frozen panel.

## Evidence

```
python3 -m unittest discover -s tests/harness -t .   ->  227 tests, OK
python3 scripts/harness/cli.py check                 ->  34 rules, 0 error, 8 warn
python3 scripts/check_claim_wording.py               ->  passed
python3 scripts/check_no_softmax.py                  ->  OK: 1649 remaining, baseline 1649
scripts/storage/research-data status                 ->  999 MB archived, 3/3 restorable
```

## Limitations

- D25's rule freeze still governs: this unit records a ruling and moves data; it adopts no
  mechanism, lifts no training hold, and rents nothing.
- The ternary/XOR brief is a design assessment. It contains no measurement of this project's
  own, and the literature numbers it cites are other people's, at their scopes.
- The archived bytes still exist in this repository's git history; they are no longer
  checked out. This is a working-copy change, not a history rewrite, so it is reversible.
- Worktrees owned by other labs were not touched. `worktree-audit` reports them; their
  owners decide.

**Next:** two labs are mid-work (kimi on OP1, codex reconciling instructions) and hold
worktrees with uncommitted files; each should commit and switch into the single checkout
when its unit lands. `claude-starling3` is clean and fully in `origin/main` and is
removable by its owner. The ternary/XOR proposal needs an owner decision on whether to
carry it as a labelled non-geometric arithmetic substitution, or to pursue the geometric
version the brief describes (the three alphabet values as actual group elements rather than
a truncation).
