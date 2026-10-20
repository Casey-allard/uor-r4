#!/bin/bash
# add workers on cores 0 and 1 once the E8 and cosine text runs finish
cd "$(dirname "$0")"
until [ -f full_hyb_e8_s0.json ]; do sleep 10; done; ./tree_worker.sh 0 &
until [ -f full_hyb_cos_s0.json ]; do sleep 10; done; ./tree_worker.sh 1 &
until [ -f full_hyb_hyp_s0.json ]; do sleep 10; done; ./tree_worker.sh 2 &
wait
echo DONE > tree/all.done
