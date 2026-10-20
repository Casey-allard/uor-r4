# Post-teardown source-workspace disk deviation

After owned timing2 pod deletion at02:35:07Z, creation of the owned docs-delivery
worktree briefly materialized the full tracked checkout. The first sparse
reapply left paths whose index stat state needed refresh. The observed local
free space was31,315,980KiB, below the31,457,280KiB floor. No new lease or model
work was started during the deviation.

Parent immediately inspected status (only its claims.json was modified),
refreshed the index and reapplied the declared sparse checkout. That removed
only redundant pristine tracked material. Worktree size became1,768KiB and free
space recovered to31,808,948KiB. Owner checkout and peer work were untouched;
unique source/evidence stayed intact. Future creation must initialize the index
before sparse materialization and avoid a full worktree restore; fresh df remains
mandatory before any lease. This is an operational deviation, not model evidence.
