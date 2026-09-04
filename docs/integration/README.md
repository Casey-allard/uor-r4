# Integration archive and optional tools

Current work follows the [native geometric AI plan](project-track.md),
[current implementation](current-state.md), and [execution policy](agent-execution-policy.md).
This directory also preserves dated research/integration inventories. Their
snapshots and “next” actions are historical; live GitHub owns issue status.

Core prime/zeta/R4/UOR architecture is primary. External research, source
indexes and review tools may answer a concrete implementation question without
becoming a serial proof or bookkeeping requirement. Existing Python knowledge
index tooling is optional repository tooling, not the native model lifecycle.

| Question | Source |
|---|---|
| What are we building and why? | [Canonical project plan](project-track.md) |
| What is implemented and next? | [Current state](current-state.md), [continuation prompt](CONTINUE.md) |
| What bounds execution? | [Policy](agent-execution-policy.md), [machine invariants](agent-execution-policy.json) |
| What was discovered in the prior audit? | [Historical reconciliation](roadmap-reconciliation.md), [dated snapshot](roadmap-state.json), [source catalog](source-catalog.json) |
| What components might be reused? | [UOR source audit](uor-source-audit.md), [external-source review](external-research-audit.md), [historical frontend port plan](frontend-port-plan.md) |
| How do I use the native CLI/workbench? | [Native workflow](../native_geometric_workflow.md), [current implementation](current-state.md) |
| What earlier workbench source is preserved? | [#1107 historical candidate](../r4_workbench_candidate_1107.md), [static review](../r4_workbench_candidate_1107_review.md) |

## Query the actual local index

`uor-knowledge` is installed as an isolated Python tool; its source is in [tools/uor-knowledge](../../tools/uor-knowledge/README.md). The MCP registration is `uor_knowledge` and exposes `search_knowledge`, `get_source`, `related_sources` and `knowledge_status`. The importer is CLI-only. The default database is `~/.local/share/uor-r4/knowledge/knowledge.sqlite3`; `UOR_KNOWLEDGE_DB` can select a different local index.

```bash
uor-knowledge status
uor-knowledge search '1079 token'
uor-knowledge search 'kappa composition'
uor-knowledge search 'memory_dimension'
uor-knowledge search 'Antigravity' --scope private
```

Search uses literal AND-connected terms, not a natural-language query planner. Fetch the returned source ID for the original excerpt and provenance. Ask the agent to follow native dependency edges and check live GitHub before answering “what can we execute now?” Public scope is default; private records/relationships require explicit private/all scope. Scope filtering prevents accidental disclosure through the default API, but is not a multi-user authentication boundary.

The durable acquisition cache on this workstation is `~/.local/share/uor-r4/knowledge/audits/2026-09-03`. It contains source manifests, raw native issue snapshots, explicit audit limits and restricted records. It is not part of the public repository. The six imported Antigravity documents are historical context; no full binary conversation decoding or Gemini cloud export was performed.

## Refresh and ingest when the active task needs it

The commands below document optional index maintenance. Run it only when
current source retrieval materially helps the active implementation or the
owner requests a refresh; do not update the claim ledger merely to record that
maintenance.

The builder validates pinned Git/source hashes and emits separate public/private JSONL files. It does not fetch new source: refresh the audit inputs first when a new snapshot is needed. It records the new origin/revision/content identity, preserving prior records.

```bash
python3 scripts/build_project_knowledge_seed.py \
  --audit-root /path/to/audit \
  --project-root /path/to/uor-r4 \
  --output-root /path/to/local/import
uor-knowledge ingest /path/to/local/import/public.jsonl
uor-knowledge ingest /path/to/local/import/private.jsonl
```

Only pass `--antigravity-brain /path/to/brain` when importing that authorized local history. This audit adapter selects the six known project documents across two task folders; it is not a universal provider-history decoder. Each import receipt lists the actual document paths and digests. Private files and raw exports should stay outside the repository.

## Refresh live public GitHub and the accepted plan

```bash
python3 scripts/sync_project_knowledge.py \
  --repo-root /path/to/uor-r4 \
  --open-issues --issue 1079 --issue 1081 --issue 1082 \
  --output-root /path/to/local/new-snapshot --ingest
```

This command reads live native parents/blockers and the last two comments per
selected issue, plus the explicit public planning/claim/source documents. It
rejects private repositories and document symlinks escaping the checkout.
The optional `--ingest` performs the local import; GitHub remains read-only.
Omit it to inspect the JSONL first. Records preserve old snapshots rather than
rewriting them. Always check collection time and live eligibility when acting.

## Inventory storage before large work

```bash
python3 scripts/project_storage_inventory.py \
  --repo /path/to/uor-r4 \
  --path product=/path/to/uor-r4-project \
  --output /path/to/local/storage-inventory.json
```

This command only measures. It reports permission/time limits and a proposed reserve; it never deletes anything or changes sealed-data permissions. Its byte rows can overlap and are not additive. Review exact worktree/build/cache candidates under the plan's retention policy before any cleanup. No recurring monitor was activated by this tooling installation.

## Project instructions

The installed `uor-project-workflow` skill is maintained at
[tools/skills/uor-project-workflow](../../tools/skills/uor-project-workflow/SKILL.md),
with a deferred [paper-workflow reference](../../tools/skills/uor-project-workflow/references/publication.md).
The skill must follow the native execution policy and must not turn this archive into
a routine task checklist.
