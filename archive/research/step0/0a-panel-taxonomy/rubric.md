# Step 0a failure-taxonomy rubric (fixed before any row was labelled)

Unit: one graded panel row that is NOT acceptable (acceptable = grader judged both fluent and relevant).
The annotator (Claude) reads the full conversation (user turns, any earlier assistant turn, and the model's last reply)
and assigns exactly ONE primary label. Grader verdicts (fluent/relevant) are shown but do not decide the label.

Labels:
- R  recall-anaphora-consistency: the request depends on content the model must carry from an EARLIER TURN or from
     material SUPPLIED INSIDE THE REQUEST (a passage, list, numbers, names, a code snippet, a quoted sentence,
     a pronoun/anaphor resolved by prior context), and the reply fails to use it, uses it wrongly, or contradicts it.
     A request that merely names a topic ("tell me about dogs") is not context-dependent.
- I  instruction: the reply attempts the topic but ignores the requested task type, format or constraint
     (asked for a list/poem/code/short answer/N items/translation/rewrite and got something else; answered a
     different question type, e.g. tells a story instead of giving advice).
- K  knowledge-absent: the request needs world/domain knowledge (facts, definitions, procedures, arithmetic,
     how-to steps, domain advice) and the reply lacks it, invents wrong facts, or substitutes generic filler while
     being readable English in the right task form.
- N  incoherent: the reply is ungrammatical, word salad, a repetition loop, self-contradictory within itself, or
     drifts into unrelated narrative such that no attempt at the request is identifiable.
- T  truncation: the reply is on track (right task, sensible content) and fails mainly because it stops at the
     64-token budget mid-sentence/mid-list.
- O  other: grader false negative (reply looks acceptable to the annotator), refusal/deflection with no attempt,
     role confusion (writes the user's turn), empty reply, or anything not covered.

Precedence when several apply (fixed): N > R > I > K > T > O.
  N first: if no attempt is identifiable, no finer diagnosis is meaningful.
  R second: deliberately generous to retrieval, so a low R share is a conservative result for the 0a decision rule.
  I before K: a wrong task form is diagnosed before content quality.
  T only when nothing above applies.

Secondary flag (recorded for every row, independent of primary label):
  ctx = 1 if the request is context-dependent in the R sense (whether or not the reply failed on it).
  r_any = 1 if R applies at all, even when a higher-precedence label (N) won. Reported as an upper bound.

Self-agreement: after all labels are fixed, a seeded random 40-row subset of the 96M failing rows is re-labelled
from a fresh dump that omits the first labels (blind pass, rubric only). Cohen's kappa over the 6 primary labels.

Decision rule (0a, from final-assessment.md Step 0, numeric):
  if R share of 96M failing rows < 15%: "retrieval cannot move the open panel; panel lever = corpus/knowledge (Step 5)"
  else: "retrieval can move the panel".
