# #1719 — Refine learned geometric completion with one native-accepted Q4 round

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T05:27:01Z
- **Updated:** 2026-10-09T13:17:31Z
- **Closed:** 2026-10-05T17:11:52Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1719
- **Merged:** 2026-10-05T17:11:51Z
- **Merge commit:** 9d779087550876cdf300b2eb5eec18198f1b4168
- **Base / Head:** main / UOR-Foundation:codex/geometric-source-end-refinement-20261005
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #1552 and #820.

The learned SourceEnd field improved completion but still lost at some boundaries against global Copy aliases. This adds one bounded native-accepted refinement: six fixed Q4 payloads, one full128 gradient to propose changes, deduplication, independent export/reload and strict finite native full-answer CE selection including EOS. It adds no runtime gate or optimizer dose.

Historical CPU Linux run at numerical commit5eca7aec: selected Stop-only proposal reduced native CE0.241255→0.218124, development completions92→96/128, matched transfer7→9/32, with zero membership losses. All six gains repaired missed EOS. Source selection and punctuation remain; complete transfer pairs0/16. Rejected-proposal generation NOT_RUN. These scoped results do not qualify general chat or subsequent supported-reader transfer.

Reconciled current main75998b91 at exact head599b45789305978331dec7f202ee90bb1f456f47. Refinement helper remains byte-identical to reviewed8a2b4480. A root-example compile defect inherited from main required60 pub(super)→pub(crate) substitutions in the example, preserving its numeric bodies and library API. Actual exact-head checks on the pod: touched-file formatting,5 source_end_ tests and release builds of geometric-bank-fit plus geometric-native-bank-observation PASS; wrapper exit0,196.208s,2.327GB child peak RSS,48CPU threads,noGPU. Metal unavailable and not counted; CI acknowledgements are not tests. Independent exact-head source/evidence review PASS.

The complete historical74-file result/panel set194596060B has a verified same-pod /workspace archive. Original results and audit retained; this is not an off-host backup. Live next attention work is record-occurrence credit, not another unchanged refinement dose.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5988674315 — 2026-10-05T05:28:30Z — Casey-allard

Exact-head review at8a2b4480: numericalexample/support byte-identical to compiled5eca7aec; only documentation added since measuredhead. Scopedformat/fivefocusedsource-endtests/releasePASS onCPU-onlyLinux, actualrunexit0. Independent savedoutcome205817checksPASS/0errors (auditJSONa6443b43), notencoder/tokenizer/backward/BLAKE3 rerun. Reconstructsproposal6→4dedup/quartergrid/payloads/nativeglobalaliasCE/strictselection/greedyties/answer membership; sameprefixCopy/carrier checks1122dev+276fresh. Selectedgradient-stop-q2 changes37Stop/0Period, CE.241255→.218124. Development92→96 andfresh7→9, zero membershiplosses; all6gains repairmissedEOS. Freshpairs0/16; notsourcebinding/chatqualification. Sourceadversarialreview approved exactnumericalhashes. Claimwording/diffcheckPASS. Report/rootarchiveSHAsetverified; samepodarchive notoffhostbackup. Onefinitecalibrationroundends; compilerreuse/nativelowering next. Self exact-head delivery review approved undercurrentownerpolicy.

### comment 5988704709 — 2026-10-05T05:31:39Z — Casey-allard

Delivery observation: auto-merge enabled but mergeQueueEntry null, stateOPEN/notmerged. Explicitenqueue at exact8a2b4480 refused: 5of5 requiredstatuschecks expected. Exactheadcheck-runs show allfive compatibilitynames success, combinedcommitstatus has no statuses. This is a protecteddelivery discrepancy, not test/modelnegative; no bypass/fabricatedchecks. ActualLinux tests+outcome evidence above remainsvalid. Leaveautomerge enabled and continueindependentcompilerintegration; reconcilepublisher/queue as boundeddeliveryfollowup, notanothermodelprogramme.


---
_Comment/review ids in this file: 5988674315, 5988704709_

