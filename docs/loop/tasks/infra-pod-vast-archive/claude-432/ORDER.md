# ORDER: infra-pod-vast-archive, claude-432

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `caa0f10c3` (claude-431, PR #4092). Asked for by Casey on 18 Nov ("Yes, fix it here"), after
claude-431's REPORT listed the failure under Still open.

## Owner approval

This unit changes the owner path `scripts/pod/uor-pod`. Asked in Claude's chat "Fix the uor-pod
Vast archive bug next?", Casey answered verbatim: "Yes, fix it here" (2026-11-18T22:34:45Z).

## Why

On 18 Nov `uor-pod down vast-61321505` refused to delete the pod after copying its results:

```
bash: line 2: uor-r4/pods: Is a directory
uor-pod: warning: cannot reach vast-61321505 over SSH to verify the laptop archive
uor-pod: vast-61321505: results not archived (see above), nothing deleted. ...
```

The pod's results had to be archived by hand and the pod deleted with `--confirm-archived`.

The cause is in `verify_laptop_archive` (`scripts/pod/uor-pod`). `vast_archive` passes it the
result directories as the pod listed them: one directory per line. The verifier puts that
string straight into the command it sends over SSH (`find $have -type f ...`). The newline ends
the remote command after the first directory, and the pod's shell runs `uor-r4/pods ...` as a
second command. The `tar` and `du` calls in `vast_archive` flatten the list with `$(echo $have)`;
the verifier does not.

It only shows when the pod holds two or more of `uor-r4/{jobs,pods,runs,evals}`. The test suite
covers only a pod with `jobs/` alone, so it never ran this case. Both `down` and the reaper's
archive go through this verifier.

## What it does

1. **The fix:** `verify_laptop_archive` puts the directory list on one line (`have=$(echo $3)`)
   before it builds either the remote or the local command. Nothing else changes.
2. **The test** (`scripts/pod/tests/uor-pod-dryrun.sh`): a Vast pod whose `/workspace` holds
   both `uor-r4/jobs/` and `uor-r4/pods/`, one file in each. `down` must archive, verify and
   destroy it. No output line may say "is a directory" (either case: the laptop's bash prints it
   in lower case). The pods file must be on the laptop, and the ledger must record a verified
   archive of 2 files.

## Gates

- **G1, the suite:** `bash scripts/pod/tests/uor-pod-dryrun.sh` ends `passed N, failed 0`
  (main: 376 passed; this branch adds 4 checks).
- **G2, the negative:** with the `uor-pod` change set aside against `origin/main`, the three
  new archive checks fail, and only those. Restore afterwards.
- **G3:** `bash -n scripts/pod/uor-pod` is clean, and `shellcheck -S warning` reports the same
  findings as on main.

No pod is rented for this unit: the fake `ssh` in the suite runs the archive commands for real
against a local folder, which is how the failure was reproduced.
