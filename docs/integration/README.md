# Integration archive and optional tools

This directory preserves prior research and integration records. It is not a
pre-alpha checklist. Routine work follows the
[build-first pre-alpha policy](agent-execution-policy.md): implement and run the
product path without updating this archive, the claim ledger, or the knowledge
index. Formalization, publication, independent review, replay packages, and
broad QA stay deferred until a working alpha unless the owner explicitly asks
for one earlier.

Start with the [adopted current map](current-state.md), [workflow adoption record](workflow-adoption.md), and [reusable continuation prompt](CONTINUE.md). The [research and productization plan](../uor_productization_integration_plan.md) retains the full audit and design. Native GitHub issues and current source remain authoritative; dated audit snapshots preserve earlier state.

| Question | Evidence / decision document |
|---|---|
| What is current and missing from the roadmap? | [Reconciliation](roadmap-reconciliation.md), [native snapshot](roadmap-state.json) |
| What sources were discovered and inspected? | [552-repository public catalog](source-catalog.json), [UOR source audit](uor-source-audit.md), [18 pinned candidates](uor-integration-candidates.json) |
| What can external research contribute? | [HELM / GoldSnnail / W33 / NEMESIS review](external-research-audit.md) |
| What should be ported from the browser product? | [Component and API port plan](frontend-port-plan.md) |
| What is installed and how should it be used? | [Tool status](tooling-status.json), [workflow selection](workflow-tools.md) |
| What bounds automated agent execution? | [Build-first pre-alpha policy](agent-execution-policy.md), [machine contract](agent-execution-policy.json) |
| What is the current workbench checkpoint? | [#1107 source candidate](../r4_workbench_candidate_1107.md), [independent static review](../r4_workbench_candidate_1107_review.md) |
| What is needed later for a defensible paper? | Deferred reference: [publication readiness](publication-readiness.md), [publication tools](publication-tooling.json), [initial claim ledger](claim-ledger.json) |

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

The commands below document optional index maintenance. It is not a routine
pre-alpha deliverable. Run it only when current source retrieval materially
helps the active implementation; do not update the claim ledger merely to
record that maintenance.

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
The skill must follow the build-first policy and must not turn this archive into
a routine task checklist.
