#!/bin/bash
cd "$(dirname "$0")"
until [ -f geo/worker_0.done ] && [ -f geo/worker_1.done ] && [ -f geo/worker_2.done ] && [ -f geo/worker_3.done ]; do sleep 10; done
for c in 0 1 2 3; do ./phase2_worker.sh $c & done
wait
echo DONE > phase2.done
