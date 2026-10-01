//! Offline answer oracle for authored evaluation cases.
//!
//! Accepted answer strings are authored once from the request's typed intent and
//! frozen in the case file, so every evaluator (the core driver and the native API
//! check) judges the same list. The list is membership only: no evaluator may derive
//! an alternative from the response, and a previous-location request never accepts a
//! present-tense statement of its old value.
use serde::{Deserialize, Serialize};

/// What the request asks for; the accepted forms follow from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    /// The first stored version: `VALUE.` or `OWNER was in VALUE.`
    Initial,
    /// The immediate previous version: `VALUE.` or `OWNER was in VALUE.`
    Previous,
    /// The live version: `VALUE.` or `OWNER is in VALUE.`
    Current,
    /// A proven-evicted chain: `Unknown.` only.
    Abstain,
}

/// The recorded-memory request or outcome described by an authored answer set.
/// These distinguish record order from the chronology of the outside world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordedValueIntent {
    Current,
    Initial,
    PreviousAssertion,
    PreviousDistinctValue,
    Absent,
    NoHistory,
    Evicted,
    Unresolved,
}

/// Complete forms authored once for a typed case, before candidate replies exist.
/// Validation checks the list's structure, not the semantic truth of its forms.
/// The caller binds these forms to the case's events and original file bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenAnswers {
    pub intent: RecordedValueIntent,
    pub accepted: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrozenAnswersError {
    Empty,
    BlankAnswer { index: usize },
    DuplicateAnswer { first: usize, index: usize },
}

impl std::fmt::Display for FrozenAnswersError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "a frozen answer set needs at least one complete form"),
            Self::BlankAnswer { index } => write!(f, "frozen answer {index} is blank"),
            Self::DuplicateAnswer { first, index } => {
                write!(f, "frozen answer {index} duplicates answer {first}")
            }
        }
    }
}

impl std::error::Error for FrozenAnswersError {}

impl FrozenAnswers {
    /// Reject missing, blank or byte-identical duplicate forms. Leading and
    /// trailing whitespace remains significant; no normalization is applied.
    pub fn validate(&self) -> Result<(), FrozenAnswersError> {
        if self.accepted.is_empty() {
            return Err(FrozenAnswersError::Empty);
        }
        let mut seen = std::collections::BTreeMap::new();
        for (index, answer) in self.accepted.iter().enumerate() {
            if answer.trim().is_empty() {
                return Err(FrozenAnswersError::BlankAnswer { index });
            }
            if let Some(first) = seen.insert(answer.as_str(), index) {
                return Err(FrozenAnswersError::DuplicateAnswer { first, index });
            }
        }
        Ok(())
    }

    /// Exact membership only. Validate the authored case before execution;
    /// candidate text never creates or changes an accepted alternative.
    pub fn accepts(&self, text: &str) -> bool {
        accepts(&self.accepted, text)
    }
}

/// Accepted complete answer strings for one request. A plain request (no owner-first
/// instruction) accepts the bare value or the frozen emitter's owner sentence in the
/// tense of the intent; an owner-first request accepts only that sentence.
pub fn accepted(intent: Intent, plain: bool, owner: &str, value: &str) -> Vec<String> {
    match intent {
        Intent::Abstain => vec![" Unknown.\n".to_owned()],
        Intent::Initial | Intent::Previous => sentence(plain, owner, "was", value),
        Intent::Current => sentence(plain, owner, "is", value),
    }
}

fn sentence(plain: bool, owner: &str, verb: &str, value: &str) -> Vec<String> {
    let owner_form = format!(" {owner} {verb} in {value}.\n");
    if plain {
        vec![format!(" {value}.\n"), owner_form]
    } else {
        vec![owner_form]
    }
}

/// Accepted complete answers for a current request that carries the explanatory
/// instruction (`Explain in a sentence.`). With an explicit owner-first instruction only
/// the present-tense owner sentence is accepted. Without it the inherited
/// `VALUE is the place.` form is a legitimate declared form and the owner sentence
/// remains acceptable; the bare value is not an explanation.
pub fn accepted_explanatory(owner_first: bool, owner: &str, value: &str) -> Vec<String> {
    let owner_form = format!(" {owner} is in {value}.\n");
    if owner_first {
        vec![owner_form]
    } else {
        vec![format!(" {value} is the place.\n"), owner_form]
    }
}

