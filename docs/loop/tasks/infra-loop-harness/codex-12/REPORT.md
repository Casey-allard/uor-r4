# REPORT for the three permanent branches scope registration

Lab: codex. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). [ORDER](ORDER.md).

## What did not work

The item allowed the charter and its history to change, but left the current publisher, branch registry and operational tests outside the registered file list. A change to those files needs a scope visible on the merge base.

## What changed

I approved ten named paths needed to implement Casey’s permanent branches decision. The registration covers the two workflow files, three harness modules, registry, workspace instructions, Studio publisher and its regression tests. The implementation follows as a separate reviewed change.

## Measurements

The original 63 entries are intact. Ten additional paths bring the list to 73. The harness manual is already a shared path under the claim policy and requires no item-specific registration.

## Checks run at the head

Compared the old and new parsed lists. The old list remains an identical prefix and the ten named files are the only additions. All other lines of the item remain byte-identical. The staged patch passes the whitespace check. Verification uses the current xtask roadmap check and the base-built harness’s loop pr-check with explicit base, head and head-ref.

## Skills used

The repository scope check and independent Git comparisons.

## Still open

The implementation must test both publishing and branch retention against this registration once it reaches main.

## Plain summary

This change registers the files needed to switch the documentation publisher and protect the three requested branch names.
