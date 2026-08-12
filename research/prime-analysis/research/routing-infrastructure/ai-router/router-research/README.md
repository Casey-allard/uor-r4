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

Parse + summarize:
```bash
python tools/parse_logs.py results/raw results/parsed
python tools/summarize.py results/parsed results/summary.csv
```

## Read these first
- `docs/PROJECT_CONTEXT.md`
- `docs/PLAN.md`
- `docs/IMPLEMENT.md`
- `docs/RESULTS_SNAPSHOT.md`

