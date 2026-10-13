# Router Research: Hyperbolic + 4D Polar Routing (SO(8) chart + growth)

This repo is a **Codex-friendly** research harness for a discrete routing mechanism over a hyperbolic
(Poincaré ball) latent, with:

- Discrete routing to `(shell, sector)` buckets
- Optional learned **chart** in tangent space (SO(d) rotation + scaling)
- Local memory per bucket (EMA)
- Growth splitting (loss-based new slots)

## Why this exists

Manual runs that take 15–30 minutes each are how humans reinvent suffering.
This repo is structured so an agent can run, parse, summarize, and iterate without you babysitting it.

## Quickstart

```bash
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

Run one baseline (writes logs):
```bash
bash runs/base_kmeans.sh
```

Run full staged pipeline:
```bash
bash runs/run_pipeline.sh
```

Parse + summarize:
```bash
python tools/parse_logs.py results/raw results/parsed
python tools/summarize.py results/parsed results/summary.csv
```

Run tests:
```bash
python -m unittest discover -s tests -v
```

## Read these first
- `docs/SESSION_BOOTSTRAP.md`
- `CORE_PROJECT_GOALS.md`
- `docs/PROJECT_CONTEXT.md`
- `docs/research/KILL_LIST_TRACKER.md`
- `docs/research/ACTIVE_STATE.md`
- `docs/PLAN.md`
- `docs/IMPLEMENT.md`
- `docs/RESULTS_SNAPSHOT.md`
- `docs/context/REPO_MAP.md`
- `docs/contracts/RUN_SUMMARY_SCHEMA.md`
- `docs/routes/ROUTE_MATRIX.md`
- `docs/research/SUPPORTING_EVIDENCE.md`

Quick context reload after a long pause or compaction:
```bash
python tools/context_bootstrap.py --group startup --cat
python tools/check_research_state.py
```

If the next step feels disconnected from the moonshot, also reload:
```bash
python tools/context_bootstrap.py --group theory --cat
```
