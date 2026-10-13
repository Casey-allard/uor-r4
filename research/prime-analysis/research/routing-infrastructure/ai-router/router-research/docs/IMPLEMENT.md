# Implementation Runbook (Codex)

After each PR:
1) run baseline: `bash runs/base_kmeans.sh`
2) parse: `python tools/parse_logs.py results/raw results/parsed`
3) summarize: `python tools/summarize.py results/parsed results/summary.csv`
4) update `docs/DECISIONS.md` if anything meaningful changed

Performance work (PR-2):
- add timing hooks (dataset gen, chart loop, routing, growth)
- identify top hotspots and fix those first
- add early stop and fast_dev

