#!/usr/bin/env bash
set -euo pipefail
if [[ $# != 5 ]]; then
    echo "usage: $0 PROBE UOR_CHAT STUDY_ROOT POSITIVE_ROOT NEW_OUTPUT_DIR" >&2
    exit 2
fi
bin=$(realpath "$1")
chat=$(realpath "$2")
root=$(realpath "$3")
positive=$(realpath "$4")
mkdir "$5"
out=$(realpath "$5")
prior=$HOME/uor-codex/20261118-query-region-pick
(cd "$root/assets"; sha256sum --status -c SHA256SUMS)
(cd "$prior/region/panels"; sha256sum --status -c SHA256SUMS)
(cd "$prior/region/primary"; sha256sum --status -c decisions.sha256)
sha256sum "$bin" "$chat" > "$out/executables.sha256"
for seed in 1 2; do
    trunk=$HOME/uor-codex/20261118-value-connectors/inputs/seed$seed.d11
    carrier=$root/assets/seed$seed-paired.d11
    "$bin" memory "$positive" "$trunk" "$seed" > "$out/seed$seed-memory.json"
    for split in held nuisance; do
        run=$prior/writer/primary/seed$seed-$split
        (cd "$run"; sha256sum --status -c SHA256SUMS)
        id=held-0-true-0-0-unique-ignoring-after-0
        if [[ "$split" == nuisance ]]; then
            id=nuisance-held-0-false-0-0-unique-ordinary-after-0-trailing
        fi
        witness=$out/seed$seed-$split.json
        "$bin" exposed "$prior" "$carrier" "$seed" "$split" "$id" > "$witness"
        index=$(jq -er '.row' "$witness")
        raw=$(jq -er --argjson i "$index" '.[$i].prompt' "$prior/region/panels/$split-grade.json")
        for read_mode in enabled no_read; do
            observed=$out/seed$seed-$split-$read_mode.json
            "$chat" --discrete "$carrier" --tokenizer "$root/assets/tokenizer-seed$seed.json" \
                --protocol 2 --temperature 0 --max-tokens 16 --read-mode "$read_mode" --say "$raw" > "$observed"
            # The predecessor adverse arm supplied a recorded no-source result
            # on both tapes. The explicit no-read result binds to that trajectory.
            arm=original
            if [[ "$read_mode" == no_read ]]; then arm=adverse; fi
            jq -en --slurpfile observed "$observed" --slurpfile blind "$prior/region/panels/$split-blind.json" \
              --slurpfile trajectory "$run/$arm/chosen.json" --slurpfile witness "$witness" \
              --argjson i "$index" --arg arm "$arm" --arg read "$read_mode" '
                $observed[0] as $o | $blind[0][$i] as $b | $trajectory[0].rows[$i] as $p |
                ($witness[0].arms[]|select(.arm==$arm)) as $a |
                ($o.prompt_ids==$b.prefix and $o.artifact_sha256==$witness[0].carrier and
                 $o.selected.source==$p.source and $o.generated_ids==$p.generated and $o.stop=="eos" and
                 (if $read=="enabled" then $o.paired.role_words==$a.words
                  else $o.paired.role_words==null end) and
                 $o.copy_step_words == [$p.steps[]|.after+[.emitted,(.source_id//4294967295),.copied,.release]])
              ' > "$observed.parity"
        done
    done
done
(cd "$out"; sha256sum *.json *.parity executables.sha256 > SHA256SUMS && sha256sum -c SHA256SUMS)
