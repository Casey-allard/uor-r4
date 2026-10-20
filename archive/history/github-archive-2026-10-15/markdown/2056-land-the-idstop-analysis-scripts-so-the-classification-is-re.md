# #2056 — Land the idstop analysis scripts so the classification is reproducible from main (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T16:50:12Z
- **Updated:** 2026-10-09T16:52:48Z
- **Closed:** 2026-10-09T16:52:11Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2056
- **Merged:** 2026-10-09T16:52:11Z
- **Merge commit:** 389a7fc4b1958799a04a12ce3ddc6e4da0e0471e
- **Base / Head:** main / UOR-Foundation:deepseek/idstop3-scripts
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Land the idstop analysis scripts so the classification is reproducible from main

PR #2054 landed the probe driver, which makes the 72/84 MEASUREMENT repeatable from main.
It did not land the analysis that turns those records into the classification, so the RESULT was
still unreproducible. These four scripts close that gap.

  scripts/idstop-score.py            the classifier: check / classify / simcheck / report / reasons
  scripts/idstop-crosschecks.py      independent cross-checks on the classified cells
  scripts/idstop-turn1check.py       first-turn checks
  scripts/idstop-compare-records.py  field-by-field comparison of two record sets (used for the
                                     default-unchanged proof: 288 records, 9/9 files, 0 differing fields)

All four came from the EU-RO-1 volume (bindprobe/idstop3/ and bindprobe/idstop/) and are named here
with an idstop- prefix to avoid colliding with unrelated helpers already in scripts/. Each compiles
(python3 -m py_compile). Compiled __pycache__ output is deliberately NOT included.

The evidence file gains a REPRODUCING section that chains instrument -> scripts -> numbers, so a
successor can go from main to the 72/84 classification without the author's scratch layout. It also
records a trap that is easy to misread as a broken tree: cargo in the owner's checkout fails with
'Not a directory (os error 20)' because ~/uor-r4/target is a symlink to an unmounted volume; the fix
is CARGO_TARGET_DIR=/tmp/... .

LIMITATION, STATED RATHER THAN HIDDEN: score.py and friends read the sealed rows and grid output
from the EU-RO-1 volume (rfsx702p68 = uor-shared-EU-RO-1, canonical:false); they are not standalone on
a laptop without that data. The volume paths are recorded in the headers and in the evidence file.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

