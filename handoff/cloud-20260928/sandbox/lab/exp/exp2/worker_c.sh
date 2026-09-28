#!/bin/bash
cd "$(dirname "$0")"
while [ ! -f runs/ct_dot.json ]; do sleep 5; done
exec ./worker.sh jobs_c.txt 3
