# Implementation Runbook (Codex)

After each PR / increment:
1) run baseline: `bash runs/base_kmeans.sh`
2) parse: `python tools/parse_logs.py results/raw results/parsed`
3) summarize: `python tools/summarize.py results/parsed results/summary.csv`
4) update `docs/DECISIONS.md` if anything meaningful changed
5) update the active increment doc in `docs/research/increments/`
6) refresh `docs/research/CURRENT_DIRECTION.md` and `docs/research/LIVE_WORKLOG.md`
7) if sweeping: `bash runs/run_pipeline.sh`
8) follow `docs/research/UPDATE_PROTOCOL.md` so a new session can resume without reconstructing intent from raw logs

Performance work:
- add timing hooks (dataset gen, chart loop, routing, growth)
- identify top hotspots and fix those first
- add early stop and fast_dev
- use cache controls (`--cache_chart`, `--cache_routes`) for repeat experiments

Research persistence:
- keep `docs/research/LIVE_WORKLOG.md` current enough that a new session can resume from disk
- whenever a branch family gains a coherent mechanism-level name, record it in docs even if the on-disk artifact labels stay parameterized
