#!/usr/bin/env bash
sum() { sha256sum "$1" | cut -d' ' -f1; }
pin() {
    [[ "$1" == /* && "$2" =~ ^[0-9a-f]{64}$ ]]
    printf '%s  %s\n' "$2" "$1" | sha256sum --status --check
}
