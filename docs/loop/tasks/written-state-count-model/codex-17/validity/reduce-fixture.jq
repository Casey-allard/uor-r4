def fixture_label($i):
    {
        id: "synthetic-\($i)",
        family: (if $i < 18 then "family-a" else "family-b" end),
        stratum: (if $i < 36 then "supported" else "unmentioned" end),
        matched_triple: (if $i < 24 then "triple-\(($i / 3) | floor)" else null end),
        gold_offset: (if $i < 36 then $i % 3 + 1 else null end),
        gold_start: (if $i < 36 then 4 + $i % 3 else null end)
    };

def grade($exact):
    . as $label
    | {
        index: .index, id: .id, semantics: .stratum, original_exact: $exact,
        active_exact: $exact, terminal_eos: true,
        role_nomination: (if .index == 3 then null else [10,11] end),
        gold_role_nomination: [10,11],
        source: (if .index == 4 then .gold_start + 1
                 elif .index == 6 or .index == 7 or .index == 8 or .index == 9 then null
                 else .gold_start end),
        original_first_wrong: (if $exact then null elif .index == 5 then 1 else 0 end),
        original_expected: [42,43,1],
        generated: (if $exact then [42,43,1] elif .index == 5 then [42,1] else [43,1] end),
        decoded_text: (if .index < 36 then "synthetic literal" else "No." end),
        negative: (if .index < 36 then null else {classification:"no",source:null} end)
    }
    | if $label.index < 36 then . else .original_exact = null end;

def arm($name; $winners; $labels):
    {
        arm: $name,
        report: {
            schema: "uor-r4.value-rank-observed-grader/1",
            rows: [$labels | to_entries[] | .key as $i
                | select($i < 36 or ($name | contains("-normal-")))
                | (.value + {index: $i}) | grade(($winners | index($i)) != null)]
        }
    };

[range(0;42) | fixture_label(.)] as $labels
| {
    labels: $labels,
    inputs: [
        (arm("requested-ordered"; [0,1,12,25]; $labels) + {
            native_sources: {
                mode: "ordered",
                rows: [
                    {ordinal:6, assertion_index:null, status:"no_candidate"},
                    {ordinal:7, assertion_index:0, status:"unseen",
                     candidates:[{status:"unseen", positive_evidence:null}]},
                    {ordinal:8, assertion_index:0, status:"unseen",
                     candidates:[{status:"unseen", positive_evidence:10, negative_evidence:30, radius:null}]},
                    {ordinal:9, assertion_index:0, status:"unseen",
                     candidates:[{status:"unseen", positive_evidence:10, negative_evidence:30, radius:9}]}
                ]
            }
        }),
        arm("requested-neutral"; [1,2,26]; $labels),
        arm("learned-normal-ordered"; [0]; $labels)
    ]
}
