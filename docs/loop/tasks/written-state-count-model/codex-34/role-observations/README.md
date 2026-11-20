# Frozen role features before the boundary change

The sixteen exposed D32 crossed prompts explicitly label three query mentions each. The full-user/repeated trace contains 12/16 requested mentions, 12/16 excluded and 16/16 incidental mentions at their exact original endpoints. On Wick we joined only those designated spans to candidate features in the existing serving tapes. The eight absent endpoints stay in the denominator. The full primary and cold records were checksum verified and gave identical diagnostic JSON.

| Observation available at the designated span | Distinct observed values across 40 present mentions | Conflicting values | Maximum correctly labelled by any deterministic function of this observation |
|---|---:|---:|---:|
| Six ordered root and offset keys | 8 | 0 | 40/40 |
| Sorted root multiset with offsets removed | 8 | 0 | 40/40 |
| Two frozen native role scores | 7 | 1 | 38/40 |

The frozen score pair [32, -20] occurs at two requested and six excluded mentions. Changing a global scalar threshold or allowing an arbitrary deterministic table of just those two scores cannot distinguish all eight. The ordered keys still distinguish the requests and exclusions on these rows. The same holds after removing order. This exposed set by itself therefore cannot demonstrate an ordering advantage. The eight observed feature vectors are also too few to estimate transfer. These upper bounds sum the majority-label frequency per observation. They apply to pointwise classification of the 40 registered query mentions, excluding other tokens and absent mentions. A decoder using other candidates, clause relations or dialogue state sees a different observation.

For the next fit, form disjoint wording families and identities with matched requested, excluded and incidental variants for the same person and values. Include counterfactuals with the same multiset and different roles, and whole-prompt pairs whose only changes are surface entity order and fact order. Report joint correctness over matched groups, the unconditional candidate loss and false factual copies. Preserve the old count bank, ordered versus offset-neutral features and independent three-way abstention control at equal active table bytes. Freeze group splits and support before training. The historical labels and exact-score collisions identify a fit test; no held accuracy or learning result was obtained here.

On Wick run `check.sh CAPTURE_DIR KEY_JSON collisions.jq NEW_OUT` on D32's ordinary-primary and ordinary-cold. Both result tapes had SHA-256 `2e4fe01ce2b0c221511304d6d87235d0671425a28bf629e663fab11c75d93efa`. [All observations](result.json)

A sensitivity check swapped the requested and excluded answer labels on row 30100 in a private copy of the key after capture. The grader detected a new collision on identical ordered features and the upper bound fell to 39/40. The original key and input files stayed sealed. [Control transformation](controls.jq)
