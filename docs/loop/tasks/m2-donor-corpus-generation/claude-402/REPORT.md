# REPORT: m2-donor-corpus-generation, claude-402 (Casey's switch to the DeepSeek API recorded as D89)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **No API request has been made.** This unit records a decision only. The API path, its sheet amendment and its
  non-author FIT belong to the production unit (claude-400). The prices and the 2,500-request limit are DeepSeek's
  published figures on 16 Nov, not measurements.
- **Two pods were paid for and never used.** vast-60895844 (8×B200, $85.91/h) and vast-60895824 (4×B200,
  $43.41/h) were leased at about 07:41Z and deleted at about 07:51Z, before SSH accepted the key and before any
  file was put on them: about $21.55 in all at the stated rates and times. `uor-pod down` refused twice: once for the missing lab flag, and once
  because it could not reach the pods to archive results. The lab confirmed there was nothing to archive and
  deleted them with `--force --confirm-archived`.
- **The cost estimate is a list-price estimate.** About $800–900 for 1B kept assumes about 1.19B generated (pod 3
  dropped about 16% at the length limit), off-peak rates and a modest input share. A different drop rate or a
  longer prompt share moves it.
- **The API's model build is not yet matched to pod 3's weights.** The API names DeepSeek-V4.1-Flash-1017; the
  production amendment has to check and record whether that is the checkpoint pod 3 serves.
- **No seed replay.** The API takes no sampling seed, so a turn cannot be regenerated bit for bit. The corpus is
  still sealed by its hashes.

## What changed

- **D89** in `docs/integration/DECISIONS.md`: the rest of D87's corpus is streamed from the DeepSeek API instead
  of a rented fan-out. D87's donor, reasoning rule, hard-text-target rule and 1B size stand.
- **The 16 Nov D89 entry** in `docs/rules/changes.md`, quoting Casey's answers.

## Measurements

None. This is a record.

## Checks run at the head

- `loop pr-check --local`: PASS at 19fe39af1 and at this head (standing advisories: the AGENTS.md size and FILEMAP rows).
- The fresh-context review of 19fe39af1 returned CHANGES with nine findings, all fixed at this head. They covered
  the D87 lab conditions an API run cannot meet, the provider moved to the owner's terms, the 8×B200 yes marked
  superseded, the key wording aligned with §3.13, one-line quotes, and the cost figures explained.
- `git diff --check` is clean.

## Skills used

- `scope-guard`: two owner-path edits named in the ORDER, nothing else.

## Still open

1. The production unit's amendment 6, covering the API client, off-peak only, the seed recorded as absent and the
   model-version check, with its non-author FIT.
2. Casey puts a DeepSeek API key in an environment variable for the instrument. The lab never reads or prints it.
3. Pod 3 keeps generating until the API path passes its checks, then it is downed.

## Plain summary

Casey chose to stream the rest of the 1B-token donor corpus from DeepSeek's own API, which costs about 55–60% of
renting GPUs at list prices. The two pods just rented for the fan-out were deleted before any work reached them, and the first
pod keeps generating until the API path is checked. This change writes Casey's words into the decision log and the
rules change log.
