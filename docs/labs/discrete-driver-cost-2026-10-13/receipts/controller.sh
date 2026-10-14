#!/usr/bin/env bash
# Prepared only. Root supplies the prospective FIT confirmation and invokes
# through the admitted uor-pod session. No rental, build, corpus or cleanup.
set -euo pipefail
export LC_ALL=C
binary=/workspace/driver-qualification/target/release/discrete-stack
source_pin=e7d926fbe38537f4faa90b817705c61a780778d6
expected_binary_sha256=399ab42713589439dd963ec50515133e723abda79cd4b1d90330d9710d80c117
report=/workspace/driver-qualification/run-a1-e7d926fbe
commands=(qualify qualify-flock qualify-driver measure-cost)
expected_controls=(11 6 8 0)
expected_updates=(0 0 8 0)
next_command=0

[[ ${UOR_LAB:-} == codex ]] || { echo 'UOR_LAB=codex is required' >&2; exit 3; }
: "${UOR_FINITE_FIT_URL:?Root must supply the exact-head prospective FIT confirmation URL before execution}"
[[ $UOR_FINITE_FIT_URL =~ ^https://github\.com/UOR-Foundation/uor-r4/(pull/2400|issues/2029|issues/2032)\#issuecomment-[0-9]+$ ]] || {
    echo 'FIT receipt must be the approved PR2400 or milestone2029/2032 comment URL' >&2
    exit 3
}
[[ -x $binary ]] || { echo 'Pinned release executable is absent' >&2; exit 3; }
# mkdir is deliberately exclusive; nothing overwrites an earlier run.
mkdir -- "$report"

finish() {
    local controller_exit=$?
    trap - EXIT
    local index
    for ((index=next_command; index<${#commands[@]}; ++index)); do
        printf '{"index":%s,"command":"%s","status":"NOT_RUN","reason":"preceding controller or command failure"}\n' \
            "$((index+1))" "${commands[index]}" >> "$report/controller.jsonl"
    done
    printf '%s\n' "$controller_exit" > "$report/controller.exit"
    if ((controller_exit != 0)); then
        printf '{"status":"FAILED","controller_exit":%s,"source":"%s","verified_total_controls":null,"verified_applied_updates":null}\n' \
            "$controller_exit" "$source_pin" > "$report/RESULT.json"
    fi
    exit "$controller_exit"
}
trap finish EXIT

sha256sum -- "$binary" > "$report/binary.sha256"
read -r observed_hash _ < "$report/binary.sha256"
[[ $observed_hash == "$expected_binary_sha256" ]] || {
    echo 'Executable hash differs; no numerical command admitted' >&2
    exit 4
}
printf '{"source":"%s","executable":"%s","binary_sha256":"%s","profile":"release","fit_confirmation":"%s","training":"HELD","planned_controls":[11,6,8],"planned_fixture_updates":8,"measurement_updates":0}\n' \
    "$source_pin" "$binary" "$expected_binary_sha256" "$UOR_FINITE_FIT_URL" > "$report/PINS.json"

now_ns() {
    local value
    value=$(date +%s%N)
    [[ $value =~ ^[0-9]{18,19}$ ]] || { echo 'Linux integer nanosecond clock is unavailable' >&2; return 5; }
    printf '%s\n' "$value"
}
has_integer() {
    local file=$1 key=$2 value=$3
    grep -Eq "^  \"$key\": $value(,)?$" "$file"
}
validate_report() {
    local index=$1 command=$2 file
    if [[ $command == measure-cost ]]; then
        file="$report/$command/measurement.json"
        if ! { [[ -f $file ]] && grep -Eq '^  "status": "PASS"(,)?$' "$file" && has_integer "$file" applied_updates 0; }; then return 1; fi
    else
        file="$report/$command/controls.json"
        if [[ $command == qualify-flock ]]; then file="$report/$command/flock-controls.json"; fi
        if ! { [[ -f $file ]] && grep -Eq '^  "status": "PASS"(,)?$' "$file" \
            && has_integer "$file" registered "${expected_controls[index]}" \
            && has_integer "$file" attempted "${expected_controls[index]}" \
            && has_integer "$file" passed "${expected_controls[index]}" \
            && has_integer "$file" failed 0 && has_integer "$file" not_run 0 \
            && has_integer "$file" applied_updates "${expected_updates[index]}"; }; then return 1; fi
        if [[ $command == qualify-driver ]]; then
            if ! { has_integer "$file" apply_calls 11 \
                && grep -Eq '^  "update_count_complete": true(,)?$' "$file" \
                && has_integer "$file" corpus_updates 0; }; then return 1; fi
        fi
    fi
    return 0
}

for ((index=0; index<${#commands[@]}; ++index)); do
    command=${commands[index]}
    start_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    start_ns=$(now_ns)
    printf '%s\n' "$start_utc" > "$report/$command.started-utc"
    printf '%s\n' "$start_ns" > "$report/$command.started-ns"
    # The numerical command itself creates its report subdirectory exclusively.
    # One invocation only; there is no retry, apply CLI or unfiltered test suite.
    if "$binary" "$command" "$report/$command" > "$report/$command.stdout" 2> "$report/$command.stderr"; then
        command_exit=0
    else
        command_exit=$?
    fi
    next_command=$((index+1))
    end_ns=$(now_ns)
    end_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    elapsed_ns=$((end_ns-start_ns))
    ((elapsed_ns >= 0)) || { echo 'Clock moved backwards; preserve the command as unverified' >&2; exit 5; }
    printf '%s\n' "$command_exit" > "$report/$command.exit"
    printf '%s\n' "$end_utc" > "$report/$command.finished-utc"
    printf '%s\n' "$end_ns" > "$report/$command.finished-ns"
    printf '%s\n' "$elapsed_ns" > "$report/$command.elapsed-ns"
    contract_exit=0
    status=PASS
    if ((command_exit != 0)); then
        status=FAILED
    elif ! validate_report "$index" "$command"; then
        status=REPORT_CONTRACT_FAILED
        contract_exit=6
    fi
    printf '{"index":%s,"command":"%s","status":"%s","command_exit":%s,"contract_exit":%s,"start_utc":"%s","end_utc":"%s","elapsed_ns":%s,"expected_controls":%s,"expected_applied_updates":%s}\n' \
        "$((index+1))" "$command" "$status" "$command_exit" "$contract_exit" "$start_utc" "$end_utc" "$elapsed_ns" \
        "${expected_controls[index]}" "${expected_updates[index]}" >> "$report/controller.jsonl"
    if ((command_exit != 0)); then exit "$command_exit"; fi
    if ((contract_exit != 0)); then exit "$contract_exit"; fi
done
printf '{"status":"PASS","controller_exit":0,"source":"%s","verified_total_controls":25,"verified_applied_updates":8,"corpus_updates":0,"measurement_updates":0,"training":"HELD"}\n' \
    "$source_pin" > "$report/RESULT.json"
