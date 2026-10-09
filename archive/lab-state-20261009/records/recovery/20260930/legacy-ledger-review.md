# Independent legacy ledger migration review

Decision: **APPROVE for explicit preservation migration only**.

Reviewer: `/root/second_council_review`, an independent non-author review pass launched by the Codex parent session. Provider and GitHub account independence are not claimed. The reviewer read the preserved bytes and recomputed arithmetic/hashes; no source edits, Cargo, GitHub writes, or external-volume operations were performed.

Approved preservation baseline:

- `cumulative_ms`: **790330180**
- `limit_ms`: **1130000000**
- Covered legacy charge/extension records: **15**, listed with independently verified hashes below.
- All 16 files in the recovery manifest match its sizes and SHA-256 values.

The reconstruction supports `768447701 + 2495630 + 6844600 + 4750250 = 782538181` ms. Eight contiguous Codex Track B charges add `7066990` ms; volume reconciliation adds `725009` ms, yielding `790330180` ms. Each later charge equals its elapsed interval and joins the preceding cumulative balance. The recorded limit chain is `744000000 + 12000000 + 24000000 + 40000000 + 310000000 = 1130000000` ms.

Preserve `model-time.json` (SHA-256 `29f3adde40b5efcd5cb3936e14a05611ed4e04f79f25dd5d45e54dddeb48cb4f`) and `recovery-manifest.json` (SHA-256 `0e19f9b09c5e493b20ad4c9f3895cb4f884af86b2806eb1ebf2359f9508302fe`) as corroborating snapshot/inventory. Neither is an additional charge or a covered charge/extension record.

The Stage 1 catch-up of 2495630 ms and its reconstruction are overlapping records of the same restored charge. Earlier snapshots and charges are subsumed by the reconstructed historical balance, not added again. Similarly, the 48-hour extension restores the prior 820000000 ms limit before adding 310000000 ms; do not count the restored 40000000 ms twice.

## Historical limitations and outstanding reconciliation

- This is not complete historical accounting from zero. The initial 768447701 ms reconstruction base and original sealed report durations were not independently verified here; the supplied earlier records have gaps and a documented stale overwrite.
- Stage 1 listed seconds sum to 2495646 ms, whereas its declared charge, balance change, and reconstruction use 2495630 ms: a 16 ms discrepancy. Preserve the recorded amount; use a separate evidence-bound correction if warranted.
- Rounded reconstruction prose sums Lab 2 durations to 4750300 ms, while its numeric contribution is 4750250 ms: a 50 ms difference that may be rounding. Preserve the numeric contribution pending evidence.
- The reconstruction explicitly excludes Lab 2 failed attempts. Those still require identification and reconciliation.
- There is a 68010 ms gap between Track B checkpoint 8 ending at 2026-09-29T22:09:28.990Z and volume reconciliation starting at 22:10:37Z. It may be idle time; do not automatically charge it without activity evidence.
- Subsequent recovery, preparation, review, builds, and other labs' later work are not established as included. Append outstanding identified costs with overlap checks rather than treating this baseline as complete current accounting.
- This review does not approve autonomous admission, production deployment, model qualification, or PR source delivery. Future updates must preserve originals and use immutable typed charges or evidence-bound corrections.

## Covered records

```json
[
  {
    "file": "charge-2026-09-28-lab1-s2.json",
    "sha256": "613bde72819a342dfd870e6d5eabd666d6eeadeb0ac0af3b034d1d0b8d062010"
  },
  {
    "file": "charge-2026-09-29-codex-track-b-checkpoint-1.json",
    "sha256": "068bfc4232d682ac9134646a37b4855d1cb74576adf294777b0355fb88337a32"
  },
  {
    "file": "charge-2026-09-29-codex-track-b-checkpoint-2.json",
    "sha256": "c863c4055b28b22fb24476bc3af7c1a6f23e6c50e700f4033ba5a9b1ba5f724a"
  },
  {
    "file": "charge-2026-09-29-codex-track-b-checkpoint-3.json",
    "sha256": "427e474610658bf5d8caa777808d6535f4e8a9df58867d50f72c62361ea5bf12"
  },
  {
    "file": "charge-2026-09-29-codex-track-b-checkpoint-4.json",
    "sha256": "a9569a1880d7b55bbaad6e72beb484602eb5ad73d0d7d029816c72d69e7a1ba9"
  },
  {
    "file": "charge-2026-09-29-codex-track-b-checkpoint-5.json",
    "sha256": "b31413966d0c0e51028aea41cf1a3cf52f9c81b03da755f9a5b404ec716e4372"
  },
  {
    "file": "charge-2026-09-29-codex-track-b-checkpoint-6.json",
    "sha256": "9b0e49013a9281cf0a05c497f5915e0815936f94540fb2099092d993891f71bf"
  },
  {
    "file": "charge-2026-09-29-codex-track-b-checkpoint-7.json",
    "sha256": "2e161a1f869ed33061be38906de393815c2b04b3df02718fb9f5e70b2f547724"
  },
  {
    "file": "charge-2026-09-29-codex-track-b-checkpoint-8.json",
    "sha256": "e53b87eaf8ed9e2e2876d71233abcc1ddfac20ce80d041f6982c9f95357c5e15"
  },
  {
    "file": "charge-2026-09-29-codex-volume-reconciliation-1.json",
    "sha256": "878a66a7e9a40f4bbbf2fd3d2533c6e8287a4376d0c930c293c7bdda79d19e85"
  },
  {
    "file": "charge-2026-09-29-lab1-reconstruct.json",
    "sha256": "c8e4a139389cbcde19d04453f51ed746e455c5c7cd06e7f1c8603d11f317c11e"
  },
  {
    "file": "extension-2026-09-27-read-localization.json",
    "sha256": "40b326fba2a625843ec7331c21bd6a66c3b4ecb290cf022a00b8ada188271116"
  },
  {
    "file": "extension-2026-09-28-lab1-s2-qat.json",
    "sha256": "9bd98f0758db3778b977e9eefe0ece6d7c501826aed7206d05ab62aeeed4f2fb"
  },
  {
    "file": "extension-2026-09-29-lab1-48h.json",
    "sha256": "5e587ad78f7c6a71254c7e9758a8257e88e0c6fc95ba1741e01694d12bf15bc8"
  },
  {
    "file": "extension-2026-09-29-lab1-stage1.json",
    "sha256": "7c91c19616dc3b339c9f80c565a412f6ef82688ac996cc844d3bb647581b1657"
  }
]
```
