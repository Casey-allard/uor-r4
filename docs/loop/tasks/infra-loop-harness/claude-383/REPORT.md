# REPORT: infra-loop-harness, claude-383 (four branches with no PR, archived)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The patches are not under `archive/history/branch-archive/`.** No open item's scope covers that folder, so they sit in this unit's own folder, and `archive/history/branch-archive/INDEX.md` has no rows for them. A later archive unit moves them and adds the rows.
- **Nothing here checks whether a branch's content was already on `main` line by line.** Three of the four branches belong to units that are on `main`. Some of their files differ from `main` there, most likely from later edits on `main`. The patches keep each branch whole either way, so nothing depends on that question.

## What changed

Four patches in [`branch-archive/`](branch-archive/), one per branch, each the complete tracked diff `git diff --binary --full-index <base> <tip>`:

| Branch | Patch | Base | Tip | Tip tree | Patch SHA-256 | Bytes |
|---|---|---|---|---|---|---|
| `claude/ladder-pin` | [patch](branch-archive/claude_ladder-pin-20261115.patch) | `fbf15e56f3ab1961b30610893392e7a97fd0ab18` | `721a4784f265efea6b490d5c3100779e4c4b1607` | `abbff10fcd3a6133c85a250d903ff0c0575ed660` | `312a65adf9a823a3114a6f0c3a478cd044664eef6e6f1a5e0ef82a88b498ffe1` | 45,686 |
| `claude/evidence/writer-admission-074d54b2` | [patch](branch-archive/claude_evidence_writer-admission-074d54b2-20261115.patch) | `54be88b727ee5642fc4b9bfdc069874281d36a8a` | `8cb2ff9813515463665ab8c2c6269a0f976222c0` | `fb10e0d08e45f9c6d5e27f7e916ae9f3c2a3c20a` | `436aabc9b187e1844993710e3d8b87a43819d4176cbdbf3e894547e3050b0800` | 45,897 |
| `claude/route-in-loop-next` | [patch](branch-archive/claude_route-in-loop-next-20261115.patch) | `675f5eb43b99e5ee634cd45b5fd517f69a638c52` | `d4bcfe2c7dad146441602ac40a41dcecd9f2d6e1` | `1cef68ef7171bb77e1aabc6de80fd5abf0211039` | `d91f075b6407c11d536281b274c164450c2476072b0410533d4c6f359f712102` | 38,727 |
| `kimi/studio-final-acceptance` | [patch](branch-archive/kimi_studio-final-acceptance-20261115.patch) | `fa5004861d1bcad84f3e09da250cb290ddf70e3e` | `c17a8468220c9bf20998f5b8cef44828eec594bf` | `439d66430a458ae8f85505ec9a5c5c2a6d94ba3e` | `818d322c6f68978b5d99dcb240a331b374029d6196b734dd68d83b95dc79d4d6` | 111,061 |

To restore a branch's tree: `git checkout -b <b> <base> && git apply --index <patch> && git commit`. That gives the tip's tree, not its commit history.

Before this unit, two closed-PR branches were deleted under the standing branch-hygiene rule, because `refs/pull/<n>/head` still holds their commits: `claude/ladder` (#3908, tip `8895642dc60f8dc775be094aa84b05cdb80bc0ff`) and `kimi/public-record-repair` (#3325, tip `bf6a4efcf45554f61e203a7c693a5fd08a23a380`). Restore with `git push origin refs/pull/<n>/head:refs/heads/<b>`.

## Measurements

- Restore check: each patch, applied with `git apply --cached` to a scratch index read from its base, gives exactly its tip's tree (4 of 4 `EXACT`).
- Secret scan of the four patches (private-key headers, `ghp_`, `hf_`, `sk-`, `AKIA` patterns): no match.

## Checks run at the head

- The restore check and the secret scan, above.
- `uor-harness loop pr-check`: recorded on the PR.

## Skills used

None.

## Still open

- Move the four patches to `archive/history/branch-archive/` and add their INDEX rows, in a unit whose item covers that folder.
- Delete the four branches after this merges, with a lease on each tip, and post the #2028 note.
- Left alone at the owner's word: deepseek's `deepseek/place-read-audit` (#3937), `deepseek/e8-audit`, `deepseek/carry-hold-batch` (#3889) and the six `try-reply-*-v3` branches, plus kimi's open #3941.

## Plain summary

Four old branches with no pull request held work that existed nowhere else. Each is now saved as one patch file in the repository, checked to rebuild its branch exactly, so the branches can be deleted without losing anything.
