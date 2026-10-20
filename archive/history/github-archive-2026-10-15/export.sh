#!/usr/bin/env bash
# export.sh — archive the complete GitHub issue/PR/comment corpus of UOR-Foundation/uor-r4.
#
# Outputs (relative to this directory, docs/history/github-archive-2026-10-15/):
#   json/issues/<number>.json    raw issue object + its conversation comments ("comments" array)
#   json/pulls/<number>.json     raw issue object + raw pull object + "reviews" + "review_comments"
#                                + "comments" (conversation comments on the PR)
#   json/labels.json             all repository labels
#   json/milestones.json         all milestones (state=all)
#   markdown/NNNN-<slug>.md      human rendering; every comment section is headed
#                                "### comment <id> — <created_at> — <login>" so citations
#                                like "comment 6181921625" resolve with grep
#   INDEX.md                     contents table, reconciliation vs live counts, spot checks
#
# Method (kept request-frugal for the shared GitHub identity):
#   GET /repos/{repo}/issues            state=all (PRs included; "is_pull_request" marks them)
#   GET /repos/{repo}/pulls             state=all
#   GET /repos/{repo}/issues/comments   every conversation comment (including comments on PRs)
#   GET /repos/{repo}/pulls/comments    every inline review comment
#   GET /repos/{repo}/pulls/<n>/reviews one call per PR (no repo-level reviews endpoint exists)
#   GET /repos/{repo}/labels and /milestones
# Conversation comments are verified per item against the `comments` counter carried in the issues
# snapshot. Inline review comments are verified globally and, for every PR that has any, against a
# direct single-PR GET (GitHub's list-pulls representation omits the `review_comments` counter and
# PR diff stats); any mismatch is re-fetched per item before rendering. json/pulls/<n>.json
# therefore stores the list representation — merged_at, merge_commit_sha, base/head, user, dates,
# labels and body are present; additions/deletions/changed_files/commits are not offered by the API.
#
# Re-runnable: intermediate state lives in <worktree>/local/github-archive-<date>/ (gitignored)
# and is reused for crash recovery. Run `export.sh --fresh` to discard it and re-pull everything.
# Requires: gh (authenticated, repo read), jq, git.
set -euo pipefail

REPO="${REPO:-UOR-Foundation/uor-r4}"
ARCHIVE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git -C "$ARCHIVE_DIR" rev-parse --show-toplevel)"
EXPORT_DATE="${EXPORT_DATE:-$(date -u +%Y-%m-%d)}"
EXPORT_TS="${EXPORT_TS:-$(date -u +%Y-%m-%dT%H:%M:%SZ)}"
BASE_COMMIT="$(git -C "$REPO_ROOT" rev-parse HEAD)"
SCRATCH="${SCRATCH:-$REPO_ROOT/local/github-archive-$EXPORT_DATE}"
JSON_DIR="$ARCHIVE_DIR/json"
MD_DIR="$ARCHIVE_DIR/markdown"

if [[ "${1:-}" == "--fresh" ]]; then rm -rf "$SCRATCH"; fi
rm -rf "$JSON_DIR" "$MD_DIR"
mkdir -p "$JSON_DIR/issues" "$JSON_DIR/pulls" "$MD_DIR" \
         "$SCRATCH/raw" "$SCRATCH/item" "$SCRATCH/reviews"
: > "$SCRATCH/export.log"

log() {
  local m
  m="$(date -u +%H:%M:%S) $*"
  printf '%s\n' "$m" >&2
  printf '%s\n' "$m" >> "$SCRATCH/export.log"
}

verdict() { if [[ "$1" == "$2" ]]; then printf 'match'; else printf 'DIFFERS'; fi; }

slugify() {
  local s
  s="$(printf '%s' "$1" | LC_ALL=C tr '[:upper:]' '[:lower:]' \
      | LC_ALL=C sed -e 's/[^a-z0-9][^a-z0-9]*/-/g' -e 's/^-*//' \
      | cut -c1-60 | sed -e 's/-*$//')"
  if [[ -z "$s" ]]; then s="untitled"; fi
  printf '%s' "$s"
}

# ---------------------------------------------------------------- rate-limit helpers

# Errors that can never succeed on retry (bad number, missing object). Everything else — TLS
# handshake timeouts, HTTP 5xx, connection resets, secondary limits — is retried with backoff;
# an exhausted primary budget waits for the hourly reset.
PERMANENT_RE='HTTP 400|HTTP 401|HTTP 403|HTTP 404|HTTP 410|HTTP 422|Not Found'

sleep_until_reset() {
  local reset now wait
  reset="$(gh api rate_limit --jq .resources.core.reset 2>/dev/null || echo 0)"
  now="$(date +%s)"
  if (( reset > now )); then wait=$((reset - now + 5)); else wait=30; fi
  if (( wait > 3700 )); then wait=3700; fi
  log "core rate budget low/exhausted — sleeping ${wait}s"
  sleep "$wait"
}

