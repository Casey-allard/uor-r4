# Paraphrase label review rubric (frozen 2026-10-01 before any row is read)

Scope: Claude's raw teacher paraphrases, `paraphrases-1` (373 rows, sha256
ce816856…) and `paraphrases-2` (403 rows, sha256 b9a5f451…). Every row gets
exactly one decision. Raw files are never edited.

Label contract (M-world v2, D19 grounded session):
- relation: one of user_name, pet_name, friend_name, hometown, lucky_number,
  code_word, job, home, favorite_food, favorite_color — the USER's own value.
- act `assert`: the user states the value (first statement, no correction cue).
- act `update`: the user explicitly replaces or corrects an earlier value
  (actually, now, changed, sorry/meant, renamed, new, switched…).
- act `query`: the user asks for the current value. Contains no `{v}`.

Decisions:
- `keep`: the text is a clean instance of its inherited relation and act; for
  assert/update, `{v}` occurs exactly once and fills the value role.
- `relabel`: the text is a clean, unambiguous instance of a different
  relation and/or act (the new labels are given). Same slot rules.
- `drop`: any of
  - meaning changed (another attribute, birthplace vs upbringing, current
    location vs hometown, family vs home, responsibilities vs job,
    preference vs lucky number);
  - person changed (the assistant's or another person's value, "who am I");
  - meta or confirmation question instead of a value request;
  - `{v}` misused (as a modifier, "a {v} friend", "surrounded by {v}"),
    missing in a statement, present in a query, or repeated;
  - ambiguous between two relations or between assert and update, where a
    reasonable reader could not decide from the text alone;
  - ungrammatical or not a plausible user turn.
- `duplicate`: identical text (exact string) to an earlier row of the union
  in order p1 then p2; only the first occurrence is reviewed on its merits.

Borderline rule: when unsure between keep and drop, drop (a smaller clean set
is preferred to a noisy larger one). Generic-but-valid wordings (e.g. a pet
for a dog) are kept when the relation stays the single designated value.

Reviewer disclosure: the reviewer (Claude) has seen development-cell results
and some development wordings in session outputs. Decisions judge only label
correctness of the given text; no row is written, edited or chosen to resemble
a development wording. The derivative only keeps, relabels or drops rows.
