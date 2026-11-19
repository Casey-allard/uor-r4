# [codex] Which address reaches the exact writer?

On the new factorial panel, the joint operator repaired 15 positive replies and broke 14. Every change in either direction occurred at the first emitted token. It substituted an excluded person on 64 unsupported requests, all in “Tell me the thing for [requested], excluding [other].” Each of the four unsupported conditions contributed 16 false copies. With the gold requested occurrence, false copies fell to zero. The strict support rule withheld all 48 duplicated affirmative facts even when both facts gave the same value. The comparison below uses actual generated IDs, including EOS. Both released seed files gave these same counts. [Factorial inputs](CENSUS.json) · [Writer and cubins](FROZEN-WRITER.txt)

| Frozen-writer input | Exact on 48 single assertions | Exact on 48 agreeing duplicate assertions | Unsupported reads copied /192 | Both role directions exact on 24 identical fact histories |
|---|---:|---:|---:|---:|
| Prior ungated | 26 | 28 | 192 | 11 |
| Prior action | 11 | 14 | 88 | 4 |
| New nomination with prior permission | 26 | 28 | 120 | 11 |
| Prior nomination with strict support | 11 | 0 | 64 | 4 |
| New nomination and strict support | 26 | 0 | 64 | 11 |
| Requested-query oracle and strict support | 41 | 0 | 0 | 18 |
| Direct gold value on answerable only | 48 | 48 | 0 | 24 |
| Forced peer | 0 | 0 | 192 | 0 |
| No read | 0 | 0 | 0 | 0 |

For “Joe held brown. May held pink.” and the two “Regarding …” requests, the prior gate withheld both. The first emitted ID was 1155 for either. Joint selection generated pink [1481, 1] or brown [1730, 1] according to the requested person. For the same facts and the wording ending “excluding [other]”, both operators copied the other person's color at index zero. Another Joe/May pair changed the wrong first ID from 1155 to 379, leaving the error at zero. Fourteen earlier exact duplicate cases became index-zero errors. The complete table against the prior gate on 96 answerable rows is 40 zero→zero, 15 zero→exact, 16 one→one, 14 exact→zero, and 11 exact→exact. [Paired prompts and full IDs](PAIRED-EXAMPLES.json)

The seven singleton misses under the query oracle all have correct query and fact coordinates. The first wrong coordinate is the frozen value proposal, two prompt tokens after the correct value on every row. It crosses the period four times into the next question (“Set aside …”, first ID 301), and three times into the other person's name (first ID 379). Word-start selection, connector continuation and the actual writer all retain that wrong source. E.g. with “May held pink. Joe held brown. Set aside May, and describe the possession of Joe.”, the model reads position 13 instead of 11 and emits 301 in place of 1730. Passing the correct value address directly yields the literal response through EOS on all 96. A within-assertion fence is a testable intervention on these seven rows. [Seven complete source traces](QUERY-VALUE-FAULTS.json)

The original historical mixed panel contains 48 supported requests, 48 absent, 48 ambiguous, and 50 ordinary requests. New nomination with strict support answers all 48 supported, recovers all 24 prior index-zero misses, and copies on all 48 ambiguous requests. The prior gate answered 24/48 and copied on none of the 146 unsupported. With a gold requested occurrence the bounded positive score is 48/48 and all unsupported actions withhold. All 50 ordinary literal references remain unmatched. No literal reply was imposed on undefined or contradictory requests. The 50 ordinary references cover four distinct ordinary questions. [Actual-feedback grader](grade.jq)

## Reproduction

Run the existing Rust tokenizer example with the released tokenizer, `--requested-support` and a new output directory. Both local independent generations matched the four panel digests. Upload the four inputs to the paths in the scripts. With the registered frozen predecessors on Wick, run `bash old-wick.sh`, then `bash writer-wick.sh ARM held` for role, old, none, right and wrong. Run the corresponding `grade-wick.sh`. After blind and separately marked oracle handoffs are sealed, run `bash run-candidates.sh blind`, `oracle` and `compare`. The old role, gate and no-read historical outputs are bound by model, prompt bytes, per-row prefix hashes and the same writer cubins. A corrupted prompt hash is rejected. Each actual rollout is independently repeated, uses emitted IDs as the next input and checks raw device versus reference at every transition. [Panel digests](SHA256SUMS) · [Negative control](HANDOFF-NEGATIVE.txt)

## Historical generalization and replication

| Exact replies through EOS per released seed | Prior role | Prior action | Nomination only | Strict support only | Joint learned | Query occurrence oracle | Read disabled |
|---|---:|---:|---:|---:|---:|---:|---:|
| Historical unseen /1,728 | 1,406 | 794 | 1,026 | 864 | 1,008 | 1,728 | 0 |
| Historical transfer /1,728 | 1,610 | 1,610 | 1,610 | 1,614 | 1,614 | 1,614 | 0 |
| Historical mixed positive /48 | 48 | 24 | 48 | 24 | 48 | 48 | 0 |

On historical unseen, the joint model recovered 214 exact replies from the prior gate and moved 86 additional first errors from index zero to one. Compared with the prior ungated selector it recovered 70 exact replies but lost 468. Its role-swapped pairs with both answers exact were 504/864, compared with 580/864 ungated. On transfer it retained 1,610 prior gated exact replies and converted four index-zero errors to exact. The 114 remaining transfer errors begin at index zero even under the requested-occurrence oracle. The two full primary seed runs cover 42 arm/seed/panel files and 146,439 native/reference transitions per pass. All emitted token and trace files were byte-identical on repeat for each seed. The factual and ordinary scores matched across seeds. The raw per-seed distributions and controls are preserved. [Seed 1 counts](SEED1-SUMMARY.json) · [Seed 2 counts](SEED2-SUMMARY.json)

The [Wick primary and repeat writer archive](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/024ba037505430b1c882c419b7315a6425764706/results/codex/content-support-count-20261118/panels-writer-wick.tar.gz) was downloaded from the pinned revision and compared byte-for-byte. Archive SHA-256 `9861e4cd47ca3dfb9455e61f9d47e042d14129dcce4428eb449ab3eb4fb62df9`. It includes actual emitted IDs and all native transitions, primary and repeat measurements, all post-feedback grades, both gold and blind factorial files, scripts and the predecessor writer binary. Historical control trajectories and predecessor model files are bound by the [previous released archive](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/746913b113e99a515fdd3273492ec4656aed07cd/results/codex/role-boundary-count-20261118/crossed-writer-wick.tar.gz). A partial first-pass writer was preserved when the historical mixed oracle needed a multi-piece annotation repair. Every selected address used by the reported writer was compared to the repaired primary and independent repeated sealed handoffs. [Agreement supplement and independent receipt](AGREEMENT-RESULT.md)
