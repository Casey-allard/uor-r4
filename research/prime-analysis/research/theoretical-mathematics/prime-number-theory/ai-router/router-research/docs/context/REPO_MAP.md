# Repository Map

## Core
- `hyperbolic_router_so8.py`: main research executable (routing + chart + growth + metrics).
- `requirements.txt`: python dependencies.
- `README.md`: project quickstart.

## Runs
- `runs/base_kmeans.sh`: baseline route wrapper.
- `runs/polar_phase2.sh`: phase2 route wrapper.
- `runs/run_pipeline.sh`: full automated pipeline (sweep -> parse -> summarize).

## Tooling
- `tools/parse_logs.py`: extracts `__JSON_SUMMARY__` from logs into JSON.
- `tools/summarize.py`: tabulates parsed summaries into `results/summary.csv`.
- `tools/sweep.py`: staged route sweeps with decision gate note output.
- `tools/proxy_sweep.py`: staged LM proxy route sweeps with route-health-aware aggregation.
- `tools/prepare_wikitext2.py`: deterministic data prep for WikiText-2 proxy track.

## Tasks
- `tasks/wikitext2_proxy.py`: converts prepared corpus into routing-compatible tensors.
- `tasks/dense_baseline.py`: dense baseline comparator for real-task hardware/quality checks.
- `tasks/router_proxy_eval.py`: runs the geometry router directly on LM proxy tensors for route transfer evaluation.

## Docs
- `docs/PROJECT_CONTEXT.md`: mechanism summary and current research thesis.
- `docs/PLAN.md`: PR-by-PR strategy.
- `docs/IMPLEMENT.md`: runbook after each PR.
- `docs/DECISIONS.md`: canonical decision log.
- `docs/RESULTS_SNAPSHOT.md`: pasted historical outcomes.
- `docs/context/HISTORY_RECONSTRUCTED.md`: reconstructed project timeline.
- `docs/contracts/CLI_FLAGS_CONTRACT.md`: required CLI and argument semantics.
- `docs/contracts/RUN_SUMMARY_SCHEMA.md`: schema for machine-readable run output.
- `docs/routes/ROUTE_MATRIX.md`: route IDs, hypotheses, kill criteria.
- `docs/research/RESEARCH_STRATEGY.md`: research north star, success criteria, and anti-goals.
- `docs/research/CURRENT_DIRECTION.md`: current best route, current mechanistic reading, and next tracks.
- `docs/research/ADAPTIVE_PHASE4D_SPEC.md`: formal spec for time-expanded adaptive 4D phase routing.
- `docs/research/OPEN_QUESTIONS.md`: prioritized unanswered research questions.
- `docs/research/DECISION_RUBRIC.md`: scoring rubric for future increments.
- `docs/research/LEAD_RESEARCH_LOOP.md`: workflow for lead-research decisions and user escalation.
- `docs/research/FLEET_ASSIGNMENTS.md`: current team responsibilities and active assignments.
- `docs/research/INTEGRATION_PATHS.md`: candidate insertion points into real model architectures.
- `docs/pipelines/*.md`: pipeline contracts and I/O definitions.
- `docs/governance/*.md`: decision gates and advisor recommendation template.
- `docs/program/*.md`: story and sprint operating templates.
- `docs/agents/packets/*.md`: pod-specific execution context packets.

## Results
- `results/raw/`: raw run logs.
- `results/parsed/`: parsed JSON summaries.
- `results/analysis/`: experiment-level aggregate analyses for transfer batches.
- `results/summary.csv`: tabulated run records.
- `results/INDEX.md`: result batches and associated decisions.