check_rate_budget() {
  local remaining
  remaining="$(gh api rate_limit --jq .resources.core.remaining 2>/dev/null || echo 1000)"
  if (( remaining < 300 )); then sleep_until_reset; fi
  return 0
}

fetch_pages() { # <endpoint> <outfile> — paginated fetch with reset-aware retry
  local endpoint="$1" out="$2" attempt=0 err="$SCRATCH/gh.err"
  while :; do
    if gh api --paginate "$endpoint" > "$out.tmp" 2>"$err"; then
      mv "$out.tmp" "$out"
      return 0
    fi
    attempt=$((attempt + 1))
    if (( attempt > 8 )); then
      log "FAILED after $attempt attempts: $endpoint"
      sed -n '1,4p' "$err" >&2
      return 1
    fi
    if grep -q 'API rate limit exceeded' "$err"; then
      sleep_until_reset
    elif grep -qiE 'secondary rate limit|abuse detection' "$err"; then
      local wait=$((attempt * 30))
      log "secondary rate limit — sleeping ${wait}s (attempt $attempt): $endpoint"
      sleep "$wait"
    elif grep -qiE "$PERMANENT_RE" "$err"; then
      log "permanent API error (no retry): $endpoint"
      sed -n '1,4p' "$err" >&2
      return 1
    else
      local wait=$((attempt * 20))
      log "transient API/network error — sleeping ${wait}s (attempt $attempt): $endpoint"
      sleep "$wait"
    fi
  done
}

gh_json() { # <endpoint> [jq-filter] — single call with reset-aware retry
  local endpoint="$1" filter="${2:-.}" attempt=0 err="$SCRATCH/gh.err" out
  while :; do
    if out="$(gh api "$endpoint" --jq "$filter" 2>"$err")"; then
      printf '%s' "$out"
      return 0
    fi
    attempt=$((attempt + 1))
    if (( attempt > 6 )); then log "FAILED: $endpoint"; return 1; fi
    if grep -q 'API rate limit exceeded' "$err"; then
      sleep_until_reset
    elif grep -qiE 'secondary rate limit|abuse detection' "$err"; then
      sleep $((attempt * 30))
    elif grep -qiE "$PERMANENT_RE" "$err"; then
      sed -n '1,3p' "$err" >&2
      return 1
    else
      sleep $((attempt * 10))
    fi
  done
}

# ---------------------------------------------------------------- markdown renderer

cat > "$SCRATCH/md.jq" <<'MD_JQ_PROGRAM_EOF'
# Render one archived issue or pull request as markdown.
# Input: the wrapper object written to json/issues/<n>.json or json/pulls/<n>.json.

def blankbody:
  if . == null or (((tostring | gsub("\\s"; "")) | length) == 0)
  then "_No body._" else . end;

