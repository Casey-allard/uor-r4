#!/bin/bash
# stage 2a extra arms: annealed exact conversion and direct curved (Gromov) conversion; waits for chain A's last run
S=/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad
cd $S/lab/exp/lead_conv
export PYTHONPATH=$S/leadlib:$S/pylib OMP_NUM_THREADS=1 XLA_FLAGS="--xla_cpu_multi_thread_eigen=false intra_op_parallelism_threads=1"
until [ -f ft_hpol_s1.json ]; do sleep 30; done
[ -f ft_hpolanneal_s1.json ] || { python3 ft.py --var hpol --seed 1 --steps 600 --anneal_to -2 --anneal_steps 300 --out ft_hpolanneal_s1.json > ft_hpolanneal_s1.log 2>&1; echo "EXIT $? hpolanneal_s1" >> ft_hpolanneal_s1.log; }
[ -f ft_gromov_s1.json ] || { python3 ft.py --var gromov --seed 1 --steps 600 --out ft_gromov_s1.json > ft_gromov_s1.log 2>&1; echo "EXIT $? gromov_s1" >> ft_gromov_s1.log; }