/// Membership in the frozen list; nothing else.
pub fn accepts(accepted: &[String], text: &str) -> bool {
    accepted.iter().any(|a| a == text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_answers_validate_structure_without_normalizing_forms() {
        let mut answers = FrozenAnswers {
            intent: RecordedValueIntent::PreviousDistinctValue,
            accepted: Vec::new(),
        };
        assert_eq!(answers.validate(), Err(FrozenAnswersError::Empty));
        for blank in ["", " \t\r\n"] {
            answers.accepted = vec![blank.into()];
            assert_eq!(
                answers.validate(),
                Err(FrozenAnswersError::BlankAnswer { index: 0 })
            );
        }
        answers.accepted = vec!["Aster.".into(), "Aster.".into()];
        assert_eq!(
            answers.validate(),
            Err(FrozenAnswersError::DuplicateAnswer { first: 0, index: 1 })
        );
        answers.accepted = vec!["Aster.".into(), " Aster.\n".into()];
        let before = answers.clone();
        assert_eq!(answers.validate(), Ok(()));
        assert_eq!(answers, before);
    }

    #[test]
    fn frozen_answers_require_the_complete_authored_reply_and_strict_schema() {
        let answers = FrozenAnswers {
            intent: RecordedValueIntent::Initial,
            accepted: vec![" You first recorded Aster.\n".into()],
        };
        assert_eq!(answers.validate(), Ok(()));
        assert!(answers.accepts(" You first recorded Aster.\n"));
        for wrong in [
            "You first recorded Aster.",
            " Aster.\n",
            " You currently record Aster.\n",
            " You first recorded Aster.\nActually, no.",
        ] {
            assert!(!answers.accepts(wrong));
        }
        let encoded = serde_json::to_vec(&answers).expect("answer set");
        assert_eq!(
            serde_json::from_slice::<FrozenAnswers>(&encoded).expect("reload"),
            answers
        );
        assert!(serde_json::from_str::<FrozenAnswers>(
            r#"{"intent":"initial","accepted":["Aster."],"extra":true}"#
        )
        .is_err());
        assert!(serde_json::from_str::<FrozenAnswers>(
            r#"{"intent":"previous","accepted":["Aster."]}"#
        )
        .is_err());
    }

    #[test]
    fn answer_oracle_explanatory_owner_first_requires_present_tense_owner_sentence() {
        let both = accepted_explanatory(true, "pemru", "amber quay");
        assert_eq!(both, vec![" pemru is in amber quay.\n".to_owned()]);
        assert!(!accepts(&both, " amber quay is the place.\n"));
        assert!(!accepts(&both, " pemru was in amber quay.\n"));
        assert!(!accepts(&both, " quay holds pemru is the place.\n"));
        assert!(!accepts(&both, " amber quay.\n"));
    }

    #[test]
    fn answer_oracle_explanatory_only_accepts_inherited_place_form_or_owner_sentence() {
        let explain = accepted_explanatory(false, "pemru", "amber quay");
        assert!(accepts(&explain, " amber quay is the place.\n"));
        assert!(accepts(&explain, " pemru is in amber quay.\n"));
        assert!(!accepts(&explain, " amber quay.\n"));
        assert!(!accepts(&explain, " pemru was in amber quay.\n"));
    }

    #[test]
    fn answer_oracle_previous_request_rejects_present_tense_old_value() {
        let plain = accepted(Intent::Previous, true, "selvi", "Copper Vale");
        assert_eq!(
            plain,
            vec![" Copper Vale.\n", " selvi was in Copper Vale.\n"]
        );
        assert!(!accepts(&plain, " selvi is in Copper Vale.\n"));
        assert!(accepts(&plain, " Copper Vale.\n"));
        let owner_first = accepted(Intent::Previous, false, "selvi", "Copper Vale");
        assert_eq!(owner_first, vec![" selvi was in Copper Vale.\n"]);
        assert!(!accepts(&owner_first, " Copper Vale.\n"));
        assert!(!accepts(&owner_first, " selvi is in Copper Vale.\n"));
    }

    #[test]
    fn answer_oracle_current_initial_and_abstain_forms_are_typed() {
        assert_eq!(
            accepted(Intent::Current, true, "selvi", "Amber Field"),
            vec![" Amber Field.\n", " selvi is in Amber Field.\n"]
        );
        assert!(!accepts(
            &accepted(Intent::Current, false, "selvi", "Amber Field"),
            " selvi was in Amber Field.\n"
        ));
        assert_eq!(
            accepted(Intent::Initial, false, "selvi", "Dusk Ridge"),
            vec![" selvi was in Dusk Ridge.\n"]
        );
        let abstain = accepted(Intent::Abstain, true, "selvi", "Dusk Ridge");
        assert_eq!(abstain, vec![" Unknown.\n"]);
        assert!(!accepts(&abstain, " Dusk Ridge.\n"));
        assert!(!accepts(&abstain, " selvi was in Dusk Ridge.\n"));
        assert!(!accepts(&[], "anything"));
    }
}
