# #2098 — Archive the criterion-2 retrain pair to iCloud (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T22:07:47Z
- **Updated:** 2026-10-09T22:11:48Z
- **Closed:** 2026-10-09T22:09:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2098
- **Merged:** 2026-10-09T22:09:32Z
- **Merge commit:** db8568da2647403dd1ec36c5ac89df0aeb189d11
- **Base / Head:** main / UOR-Foundation:deepseek/criterion2-pin-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to #2096 (merged as 3316080db), as directed.

The artifact measured by the protocol pin — the **only surviving artifact behind the published float cell 0.87755** and the round-23 quantiser table — existed in exactly one place, the non-canonical EU-RO-1 volume `rfsx702p68`, and in no cloud-store index entry. It is now archived:

```text
cloud-store put deepseek c2p-retrain-pair-20261009
-> stored c2p-retrain-pair-20261009 (91700736 bytes, md5 df290deec27236217cb4b4177b4d28d8)
   at icloud:UOR-R4/results/deepseek/c2p-retrain-pair-20261009.tar
index row: c2p-retrain-pair-20261009  91700736  df290deec27236217cb4b4177b4d28d8  ...  2026-10-09T22:01:15Z  deepseek
```

Contents: `model-a1/model/model.safetensors` (`cbef6906…`, 79,726,120 B), `export-a1/model.lut` (`721d4bdb…`, 11,891,652 B) and the provenance set (`attempt.json`, `report.json`, `config.json`, `export.json`, `snap.json`, `d11-a1/evaluation.json`).

**Round trip tested, not assumed:** `cloud-store fetch c2p-retrain-pair-20261009 <scratch>` returned `md5 verified`, and the extracted `model.safetensors` and `model.lut` re-hashed to `cbef6906…` and `721d4bdb…` exactly. The local copy was then deleted, so exactly one durable copy remains.

This PR names the object in the record (`docs/labs/criterion2-protocol-pin-2026-10-09/README.md` limitations 4 + Evidence, and the evidence transcript) and in the newest current-state entry, so the artifact behind the published float cell is reachable rather than just cited by sha256. The `Next:` line now carries the criterion-2 protocol restatement, with preservation removed from it.

No measurement changed. Read-only fetch was authorised from the Lead's already-running `reply-cap96` pod on the same volume (a new pod was refused by the rate cap); **bash extra spend**, no training, no export, no quantiser tuning, no float generation. `python3 scripts/check_claim_wording.py` passes.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

