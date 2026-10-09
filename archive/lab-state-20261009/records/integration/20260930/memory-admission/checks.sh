#!/bin/bash
set -euo pipefail
/Users/casey.allard/.cargo/bin/cargo test -p lab-runner --offline --lib host::tests -- --test-threads=2
/Users/casey.allard/.cargo/bin/cargo build -p lab-runner --offline
