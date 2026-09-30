# Delivery gate deployment and administrator configuration

The checked-in workflow is an implementation, not evidence that a required
server gate is installed. Until the administrator completes and verifies this
runbook, report **coordinator enforcement; server enforcement unverified**.
The existing five required CI names remain compatibility acknowledgements and
receive no credit as executed tests. The local coordinator commands are
`lab-runner delivery check <receipt.json>` and
`lab-runner delivery enqueue <receipt.json>`; neither installs a GitHub ruleset.

## Trusted workflow and its boundary

[`lab-delivery.yml`](../../.github/workflows/lab-delivery.yml) publishes the exact
commit status **`mission-delivery-gate`**. Its job display name is different;
require the commit status above, after observing its producer on a real run.
It validates immutable source identities, current task ownership and hashes of
recorded local execution/review evidence. A successful status means those
records passed validation. It does not mean the GitHub runner executed the
model, reran Cargo or independently established the truth of a log.

The workflow runs trusted default-branch code through `pull_request_target`,
manual dispatch restricted to `main`, and `workflow_run` after the existing
`ci` workflow receives a `merge_group` event. It never checks out a PR, executes
receipt commands, imports PR modules, downloads workflow artifacts, or restores
PR caches. The action dependency is pinned to a full commit. Its token has
read access for content/PRs/issues/actions and write access only to commit
statuses. These choices follow GitHub's guidance for
[`pull_request_target`](https://docs.github.com/en/actions/reference/security/securely-using-pull_request_target)
and [privileged workflow events](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows).

This is a cooperative evidence gate. A shared GitHub account cannot prove that
two named sessions were independent, and a hash cannot prove a claimed command
ran. A required status restricted to the GitHub Actions App also does not
isolate this publisher from a different repository workflow permitted to write
statuses. Preserve independent recorded review and restrict workflow changes.
For stronger server authority, use an organization-required trusted workflow
where supported, or a dedicated GitHub App publisher whose credentials are
unavailable to PR execution; then require its observed App identity. Do not
claim cryptographic reviewer independence or malicious-writer resistance from
the current shared-account design. Review GitHub's
[secure-use guidance](https://docs.github.com/en/actions/reference/security/secure-use)
before changing this trust boundary.

## Publish evidence after the source head is fixed

Evidence cannot be committed into the source PR while simultaneously naming
that same commit: the commit would change its own hash. Keep the source head
fixed and publish its receipt/evidence on `codex/lab-state`. The workflow reads
one pinned operational commit for all envelope evidence, then rereads the
current operational claim and GitHub server time before publishing a status.

The ordinary-PR path is `delivery/pr-<number>/<full-head-sha>.json`. The queue
path is `delivery/merge-groups/<full-synthetic-sha>.json`. Both have this envelope:

```json
{
  "schema": "uor-r4.server-delivery/1",
  "receipt": { "schema": "uor-r4.delivery-receipt/1" },
  "evidence": [
    {
      "local_path": "/absolute/path/to/local-check.log",
      "path": "delivery/evidence/<sha256>.log",
      "sha256": "<64-lowercase-hex-digest>"
    }
  ]
}
```

This abbreviated example is not an eligible receipt. `receipt` must contain
every required field in
[`DeliveryReceipt`](../../tools/lab-runner/src/delivery.rs): repository, PR,
task issue, author session, current `claim_session`, nonzero `claim_epoch`,
`work_card`, exact head/base SHAs, change kind/class, checks, reviews and any
required result/council evidence. The task must remain claimed, unexpired and
owned by that generation; its steward must be available with a fresh heartbeat.
Restewardship is explicit: a successor revalidates the unchanged source evidence
and binds a new receipt to its current claim. An expired author cannot use an
old receipt to deliver a successor's task.

Every `FileEvidence.path` in the local receipt maps to exactly one envelope
`local_path`, and its digest must match the published bytes. Publish only
appropriate nonsecret logs/review records. Files must be nonempty and at most
1 MiB each for this server API; local receipts allow larger evidence, which
must be represented by a deliberately produced bounded evidence record before
server delivery. Such a record must explain omitted material and bind the
retained full artifact; do not relabel a summary as a full log. Paths on the
operational branch must be under `delivery/`, without `..` components.

Required execution records are `diff-check` and `claim-wording` for documentation;
`diff-check`, `format`, `compile`, `focused-tests` for code; and additionally
`loaded-behavior` for model changes. Each names actual argv, exit status zero,
`PASS`, head/base and a hashed nonempty log. Choose meaningful commands for the
changed boundary. A fabricated `PASS`, an acknowledgement, or a conditional
test that skipped required evidence is not execution evidence. Source paths
cannot label themselves documentation; core model/serving path changes require
model evidence. Shared policy and workflow changes require class C council
approval. Class B/model results require an independent named result reader;
class C requires three distinct decision passes, at least two non-author
participants, two approvals and written reasons. Native GitHub `blockedBy`
dependencies must be closed; incomplete dependency pagination fails closed.

The first version does not provide a receipt-publication CLI. Use a dedicated
operational checkout and normal Git fast-forward publication:

1. Fetch `codex/lab-state` and record its exact head. Prepare the new files atop
   that head without changing `state.json`, prior events or prior evidence.
2. Stage only the intended `delivery/` files. Before committing, verify that no
   other paths are staged. Make one commit with that observed head as its sole
   parent, then push it normally to `refs/heads/codex/lab-state`.
3. If another writer won, preserve the proposed files, fetch its new head and
   reapply only the new delivery files to a fresh checkout of that head. Check
   existing paths: identical content is idempotent; conflicting content requires
   explicit reconciliation. Commit with one parent and retry the normal push.
   Never force-push, merge two operational histories, or restore an old state
   snapshot over a concurrent heartbeat. Never reset the owner's checkout.
4. After an uncertain push result, fetch and compare the remote files and their
   digests before retrying. Once confirmed, manually dispatch `Lab delivery
   evidence` on `main` with the PR number if its initial run found no receipt.

Receipt publishers and `coord apply` share the same remote serialization point:
the non-forced fast-forward ref update. A local lock alone cannot serialize
other machines. This protocol is conditional on every writer obeying it;
server branch protection below prevents destructive ref replacement but does
not validate every JSON transition.

## Merge-queue integration

Version 1 supports **one PR per merge group**. Set both
`max_entries_to_build = 1` and `max_entries_to_merge = 1` in the main merge-queue
rule. A larger group fails closed, because an individual review cannot stand
in for validation of several interacting PRs. The queue's synthetic commit
must have the reviewed main base as its first parent and the same tree as the
current `refs/pull/<number>/merge` candidate. GitHub describes the synthetic
candidate and required integration checks in its
[merge-queue documentation](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue).

After ordinary PR evidence passes, use the local coordinator to enqueue the
exact approved head with `--match-head-commit`. Observe the actual queue SHA,
reserve the required local resources, and execute a meaningful focused check
of that exact synthetic candidate. Add the following to the queue envelope,
preserving the original source receipt and all its evidence mappings:

```json
{
  "integration": {
    "head_sha": "<actual-queue-sha>",
    "base_sha": "<reviewed-main-base-sha>",
    "pull_request": 123,
    "pr_head_sha": "<reviewed-pr-head-sha>",
    "checks": [
      {
        "name": "integration-check",
        "head_sha": "<actual-queue-sha>",
        "base_sha": "<reviewed-main-base-sha>",
        "argv": ["<actual-executable>", "<actual-argument>"],
        "exit_code": 0,
        "outcome": "PASS",
        "log": { "path": "/absolute/integration.log", "sha256": "<digest>" }
      }
    ]
  }
}
```

The source receipt cannot substitute for this integration record. The gate
waits at most 20 minutes for a missing group envelope; this wait does not grant
compute or extend a task lease. Maintain the five-minute steward heartbeat.
Missing, failed, unknown or stale records cannot pass. A long check can finish
within its authorized limits and be followed by a manual dispatch with the
same still-live queue SHA. If GitHub rebuilds the group or the base changes,
refresh the relevant reviews/checks and publish the new exact identities.
Do not copy the old success to a new SHA. The data-only workflow does not wake
or run the local coordinator; an active lab must perform this integration step
until a tested scheduler exists. Without one, the merge remains blocked.

## Administrator activation and acceptance

Perform these steps with repository administration authority. Do not bypass
current protection to bootstrap this workflow, and do not require its status
before it exists on trusted `main`.

1. Deliver the workflow and policy through the existing protected queue using
   reviewed local evidence. Confirm the exact workflow source on `main`.
2. Create/reuse an ordinary test PR and a current task claim. Record a missing
   receipt failure, then valid evidence success. Repeat with a changed head,
   changed base, expired/changed task generation, self-review, unresolved review
   fix, changed log bytes and `UNAVAILABLE` test outcome. Each must fail on the
   intended exact commit. Verify no PR-controlled code executes in this job.
3. Observe and record the status producer/App ID, workflow run URL and exact
   context `mission-delivery-gate`. Add that context to the ruleset protecting
   `main`, selecting the observed trusted App when supported. Keep the merge
   queue and existing required contexts, direct-push restrictions and no lab
   bypass. Set queue build/merge maxima to one. Do not infer enforcement from
   editing workflow YAML or a green unrelated job.
4. Protect `codex/lab-state` from force pushes and deletion and require linear
   history. Permit the authorized writers' normal fast-forward updates without
   a PR requirement, so each heartbeat remains an atomic operation. Do not
   require main's build/merge-queue checks on this operational branch. Preserve
   its append history and periodically verify single-parent transitions and
   immutable old records; server linear-history protection does not validate
   JSON schemas or immutable file semantics.
5. Exercise a real single-PR queue candidate. First verify missing integration
   evidence blocks it; then execute/publish exact-group checks and observe
   success and merge. Rebuilding the group must invalidate old evidence. A
   multiple-PR group must fail, including if someone later relaxes queue size.
6. Record ruleset IDs/revisions, observed App, passing and failing run URLs,
   exact PR/group/merge SHAs and main source/tree comparison in the rollout
   issue. Only then mark that server capability verified. Enqueueing is not
   merging; keep the issue open until its acceptance and delivered tree are
   observed.

The op-ref, GitHub issue graph and merge queue are separate transactions. A
freshness check immediately before publication/enqueue narrows races but does
not make them one atomic database. Revalidate after changes, retain fencing
epochs, and reconcile rather than claiming an impossible all-service lock.
If the workflow, API, adapter or administrator is unavailable, retain bounded
coordinator work and report the actual blocker; never manufacture the required
status to clear the queue.
