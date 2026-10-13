# Pipeline: Parse + Summarize

## Input
- `results/raw/*.log`

## Steps
1. `python tools/parse_logs.py results/raw results/parsed`
2. `python tools/summarize.py results/parsed results/summary.csv`

## Output
- parsed JSON files in `results/parsed/`
- deduplicated summary rows in `results/summary.csv`
