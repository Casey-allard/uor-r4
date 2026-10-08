#!/bin/bash
# D20 run helper. Usage: run_one.sh <arm:geo|ctl|baselines> <seed> <root>
# NOTE: <root> must NOT exist -- the binary claims it exclusively itself.
set -u
BIN=$HOME/.cache/uor-r4-d20control/release/examples/mqar-bench
ARM=$1; SEED=$2; ROOT=$3
MAXSEC=${4:-3600}
PROBE=${5:-none}
LOGDIR=$HOME/uor-r4-worktrees/d20-run/cost
mkdir -p "$LOGDIR"
NAME=$(basename "$ROOT")
COST=$LOGDIR/$NAME.cost.txt
FREE_BEFORE=$(python3 -c "import os;s=os.statvfs('/System/Volumes/Data');print(s.f_bavail*s.f_frsize)")
START=$(date +%s)
COMMON="out=$ROOT device=cpu context=512 batch=8 steps=1800 lr=0.001 warmup=100 min_lr=0.1 \
weight_decay=0.1 clip=1.0 eval_every=100 curve_sequences=8 final_sequences=64 \
seed=$SEED max_seconds=$MAXSEC probe_steps=$PROBE"
if [ "$ARM" = "geo" ]; then
  ARGS="pattern=rrarra read=l2 rotation=true width=128 heads=4 mlp=384"
elif [ "$ARM" = "ctl" ]; then
  ARGS="arm=transformer layers=6 width=128 heads=4 mlp=matched match_pattern=rrarra match_read=l2 match_rotation=true"
else
  ARGS="mode=task-baselines"
fi
# shellcheck disable=SC2086
/usr/bin/time -l "$BIN" $COMMON $ARGS > "$LOGDIR/$NAME.stdout.txt" 2> "$LOGDIR/$NAME.stderr.txt"
EXIT=$?
END=$(date +%s)
FREE_AFTER=$(python3 -c "import os;s=os.statvfs('/System/Volumes/Data');print(s.f_bavail*s.f_frsize)")
SIZE=$(du -sk "$ROOT" 2>/dev/null | cut -f1)
MAXRSS=$(grep -a "maximum resident set size" "$LOGDIR/$NAME.stderr.txt" | tail -1 | awk '{print $1}')
{
  echo "arm=$ARM seed=$SEED root=$ROOT exit=$EXIT"
  echo "wall_seconds=$((END-START)) max_rss_bytes=${MAXRSS:-unknown} root_kib=${SIZE:-0}"
  echo "free_bytes_before=$FREE_BEFORE free_bytes_after=$FREE_AFTER"
} | tee "$COST"
exit $EXIT
