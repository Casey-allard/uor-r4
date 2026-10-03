# Value-faith panels (2026-10-02) — preserved source

Twelve 10-row probe panels from the value-faith investigation. Their run results
are already retained (five `value-faith-*` objects in the iCloud result store);
these are the **panels themselves**, which were never published. Until now the
repo held only `value-faith-oracle-20261002.json` and its `.md`.

## Two pairs are byte-identical aliases — recorded, not silently deduplicated

The source files on disk were hashed during delivery and two pairs collide
exactly. They are preserved as found, because deleting one of an aliased pair
would lose the fact that the two condition names pointed at the same rows:

| sha256 (first 12) | files |
|---|---|
| `364ea161032a` | `value-faith-A-original-20261002.json`, `value-faith-parent-20261002.json` |
| `80985a62f96e` | `value-faith-B-no-distractor-20261002.json`, `value-faith-F-adjacent-20261002.json` |

**Consequence for any reading of these results:** `A-original` and `parent` are
one condition under two names, and `B-no-distractor` and `F-adjacent` are one
condition under two names. A result table that reports both members of a pair as
independent conditions is double-counting, and "F-adjacent" cannot be contrasted
against "B-no-distractor" because they are the same 10 rows. This was not
recorded anywhere before; it is the reason this note exists.

## What these panels are, and are not

Each row is a 10-row authored memory probe: a fact is stated and then queried,
with the distractor varied by condition. All results derived from them are
**phrase/value-presence** measurements — whether the expected value appears in
the reply — **not complete-answer membership**. See the claim reconciliation on
#820 for the distinction and for why the 14/120 / 31/120 / 9/120 figures must be
quoted with their artifact and scoring rule.

## Panel inventory

| file | rows | note |
|---|---:|---|
| `value-faith-A-original` | 10 | alias of `parent` |
| `value-faith-B-no-distractor` | 10 | alias of `F-adjacent` |
| `value-faith-C-distractor-first` | 10 | |
| `value-faith-D-distractor-not-an-assignment` | 10 | |
| `value-faith-E-distractor-unrelated-relation` | 10 | |
| `value-faith-F-adjacent` | 10 | alias of `B-no-distractor` |
| `value-faith-G-reasserted` | 10 | |
| `value-faith-H-two-relations` | 10 | |
| `value-faith-control` | 10 | |
| `value-faith-heldout` | 10 | |
| `value-faith-oracle` | 10 | already on main |
| `value-faith-parent` | 10 | alias of `A-original` |

References #1512
