#!/bin/bash
set -u
cd /tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/lab/exp/c3b
until grep -q ALL_DONE /tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/lab/exp/c3b/ft_status.log 2>/dev/null; do sleep 20; done
for m in lorentzflat_s1 dot_s1 lorentzflat_s2 dot_s2; do
  ( /tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/target/release/examples/joint-integer-parity model=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/lab/exp/c3b/ft_qat_${m}/model tables=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/lab/exp/c3b/tables tokens=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/c3/data/code/valid.u16 lens=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/c3/data/code/lens.u16 out=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/lab/exp/c3b/qatparity_${m} windows=512 > /tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/lab/exp/c3b/qatparity_${m}.log 2>&1; echo "$m exit $?" >> /tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/lab/exp/c3b/qatparity_status.log ) &
done
wait
echo PARITY_DONE >> /tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad/lab/exp/c3b/qatparity_status.log