def login: (.user.login // "ghost");

def comment_section:
  "### comment \(.id) — \(.created_at) — \(login)\n\n"
  + (if ((.updated_at // "") != (.created_at // "")) then "_edited \(.updated_at)_\n\n" else "" end)
  + (.body | blankbody) + "\n\n";

def review_section:
  "### review \(.id) — \(.submitted_at // .created_at // "") — \(login) — \(.state // "UNKNOWN")\n\n"
  + (if ((((.body // "") | gsub("\\s"; "")) | length) == 0)
     then "" else ((.body | blankbody) + "\n\n") end);

def review_comment_section:
  "### review-comment \(.id) — \(.created_at) — \(login)\n\n"
  + "_on \(.path // "?")"
  + (if .line then ":" + (.line | tostring)
     elif .original_line then ":" + (.original_line | tostring) else "" end)
  + (if .in_reply_to_id then " · in reply to review-comment \(.in_reply_to_id)" else "" end)
  + "_\n\n"
  + (.body | blankbody) + "\n\n";

def timeline:
  ([ (.comments        // [])[] | {ts: (.created_at // ""),                     kind: "comment",        e: .} ]
   + [ (.reviews         // [])[] | {ts: (.submitted_at // .created_at // ""), kind: "review",         e: .} ]
   + [ (.review_comments // [])[] | {ts: (.created_at // ""),                  kind: "review_comment", e: .} ])
  | sort_by(.ts)
  | map(.e as $e |
        if .kind == "comment" then ($e | comment_section)
        elif .kind == "review" then ($e | review_section)
        else ($e | review_comment_section) end)
  | join("");

def idlist:
  ([ (.comments // [])[] | .id ] + [ (.reviews // [])[] | .id ] + [ (.review_comments // [])[] | .id ])
  | map(tostring) | join(", ");

"# #\(.issue.number) — \(.issue.title)\n\n"
+ "- **Type:** \(if .is_pull_request then "Pull request" else "Issue" end)\n"
+ "- **State:** \(.issue.state)\(if .issue.state_reason then " (" + .issue.state_reason + ")" else "" end)\n"
+ "- **Author:** @\(.issue.user.login // "ghost")\n"
+ "- **Created:** \(.issue.created_at)\n"
+ "- **Updated:** \(.issue.updated_at)\n"
+ (if .issue.closed_at then "- **Closed:** \(.issue.closed_at)\n" else "" end)
+ (if ((.issue.labels // []) | length) > 0
   then "- **Labels:** \([(.issue.labels // [])[] | (if type == "object" then .name else . end)] | join(", "))\n"
   else "" end)
+ (if .issue.milestone then "- **Milestone:** \(.issue.milestone.title)\n" else "" end)
+ (if ((.issue.assignees // []) | length) > 0
   then "- **Assignees:** \([(.issue.assignees // [])[] | (.login // "ghost")] | join(", "))\n"
   else "" end)
+ "- **URL:** \(.issue.html_url)\n"
+ (if (.is_pull_request and (.pull != null))
   then
     "- **Merged:** \(if .pull.merged_at then .pull.merged_at else "not merged" end)\n"
   + (if .pull.merge_commit_sha then "- **Merge commit:** \(.pull.merge_commit_sha)\n" else "" end)
   + "- **Base / Head:** \(.pull.base.ref) / \(.pull.head.label)\n"
   + ([ (if .pull.commits != null then "commits \(.pull.commits)" else empty end),
        (if .pull.changed_files != null then "files changed \(.pull.changed_files)" else empty end),
        (if .pull.additions != null then "+\(.pull.additions) lines" else empty end),
        (if .pull.deletions != null then "-\(.pull.deletions) lines" else empty end) ]
      | if length > 0 then "- **Diff stats:** " + join(" · ") + "\n" else "" end)
   + "- **Reviews / inline review comments:** \((.reviews // []) | length) / \((.review_comments // []) | length)\n"
   else "" end)
+ "- **Archived:** \(.exported_at) · \(.source_repo)\n"
+ "\n## Body\n\n" + (.issue.body | blankbody) + "\n"
+ (if .is_pull_request then
     "\n## Timeline (\((.comments | length) + (.reviews | length) + (.review_comments | length)) entries; chronological: conversation comments, reviews, review comments)\n\n"
     + (if (((.comments | length) + (.reviews | length) + (.review_comments | length)) == 0)
        then "_No comments or reviews._\n" else timeline end)
   else
     "\n## Comments (\(.comments | length))\n\n"
     + (if ((.comments | length) == 0) then "_No comments._\n" else timeline end)
   end)
+ "\n---\n_Comment/review ids in this file: \(idlist)_\n"
MD_JQ_PROGRAM_EOF

# ---------------------------------------------------------------- phase 1: bulk fetches

log "export started: repo=$REPO export_date=$EXPORT_DATE worktree_base=$BASE_COMMIT"
log "phase 1: bulk fetches"

fetch_pages "repos/$REPO/issues?state=all&per_page=100&sort=created&direction=asc" "$SCRATCH/raw/issues.pages"
jq -s 'add // []' "$SCRATCH/raw/issues.pages" > "$SCRATCH/issues.json"
SNAPSHOT_TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
jq -r '.[] | "\(.number)\t\(if .pull_request then "pr" else "issue" end)"' "$SCRATCH/issues.json" > "$SCRATCH/items.tsv"
EXPECTED_TOTAL="$(wc -l < "$SCRATCH/items.tsv" | tr -d ' ')"
log "issues snapshot: $EXPECTED_TOTAL items (issues + PRs) as of $SNAPSHOT_TS"

fetch_pages "repos/$REPO/pulls?state=all&per_page=100&sort=created&direction=asc" "$SCRATCH/raw/pulls.pages"
jq -s 'add // []' "$SCRATCH/raw/pulls.pages" > "$SCRATCH/pulls.json"
jq -r '.[].number' "$SCRATCH/pulls.json" > "$SCRATCH/pr-numbers.txt"

fetch_pages "repos/$REPO/labels?per_page=100" "$SCRATCH/raw/labels.pages"
jq -s 'add // []' "$SCRATCH/raw/labels.pages" > "$JSON_DIR/labels.json"

fetch_pages "repos/$REPO/milestones?state=all&per_page=100" "$SCRATCH/raw/milestones.pages"
jq -s 'add // []' "$SCRATCH/raw/milestones.pages" > "$JSON_DIR/milestones.json"

fetch_pages "repos/$REPO/issues/comments?per_page=100&sort=created&direction=asc" "$SCRATCH/raw/issue-comments.pages"
jq -s 'add // []' "$SCRATCH/raw/issue-comments.pages" > "$SCRATCH/issue-comments.json"

fetch_pages "repos/$REPO/pulls/comments?per_page=100&sort=created&direction=asc" "$SCRATCH/raw/review-comments.pages"
jq -s 'add // []' "$SCRATCH/raw/review-comments.pages" > "$SCRATCH/review-comments.json"

TOTAL_ISSUE_COMMENTS="$(jq 'length' "$SCRATCH/issue-comments.json")"
TOTAL_REVIEW_COMMENTS="$(jq 'length' "$SCRATCH/review-comments.json")"
log "comments fetched: $TOTAL_ISSUE_COMMENTS conversation, $TOTAL_REVIEW_COMMENTS inline review"

# live reconciliation numbers (search API is a separate, small budget)
OPEN_ISSUES_COUNT="$(gh_json "repos/$REPO" .open_issues_count     || echo 'unavailable')"
SEARCH_ISSUES="$(    gh_json "search/issues?q=repo:$REPO+type:issue&per_page=1"         .total_count || echo 'unavailable')"
SEARCH_PRS="$(       gh_json "search/issues?q=repo:$REPO+type:pr&per_page=1"            .total_count || echo 'unavailable')"
SEARCH_OPEN_ISSUES="$(gh_json "search/issues?q=repo:$REPO+type:issue+state:open&per_page=1" .total_count || echo 'unavailable')"
SEARCH_OPEN_PRS="$(   gh_json "search/issues?q=repo:$REPO+type:pr+state:open&per_page=1"    .total_count || echo 'unavailable')"
LIVE_TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
log "live counts: open_issues_count=$OPEN_ISSUES_COUNT search(issue)=$SEARCH_ISSUES search(pr)=$SEARCH_PRS"

# ---------------------------------------------------------------- phase 2: split + verify

log "phase 2: split per item and verify comment counts"

jq -r '.[] | "\(.number)\t\(tojson)"' "$SCRATCH/issues.json" | while IFS=$'\t' read -r n obj; do
  printf '%s\n' "$obj" > "$SCRATCH/item/$n.issue.json"
done

jq -r '.[] | "\(.number)\t\(tojson)"' "$SCRATCH/pulls.json" | while IFS=$'\t' read -r n obj; do
  printf '%s\n' "$obj" > "$SCRATCH/item/$n.pull.json"
done

jq 'group_by(.issue_url | split("/") | last | tonumber)
    | map({key: (.[0].issue_url | split("/") | last), value: (sort_by(.created_at, .id))})
    | from_entries' "$SCRATCH/issue-comments.json" > "$SCRATCH/comments-by-issue.json"

jq 'group_by(.pull_request_url | split("/") | last | tonumber)
    | map({key: (.[0].pull_request_url | split("/") | last), value: (sort_by(.created_at, .id))})
    | from_entries' "$SCRATCH/review-comments.json" > "$SCRATCH/review-comments-by-pull.json"

jq -r 'to_entries[] | "\(.key)\t\(.value | tojson)"' "$SCRATCH/comments-by-issue.json" \
  | while IFS=$'\t' read -r n arr; do printf '%s\n' "$arr" > "$SCRATCH/item/$n.comments.json"; done

jq -r 'to_entries[] | "\(.key)\t\(.value | tojson)"' "$SCRATCH/review-comments-by-pull.json" \
  | while IFS=$'\t' read -r n arr; do printf '%s\n' "$arr" > "$SCRATCH/item/$n.review-comments.json"; done

while IFS=$'\t' read -r n kind; do
  [[ -f "$SCRATCH/item/$n.comments.json" ]]        || printf '[]\n' > "$SCRATCH/item/$n.comments.json"
  [[ -f "$SCRATCH/item/$n.review-comments.json" ]] || printf '[]\n' > "$SCRATCH/item/$n.review-comments.json"
done < "$SCRATCH/items.tsv"

# orphan check: comments fetched but not attached to a snapshot item
ORPHAN_COMMENTS=$(( TOTAL_ISSUE_COMMENTS - $(jq '[.[] | length] | add // 0' "$SCRATCH/comments-by-issue.json") ))
ORPHAN_REVIEW_COMMENTS=$(( TOTAL_REVIEW_COMMENTS - $(jq '[.[] | length] | add // 0' "$SCRATCH/review-comments-by-pull.json") ))
if (( ORPHAN_COMMENTS != 0 || ORPHAN_REVIEW_COMMENTS != 0 )); then
  log "WARNING: $ORPHAN_COMMENTS orphan conversation comments, $ORPHAN_REVIEW_COMMENTS orphan review comments"
fi

# verify conversation comments against each item's `comments` counter
: > "$SCRATCH/comments-mismatch.txt"
while IFS=$'\t' read -r n kind; do
  expected="$(jq -r '.comments' "$SCRATCH/item/$n.issue.json")"
  actual="$(jq 'length' "$SCRATCH/item/$n.comments.json")"
  if [[ "$expected" != "$actual" ]]; then printf '%s\t%s\n' "$n" "$expected" >> "$SCRATCH/comments-mismatch.txt"; fi
done < "$SCRATCH/items.tsv"

CMT_EXACT=$(( EXPECTED_TOTAL - $(wc -l < "$SCRATCH/comments-mismatch.txt" | tr -d ' ') ))
CMT_REFETCH=0
CMT_UNRESOLVED=0
if (( $(wc -l < "$SCRATCH/comments-mismatch.txt" | tr -d ' ') > 0 )); then
  log "comment-count mismatches to resolve by direct fetch: $(wc -l < "$SCRATCH/comments-mismatch.txt" | tr -d ' ')"
  while IFS=$'\t' read -r n expected; do
    fetch_pages "repos/$REPO/issues/$n/comments?per_page=100" "$SCRATCH/raw/cmt-$n.pages"
    jq -s 'add // []' "$SCRATCH/raw/cmt-$n.pages" > "$SCRATCH/item/$n.comments.json"
    actual="$(jq 'length' "$SCRATCH/item/$n.comments.json")"
    if [[ "$expected" =~ ^[0-9]+$ ]] && (( actual >= expected )); then
      CMT_REFETCH=$((CMT_REFETCH + 1))
      log "  #$n resolved by refetch ($expected expected, $actual fetched)"
    else
      CMT_UNRESOLVED=$((CMT_UNRESOLVED + 1))
      log "  #$n UNRESOLVED ($expected expected, $actual fetched)"
    fi
  done < "$SCRATCH/comments-mismatch.txt"
fi

# Verify inline review comments. GitHub's list-pulls representation omits the per-PR
# `review_comments` counter, so verification uses a direct single-PR GET for every PR that has
# any inline review comments in the bulk snapshot (all of them), in addition to the global
# orphan check above. A grouped count below the live count is re-fetched; anything else is flagged.
N_PR="$(wc -l < "$SCRATCH/pr-numbers.txt" | tr -d ' ')"
: > "$SCRATCH/prs-with-review-comments.tsv"
jq -r 'to_entries[] | select(.value | length > 0) | "\(.key)\t\(.value | length)"' \
  "$SCRATCH/review-comments-by-pull.json" > "$SCRATCH/prs-with-review-comments.tsv"
RC_EXACT=0
RC_REFETCH=0
RC_UNRESOLVED=0
RC_VERIFIED=0
while IFS=$'\t' read -r n grouped; do
  [[ -n "$n" ]] || continue
  expected="$(gh_json "repos/$REPO/pulls/$n" .review_comments)"
  RC_VERIFIED=$((RC_VERIFIED + 1))
  if [[ "$expected" == "$grouped" ]]; then
    RC_EXACT=$((RC_EXACT + 1))
  elif [[ "$expected" =~ ^[0-9]+$ ]] && (( expected > grouped )); then
    fetch_pages "repos/$REPO/pulls/$n/comments?per_page=100" "$SCRATCH/raw/rc-$n.pages"
    jq -s 'add // []' "$SCRATCH/raw/rc-$n.pages" > "$SCRATCH/item/$n.review-comments.json"
    actual="$(jq 'length' "$SCRATCH/item/$n.review-comments.json")"
    if (( actual >= expected )); then
      RC_REFETCH=$((RC_REFETCH + 1))
      log "  PR #$n review comments resolved by refetch ($expected live, $actual fetched)"
    else
      RC_UNRESOLVED=$((RC_UNRESOLVED + 1))
      log "  PR #$n review comments UNRESOLVED ($expected expected, $actual fetched)"
    fi
  else
    RC_UNRESOLVED=$((RC_UNRESOLVED + 1))
    log "  PR #$n review comments UNRESOLVED (grouped=$grouped, live=$expected)"
  fi
  printf '%s\t%s\t%s\n' "$n" "$grouped" "$expected" >> "$SCRATCH/review-comments-verify.txt"
done < "$SCRATCH/prs-with-review-comments.tsv"
log "verification: conversation comments exact=$CMT_EXACT refetch=$CMT_REFETCH unresolved=$CMT_UNRESOLVED; review comments exact=$RC_EXACT/$RC_VERIFIED verified refetch=$RC_REFETCH unresolved=$RC_UNRESOLVED"

# ---------------------------------------------------------------- phase 3: per-PR reviews

log "phase 3: reviews for $N_PR PRs"
DONE=0
REUSED=0
while read -r n; do
  DONE=$((DONE + 1))
  if (( DONE % 100 == 0 )); then check_rate_budget; log "  reviews $DONE/$N_PR"; fi
  if [[ -f "$SCRATCH/reviews/$n.json" ]] && jq -e 'type == "array"' "$SCRATCH/reviews/$n.json" >/dev/null 2>&1; then
    REUSED=$((REUSED + 1))
  else
    fetch_pages "repos/$REPO/pulls/$n/reviews?per_page=100" "$SCRATCH/raw/reviews-$n.pages"
    jq -s 'add // []' "$SCRATCH/raw/reviews-$n.pages" > "$SCRATCH/reviews/$n.json"
  fi
done < "$SCRATCH/pr-numbers.txt"
REVIEWS_TOTAL="$(find "$SCRATCH/reviews" -name '*.json' -print0 | xargs -0 jq -s 'map(length) | add // 0' | awk '{s += $1} END {print s + 0}')"
log "reviews fetched: $REVIEWS_TOTAL total (reused from scratch: $REUSED)"

# ---------------------------------------------------------------- phase 4: build json + markdown

log "phase 4: rendering $EXPECTED_TOTAL items"
: > "$SCRATCH/index-rows.tsv"
RENDERED=0
while IFS=$'\t' read -r n kind; do
  RENDERED=$((RENDERED + 1))
  if (( RENDERED % 200 == 0 )); then log "  rendered $RENDERED/$EXPECTED_TOTAL"; fi

  title="$(jq -r '.title' "$SCRATCH/item/$n.issue.json")"
  slug="$(slugify "$title")"
  fname="$(printf '%04d' "$n")-$slug.md"

  if [[ "$kind" == "pr" ]]; then
    sub="pulls"
    [[ -f "$SCRATCH/item/$n.pull.json" ]]          || printf 'null\n' > "$SCRATCH/item/$n.pull.json"
    [[ -f "$SCRATCH/reviews/$n.json" ]]            || printf '[]\n'   > "$SCRATCH/reviews/$n.json"
    jq -n --arg exported "$EXPORT_TS" --arg repo "$REPO" \
      --slurpfile issue    "$SCRATCH/item/$n.issue.json" \
      --slurpfile pull     "$SCRATCH/item/$n.pull.json" \
      --slurpfile comments "$SCRATCH/item/$n.comments.json" \
      --slurpfile reviews  "$SCRATCH/reviews/$n.json" \
      --slurpfile rcmts    "$SCRATCH/item/$n.review-comments.json" \
      '{exported_at: $exported, source_repo: $repo, is_pull_request: true,
        issue: $issue[0], pull: ($pull[0] // null), comments: $comments[0],
        reviews: $reviews[0], review_comments: $rcmts[0]}' \
      > "$JSON_DIR/pulls/$n.json"
    jsonfile="$JSON_DIR/pulls/$n.json"
  else
    sub="issues"
    jq -n --arg exported "$EXPORT_TS" --arg repo "$REPO" \
      --slurpfile issue    "$SCRATCH/item/$n.issue.json" \
      --slurpfile comments "$SCRATCH/item/$n.comments.json" \
      '{exported_at: $exported, source_repo: $repo, is_pull_request: false,
        issue: $issue[0], comments: $comments[0]}' \
      > "$JSON_DIR/issues/$n.json"
    jsonfile="$JSON_DIR/issues/$n.json"
  fi

  jq -r -f "$SCRATCH/md.jq" "$jsonfile" > "$MD_DIR/$fname"

  jq -r --arg jsonpath "json/$sub/$n.json" --arg mdpath "markdown/$fname" --arg fallback "$EXPORT_TS" '
    [ .issue.number,
      (if .is_pull_request then "PR" else "issue" end),
      .issue.state,
      .issue.title,
      ((.comments | length) | tostring),
      (([.issue.updated_at] + [(.comments // [])[] | .created_at]
        + [(.reviews // [])[] | .submitted_at] + [(.review_comments // [])[] | .created_at])
       | map(select(. != null and . != "")) | max // $fallback),
      $jsonpath, $mdpath ] | @tsv' "$jsonfile" >> "$SCRATCH/index-rows.tsv"
done < "$SCRATCH/items.tsv"

# ---------------------------------------------------------------- phase 5: checks

log "phase 5: completeness checks"
EXPORTED_ISSUES="$(awk -F'\t' '$2 == "issue"' "$SCRATCH/items.tsv" | wc -l | tr -d ' ')"
EXPORTED_PRS="$(awk -F'\t' '$2 == "pr"' "$SCRATCH/items.tsv" | wc -l | tr -d ' ')"
EXPORTED_OPEN_ISSUES="$(jq '[.[] | select(.pull_request == null and .state == "open")] | length' "$SCRATCH/issues.json")"
EXPORTED_OPEN_PRS="$(jq '[.[] | select(.pull_request != null and .state == "open")] | length' "$SCRATCH/issues.json")"
PULLS_LIST_COUNT="$(jq 'length' "$SCRATCH/pulls.json")"
SUM_ISSUE_COMMENTS="$(jq '[.[].comments] | add // 0' "$SCRATCH/issues.json")"
GROUPED_REVIEW_COMMENTS="$(jq '[.[] | length] | add // 0' "$SCRATCH/review-comments-by-pull.json")"
JSON_ISSUE_FILES="$(find "$JSON_DIR/issues" -name '*.json' | wc -l | tr -d ' ')"
JSON_PULL_FILES="$(find "$JSON_DIR/pulls" -name '*.json' | wc -l | tr -d ' ')"
MD_FILES="$(find "$MD_DIR" -name '*.md' | wc -l | tr -d ' ')"

SPOT_IDS_ARR=(6181921625 6181460874 6174007797 6174496690 6159308818)
if [[ -n "${SPOT_IDS:-}" ]]; then read -r -a SPOT_IDS_ARR <<< "$SPOT_IDS"; fi
SPOT_FAILS=0
{
  printf '| comment id | item | in json | markdown heading | body matches live API |\n'
  printf '| ---: | --- | --- | --- | --- |\n'
} > "$SCRATCH/spot-rows.md"

for id in "${SPOT_IDS_ARR[@]}"; do
  kind="issue-comment"
  item_num="$(jq -r --argjson id "$id" '.[] | select(.id == $id) | .issue_url | split("/") | last' "$SCRATCH/issue-comments.json" | head -1)"
  if [[ -z "$item_num" ]]; then
    kind="review-comment"
    item_num="$(jq -r --argjson id "$id" '.[] | select(.id == $id) | .pull_request_url | split("/") | last' "$SCRATCH/review-comments.json" | head -1)"
  fi
  if [[ -z "$item_num" ]]; then
    printf '| %s | NOT FOUND in bulk sets | no | no | no |\n' "$id" >> "$SCRATCH/spot-rows.md"
    log "SPOT CHECK FAILED: $id not in bulk comment sets"
    SPOT_FAILS=$((SPOT_FAILS + 1))
    continue
  fi
  if [[ "$kind" == "issue-comment" ]]; then
    jfile="$JSON_DIR/issues/$item_num.json"
    found="$(jq --argjson id "$id" '[.comments[] | select(.id == $id)] | length' "$jfile")"
    exported_body="$(jq -r --argjson id "$id" '[.comments[] | select(.id == $id)][0].body' "$jfile")"
    heading_pat="^### comment $id —"
  else
    jfile="$JSON_DIR/pulls/$item_num.json"
    found="$(jq --argjson id "$id" '[.review_comments[] | select(.id == $id)] | length' "$jfile")"
    exported_body="$(jq -r --argjson id "$id" '[.review_comments[] | select(.id == $id)][0].body' "$jfile")"
    heading_pat="^### review-comment $id —"
  fi
  if [[ "$found" -gt 0 ]]; then in_json="yes"; else in_json="NO"; fi
  title="$(jq -r '.issue.title' "$jfile")"
  fname="$(printf '%04d' "$item_num")-$(slugify "$title").md"
  heading="NO"
  if [[ -f "$MD_DIR/$fname" ]] && grep -q -- "$heading_pat" "$MD_DIR/$fname"; then heading="yes"; fi
  live_body="$(gh_json "repos/$REPO/issues/comments/$id" .body || true)"
  if [[ -z "$live_body" && "$kind" == "review-comment" ]]; then
    live_body="$(gh_json "repos/$REPO/pulls/comments/$id" .body || true)"
  fi
  if [[ "$exported_body" == "$live_body" ]]; then body_match="yes"; else body_match="NO"; fi
  printf '| %s | #%s (%s) | %s | %s | %s |\n' "$id" "$item_num" "$kind" "$in_json" "$heading" "$body_match" >> "$SCRATCH/spot-rows.md"
  log "spot check $id: json=$in_json heading=$heading body_match=$body_match (item #$item_num)"
  if [[ "$in_json" != "yes" || "$heading" != "yes" || "$body_match" != "yes" ]]; then SPOT_FAILS=$((SPOT_FAILS + 1)); fi
done

# ---------------------------------------------------------------- phase 6: INDEX.md

log "phase 6: writing INDEX.md"
{
  printf '# GitHub archive — %s (issues, PRs, comments)\n\n' "$REPO"
  printf -- '- Source repository: https://github.com/%s (public)\n' "$REPO"
  printf -- '- Export timestamp: %s (UTC; GitHub API snapshot)\n' "$EXPORT_TS"
  printf -- '- Issues/PRs snapshot: %s · live reconciliation counts: %s\n' "$SNAPSHOT_TS" "$LIVE_TS"
  printf -- '- Archive worktree base commit: %s (origin/main at export time)\n' "$BASE_COMMIT"
  printf -- '- Generator: export.sh in this directory (re-runnable; its header documents the method)\n'
  cat <<EOF
- Comment ids are GitHub-global. Every comment keeps its original numeric id (e.g. 6181921625) and stays
  resolvable at the source repository while it exists and is readable:
      gh api repos/$REPO/issues/comments/<id>
  In this archive a comment is located by its heading; for example:
      grep -rn "### comment 6181921625" markdown/
- Snapshot semantics: the repository is active — items or comments created after the snapshot timestamps
  are not included. The counts below reconcile this export against live counts captured at export time.
- Layout: json/issues/<number>.json · json/pulls/<number>.json · json/labels.json · json/milestones.json ·
  markdown/NNNN-<slug>.md (chronological; each comment headed "### comment <id> — <created_at> — <login>")

## Reconciliation

| measure | exported | live | check |
| --- | ---: | ---: | --- |
| issues (non-PR, all states) | $EXPORTED_ISSUES | $SEARCH_ISSUES (search type:issue) | $(verdict "$EXPORTED_ISSUES" "$SEARCH_ISSUES") |
| pull requests (all states) | $EXPORTED_PRS | $SEARCH_PRS (search type:pr) | $(verdict "$EXPORTED_PRS" "$SEARCH_PRS") |
| PRs in issues list vs pulls list | $EXPORTED_PRS | $PULLS_LIST_COUNT | $(verdict "$EXPORTED_PRS" "$PULLS_LIST_COUNT") |
| open issues | $EXPORTED_OPEN_ISSUES | $SEARCH_OPEN_ISSUES (search type:issue+state:open) | $(verdict "$EXPORTED_OPEN_ISSUES" "$SEARCH_OPEN_ISSUES") |
| open PRs | $EXPORTED_OPEN_PRS | $SEARCH_OPEN_PRS (search type:pr+state:open) | $(verdict "$EXPORTED_OPEN_PRS" "$SEARCH_OPEN_PRS") |
| open issues + open PRs | $(( EXPORTED_OPEN_ISSUES + EXPORTED_OPEN_PRS )) | $OPEN_ISSUES_COUNT (repos.open_issues_count) | $(verdict "$(( EXPORTED_OPEN_ISSUES + EXPORTED_OPEN_PRS ))" "$OPEN_ISSUES_COUNT") |
| conversation comments | $TOTAL_ISSUE_COMMENTS | $SUM_ISSUE_COMMENTS (sum of item comment counters) | $(verdict "$TOTAL_ISSUE_COMMENTS" "$SUM_ISSUE_COMMENTS") |
| inline review comments | $TOTAL_REVIEW_COMMENTS | $GROUPED_REVIEW_COMMENTS attached to $RC_VERIFIED PRs | $(verdict "$TOTAL_REVIEW_COMMENTS" "$GROUPED_REVIEW_COMMENTS") |
| PR reviews (per-PR pulls/<n>/reviews) | $REVIEWS_TOTAL | — | — |
| orphan comments not attached to a snapshot item | $(( ORPHAN_COMMENTS + ORPHAN_REVIEW_COMMENTS )) | 0 | $(verdict "$(( ORPHAN_COMMENTS + ORPHAN_REVIEW_COMMENTS ))" 0) |
| per-item comment-count verification (issues + PRs) | $CMT_EXACT/$EXPECTED_TOTAL exact | $CMT_REFETCH resolved by refetch, $CMT_UNRESOLVED unresolved | — |
| review-comment verification vs live single-PR GET | $RC_EXACT/$RC_VERIFIED exact | $RC_REFETCH resolved by refetch, $RC_UNRESOLVED unresolved | — |
| files: json/issues | $JSON_ISSUE_FILES | $EXPORTED_ISSUES | $(verdict "$JSON_ISSUE_FILES" "$EXPORTED_ISSUES") |
| files: json/pulls | $JSON_PULL_FILES | $EXPORTED_PRS | $(verdict "$JSON_PULL_FILES" "$EXPORTED_PRS") |
| files: markdown | $MD_FILES | $EXPECTED_TOTAL | $(verdict "$MD_FILES" "$EXPECTED_TOTAL") |

Notes on deltas: the search API totals are eventually consistent and may lag the snapshot; the item
comment counters are snapshotted slightly before the comment listings, so a live repo can show small
offsetting differences. Mismatches were resolved by re-fetching the item directly (see "resolved by
refetch" rows); "unresolved" must be zero for a complete archive.

Note on the API representation: GitHub's list-pulls endpoint omits per-PR diff stats (additions,
deletions, changed_files, commits) and the comments/review_comments counters, so json/pulls/<n>.json
stores the list representation — merged_at, merge_commit_sha, base/head, user, dates, labels and body
are present — plus separately fetched reviews and inline review comments.

## Spot checks (high-citation comment ids)

EOF
  cat "$SCRATCH/spot-rows.md"
  printf '\n## Contents\n\n'
  printf '| # | type | state | title | comments | last activity | json | markdown |\n'
  printf '| ---: | --- | --- | --- | ---: | --- | --- | --- |\n'
  sort -t$'\t' -k1,1n "$SCRATCH/index-rows.tsv" | awk -F'\t' '
    { t=$4; gsub(/\|/, "\\|", t);
      printf "| #%s | %s | %s | %s | %s | %s | [%s](%s) | [%s](%s) |\n", $1,$2,$3,t,$5,$6,$7,$7,$8,$8 }'
} > "$ARCHIVE_DIR/INDEX.md"

log "===== EXPORT COMPLETE ====="
log "items: $EXPECTED_TOTAL ($EXPORTED_ISSUES issues + $EXPORTED_PRS PRs) as of $SNAPSHOT_TS"
log "comments: $TOTAL_ISSUE_COMMENTS conversation + $TOTAL_REVIEW_COMMENTS review comments + $REVIEWS_TOTAL reviews"
log "spot check failures: $SPOT_FAILS"
log "INDEX: $ARCHIVE_DIR/INDEX.md"
