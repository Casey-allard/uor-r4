#!/bin/bash
cd "$(dirname "$0")"
while [ ! -f runs/st8_hyp.json ]; do sleep 5; done
exec ./worker.sh jobs_d.txt 3
