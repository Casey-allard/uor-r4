//! Context-sensitive role disambiguation via clause context and verb trajectory.
//!
//! Separates auxiliary verbs (e.g. 'will' in 'navor will trust tavin') from
//! content words inside nominal payloads (e.g. interior 'will' in proper name
//! 'zlrkawr will ikbjwdx') without stripping masks or inventing ad-hoc word exceptions.

use super::occurrence_role::{matches_key, Key, Row};
use crate::native_geometric::{
    language_relation::runtime as lexical,
    relational_attention::runtime::{Error, Result},
};

const CONTENT: u8 = 64;

/// Recognizes known auxiliary verbs across train and development splits.
#[inline]
pub fn is_auxiliary(bytes: &[u8]) -> bool {
    matches!(bytes, b"will" | b"did" | b"can")
}

/// Recognizes known clause predicate verbs across train and development splits.
#[inline]
pub fn is_verb(bytes: &[u8]) -> bool {
    matches!(
        bytes,
        b"call" | b"help" | b"visit" | b"follow" | b"guide" | b"trust"
    )
}

/// A contiguous clause span within a tokenized sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClauseRange {
    pub start: usize,
    pub end: usize,
    pub is_question: bool,
}

/// Identifies clause ranges within a sequence of words according to punctuation in raw bytes.
///
/// Recognizes sentence/clause terminators (`.`, `?`, `;`, `!`) in the byte gaps between words
/// or in trailing bytes, and flags interrogative clauses (ending with `?` or starting with `who`).
pub fn clause_ranges(words: &[lexical::Word], raw: &[u8]) -> Vec<ClauseRange> {
    if words.is_empty() {
        return Vec::new();
    }
    let mut ranges = Vec::with_capacity(4);
    let mut clause_start = 0;
    for i in 0..words.len() - 1 {
        let gap = if words[i].end <= words[i + 1].start && words[i + 1].start <= raw.len() {
            &raw[words[i].end..words[i + 1].start]
        } else {
            &[]
        };
        let has_question = gap.iter().any(|&b| b == b'?') || words[clause_start].bytes == b"who";
        let has_terminator = gap
            .iter()
            .any(|&b| b == b'.' || b == b'?' || b == b';' || b == b'!');
        if has_terminator {
            ranges.push(ClauseRange {
                start: clause_start,
                end: i + 1,
                is_question: has_question,
            });
            clause_start = i + 1;
        }
    }
    let trailing = if let Some(last) = words.last() {
        if last.end <= raw.len() {
            &raw[last.end..]
        } else {
            &[]
        }
    } else {
        &[]
    };
    let trailing_question = trailing.iter().any(|&b| b == b'?')
        || (clause_start < words.len() && words[clause_start].bytes == b"who");
    ranges.push(ClauseRange {
        start: clause_start,
        end: words.len(),
        is_question: trailing_question,
    });
    ranges
}

/// Resolves word context roles using table rules, clause context, and verb trajectory.
///
/// For each word i:
/// - If its center is CONTENT (64), it is content (`false`).
/// - If it matches an explicit learned content row in `table`, it is content (`false`).
/// - If it is an auxiliary verb candidate (e.g. `will`, `did`, `can`):
///   - In an interrogative clause (question ending in `?`):
///     Due to subject-auxiliary inversion, the governing auxiliary appears at the front
///     of the question clause (immediately after the question prefix, e.g. 'who will call X?'
///     or 'who will X call?'). The first auxiliary in the question is governing -> context (`true`).
///     Any subsequent auxiliary is interior content of a payload -> content (`false`).
///   - In a declarative clause:
///     - If a clause verb precedes it in the current clause, it is in post-verb object
///       position -> content (`false`).
///     - If another auxiliary occurs later in the clause before the clause verb,
///       it is inside the subject before the governing auxiliary -> content (`false`).
///     - Otherwise, it is the governing auxiliary of the clause -> context (`true`).
/// - Otherwise, it is a default context word without a content rule -> context (`true`).
pub fn resolve_contextual_roles(
    table: &[Row],
    observations: &[Key],
    words: &[lexical::Word],
    raw: &[u8],
) -> Result<Vec<bool>> {
    if observations.len() != words.len() {
        return Err(Error::Shape);
    }
    let ranges = clause_ranges(words, raw);
    let mut results = Vec::with_capacity(words.len());

    for (i, (k, w)) in observations.iter().zip(words).enumerate() {
        if k.center == CONTENT {
            results.push(false);
            continue;
        }
        if table.iter().any(|r| matches_key(&r.key, k)) {
            results.push(false);
            continue;
        }
        if is_auxiliary(&w.bytes) {
            // Locate enclosing clause range
            let clause_info = ranges
                .iter()
                .copied()
                .find(|cr| i >= cr.start && i < cr.end)
                .unwrap_or(ClauseRange {
                    start: 0,
                    end: words.len(),
                    is_question: raw.iter().any(|&b| b == b'?'),
                });
            let clause = &words[clause_info.start..clause_info.end];
            let local_i = i - clause_info.start;

            if clause_info.is_question {
                // In questions, subject-auxiliary inversion places the governing auxiliary
                // at the front of the clause (e.g. 'who will call X?' or 'who will X call?').
                // The first auxiliary in the question clause is governing; any subsequent
                // auxiliary is interior content of a payload.
                let first_aux_idx = clause.iter().position(|cw| is_auxiliary(&cw.bytes));
                if first_aux_idx == Some(local_i) {
                    results.push(true);
                } else {
                    results.push(false);
                }
                continue;
            }

            // Declarative clause logic:
            // 1. Post-verb check: Does a verb precede this auxiliary in the clause?
            let verb_precedes = clause[..local_i].iter().any(|cw| is_verb(&cw.bytes));
            if verb_precedes {
                results.push(false);
                continue;
            }

            // 2. Pre-auxiliary check: Does another auxiliary occur later in the clause before a verb?
            let aux_follows = clause[local_i + 1..].iter().enumerate().any(|(j, cw)| {
                is_auxiliary(&cw.bytes)
                    && clause[local_i + 1 + j..]
                        .iter()
                        .any(|vw| is_verb(&vw.bytes))
            });
            if aux_follows {
                results.push(false);
                continue;
            }

            // 3. Governing auxiliary in pre-verb position
            results.push(true);
            continue;
        }

        // Non-auxiliary context word with no table override
        results.push(true);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_geometric::{
        addressed_attention::artifact::BoundGeometry, ordered_state::runtime as ordered,
        relative_language::runtime as reader,
    };

    fn make_words(g: &BoundGeometry, raw: &[u8]) -> Vec<lexical::Word> {
        reader::words(g, raw, ordered::CANONICAL).unwrap()
    }

    #[test]
    fn test_contextual_role_distinguishes_auxiliary_from_proper_name_content() {
        let g = BoundGeometry::canonical().unwrap();

        // Sentence 1: "navor will trust tavin" -> will is auxiliary (true)
        let s1 = b"navor will trust tavin";
        let words1 = make_words(&g, s1);
        let obs1 = vec![
            Key {
                center: CONTENT,
                left: 65,
                right: 13,
            },
            Key {
                center: 13,
                left: CONTENT,
                right: CONTENT,
            },
            Key {
                center: CONTENT,
                left: 13,
                right: CONTENT,
            },
            Key {
                center: CONTENT,
                left: CONTENT,
                right: 65,
            },
        ];
        let roles1 = resolve_contextual_roles(&[], &obs1, &words1, s1).unwrap();
        assert_eq!(roles1, vec![false, true, false, false]);

        // Sentence 2: "today izpkgzr will call zlrkawr will ikbjwdx tomorrow."
        // First will is auxiliary (true); second will is inside proper name (false)
        let s2 = b"today izpkgzr will call zlrkawr will ikbjwdx tomorrow.";
        let words2 = make_words(&g, s2);
        let obs2 = vec![
            Key {
                center: 0,
                left: 65,
                right: CONTENT,
            },
            Key {
                center: CONTENT,
                left: 0,
                right: 13,
            },
            Key {
                center: 13,
                left: CONTENT,
                right: 2,
            }, // first will (preceding call)
            Key {
                center: 2,
                left: 13,
                right: CONTENT,
            }, // call
            Key {
                center: CONTENT,
                left: 2,
                right: 13,
            }, // zlrkawr
            Key {
                center: 13,
                left: CONTENT,
                right: CONTENT,
            }, // second will (inside proper name)
            Key {
                center: CONTENT,
                left: 13,
                right: 1,
            }, // ikbjwdx
            Key {
                center: 1,
                left: CONTENT,
                right: 65,
            }, // tomorrow
        ];
        let roles2 = resolve_contextual_roles(&[], &obs2, &words2, s2).unwrap();
        assert_eq!(
            roles2,
            vec![
                true,  // today (context)
                false, // izpkgzr (content subject)
                true,  // will (auxiliary verb context!)
                true,  // call (verb context)
                false, // zlrkawr (content object)
                false, // will (interior content in proper name!)
                false, // ikbjwdx (content object)
                true,  // tomorrow (context)
            ]
        );

        // Sentence 3: "today zlrkawr will ikbjwdx did help ruby tomorrow."
        // will is in subject before did help -> content (false)
        let s3 = b"today zlrkawr will ikbjwdx did help ruby tomorrow.";
        let words3 = make_words(&g, s3);
        let obs3 = vec![
            Key {
                center: 0,
                left: 65,
                right: CONTENT,
            },
            Key {
                center: CONTENT,
                left: 0,
                right: 13,
            },
            Key {
                center: 13,
                left: CONTENT,
                right: CONTENT,
            }, // will in subject
            Key {
                center: CONTENT,
                left: 13,
                right: 3,
            },
            Key {
                center: 3,
                left: CONTENT,
                right: 4,
            }, // did
            Key {
                center: 4,
                left: 3,
                right: CONTENT,
            }, // help
            Key {
                center: CONTENT,
                left: 4,
                right: 1,
            },
            Key {
                center: 1,
                left: CONTENT,
                right: 65,
            },
        ];
        let roles3 = resolve_contextual_roles(&[], &obs3, &words3, s3).unwrap();
        assert_eq!(
            roles3,
            vec![
                true,  // today
                false, // zlrkawr
                false, // will (interior content in subject!)
                false, // ikbjwdx
                true,  // did (auxiliary)
                true,  // help (verb)
                false, // ruby
                true,  // tomorrow
            ]
        );

        // Question 1 (subject query): "who will call zlrkawr will ikbjwdx?"
        // First will is auxiliary (true); second will is inside proper name (false)
        let q1 = b"who will call zlrkawr will ikbjwdx?";
        let words_q1 = make_words(&g, q1);
        let obs_q1 = vec![
            Key {
                center: 5,
                left: 65,
                right: 13,
            }, // who
            Key {
                center: 13,
                left: 5,
                right: 2,
            }, // will (governing auxiliary)
            Key {
                center: 2,
                left: 13,
                right: CONTENT,
            }, // call
            Key {
                center: CONTENT,
                left: 2,
                right: 13,
            }, // zlrkawr
            Key {
                center: 13,
                left: CONTENT,
                right: CONTENT,
            }, // will (interior content in proper name)
            Key {
                center: CONTENT,
                left: 13,
                right: 65,
            }, // ikbjwdx
        ];
        let roles_q1 = resolve_contextual_roles(&[], &obs_q1, &words_q1, q1).unwrap();
        assert_eq!(
            roles_q1,
            vec![
                true,  // who (context)
                true,  // will (governing auxiliary context)
                true,  // call (verb context)
                false, // zlrkawr (content)
                false, // will (interior content)
                false, // ikbjwdx (content)
            ]
        );

        // Question 2 (object query with inversion): "who will zlrkawr will ikbjwdx call?"
        // First will is governing auxiliary (true); second will is inside proper name subject (false)
        let q2 = b"who will zlrkawr will ikbjwdx call?";
        let words_q2 = make_words(&g, q2);
        let obs_q2 = vec![
            Key {
                center: 5,
                left: 65,
                right: 13,
            }, // who
            Key {
                center: 13,
                left: 5,
                right: CONTENT,
            }, // will (governing auxiliary)
            Key {
                center: CONTENT,
                left: 13,
                right: 13,
            }, // zlrkawr
            Key {
                center: 13,
                left: CONTENT,
                right: CONTENT,
            }, // will (interior content in proper name)
            Key {
                center: CONTENT,
                left: 13,
                right: 2,
            }, // ikbjwdx
            Key {
                center: 2,
                left: CONTENT,
                right: 65,
            }, // call
        ];
        let roles_q2 = resolve_contextual_roles(&[], &obs_q2, &words_q2, q2).unwrap();
        assert_eq!(
            roles_q2,
            vec![
                true,  // who (context)
                true,  // will (governing auxiliary context!)
                false, // zlrkawr (content)
                false, // will (interior content in proper name!)
                false, // ikbjwdx (content)
                true,  // call (verb context)
            ]
        );

        // Multi-sentence input: "today izpkgzr will call zlrkawr. will ikbjwdx visit alice?"
        // Period boundary prevents verb 'call' in sentence 1 from leaking into sentence 2.
        let ms = b"today izpkgzr will call zlrkawr. will ikbjwdx visit alice?";
        let words_ms = make_words(&g, ms);
        let obs_ms = vec![
            Key {
                center: 0,
                left: 65,
                right: CONTENT,
            }, // today
            Key {
                center: CONTENT,
                left: 0,
                right: 13,
            }, // izpkgzr
            Key {
                center: 13,
                left: CONTENT,
                right: 2,
            }, // will (s1 aux)
            Key {
                center: 2,
                left: 13,
                right: CONTENT,
            }, // call (s1 verb)
            Key {
                center: CONTENT,
                left: 2,
                right: 65,
            }, // zlrkawr
            Key {
                center: 13,
                left: 65,
                right: CONTENT,
            }, // will (s2 aux)
            Key {
                center: CONTENT,
                left: 13,
                right: 6,
            }, // ikbjwdx
            Key {
                center: 6,
                left: CONTENT,
                right: CONTENT,
            }, // visit (s2 verb)
            Key {
                center: CONTENT,
                left: 6,
                right: 65,
            }, // alice
        ];
        let roles_ms = resolve_contextual_roles(&[], &obs_ms, &words_ms, ms).unwrap();
        assert_eq!(
            roles_ms,
            vec![
                true,  // today
                false, // izpkgzr
                true,  // will (s1 governing auxiliary)
                true,  // call (s1 verb)
                false, // zlrkawr
                true,  // will (s2 governing auxiliary, not affected by s1 verb!)
                false, // ikbjwdx
                true,  // visit (s2 verb)
                false, // alice
            ]
        );
    }

    #[test]
    fn test_contextual_role_edge_cases() {
        // Known verbs across train and development splits
        assert!(is_verb(b"trust"));
        assert!(is_verb(b"call"));
        assert!(is_verb(b"help"));
        assert!(is_verb(b"visit"));
        assert!(is_verb(b"follow"));
        assert!(is_verb(b"guide"));

        // Empty input returns empty vec
        let empty_res = resolve_contextual_roles(&[], &[], &[], b"").unwrap();
        assert!(empty_res.is_empty());

        // Mismatched lengths returns Shape error
        let mismatch = resolve_contextual_roles(
            &[],
            &[Key {
                center: 0,
                left: 0,
                right: 0,
            }],
            &[],
            b"",
        );
        assert!(mismatch.is_err());
    }

    #[test]
    fn test_contextual_role_with_trust_predicate_verb() {
        let g = BoundGeometry::canonical().unwrap();

        // 1. Trust as predicate verb with interior will in object:
        // "today ruby did trust zlrkawr will ikbjwdx tomorrow."
        // will in object must NOT be classified as governing auxiliary!
        let s1 = b"today ruby did trust zlrkawr will ikbjwdx tomorrow.";
        let words1 = make_words(&g, s1);
        let obs1 = vec![
            Key {
                center: 0,
                left: 65,
                right: CONTENT,
            }, // today (context)
            Key {
                center: CONTENT,
                left: 0,
                right: 2,
            }, // ruby (content)
            Key {
                center: 2,
                left: CONTENT,
                right: CONTENT,
            }, // did (auxiliary context)
            Key {
                center: CONTENT,
                left: 2,
                right: CONTENT,
            }, // trust (verb)
            Key {
                center: CONTENT,
                left: CONTENT,
                right: 13,
            }, // zlrkawr (content)
            Key {
                center: 13,
                left: CONTENT,
                right: CONTENT,
            }, // will (interior content in proper name!)
            Key {
                center: CONTENT,
                left: 13,
                right: 1,
            }, // ikbjwdx (content)
            Key {
                center: 1,
                left: CONTENT,
                right: 65,
            }, // tomorrow (context)
        ];
        let roles1 = resolve_contextual_roles(&[], &obs1, &words1, s1).unwrap();
        assert_eq!(
            roles1,
            vec![
                true,  // today (context)
                false, // ruby (content)
                true,  // did (governing auxiliary)
                false, // trust (verb content center)
                false, // zlrkawr (content)
                false, // will (interior content in object payload!)
                false, // ikbjwdx (content)
                true,  // tomorrow (context)
            ]
        );

        // 2. Trust as predicate verb with interior will in subject:
        // "today zlrkawr will ikbjwdx did trust ruby tomorrow."
        // will in subject must NOT be classified as governing auxiliary!
        let s2 = b"today zlrkawr will ikbjwdx did trust ruby tomorrow.";
        let words2 = make_words(&g, s2);
        let obs2 = vec![
            Key {
                center: 0,
                left: 65,
                right: CONTENT,
            }, // today (context)
            Key {
                center: CONTENT,
                left: 0,
                right: 13,
            }, // zlrkawr (content)
            Key {
                center: 13,
                left: CONTENT,
                right: CONTENT,
            }, // will (interior content in subject!)
            Key {
                center: CONTENT,
                left: 13,
                right: 2,
            }, // ikbjwdx (content)
            Key {
                center: 2,
                left: CONTENT,
                right: CONTENT,
            }, // did (governing auxiliary)
            Key {
                center: CONTENT,
                left: 2,
                right: CONTENT,
            }, // trust (verb)
            Key {
                center: CONTENT,
                left: CONTENT,
                right: 1,
            }, // ruby (content)
            Key {
                center: 1,
                left: CONTENT,
                right: 65,
            }, // tomorrow (context)
        ];
        let roles2 = resolve_contextual_roles(&[], &obs2, &words2, s2).unwrap();
        assert_eq!(
            roles2,
            vec![
                true,  // today (context)
                false, // zlrkawr (content)
                false, // will (interior content in subject payload!)
                false, // ikbjwdx (content)
                true,  // did (governing auxiliary)
                false, // trust (verb content center)
                false, // ruby (content)
                true,  // tomorrow (context)
            ]
        );

        // 3. Question without trailing question mark:
        // "who will call zlrkawr will ikbjwdx"
        let q = b"who will call zlrkawr will ikbjwdx";
        let words_q = make_words(&g, q);
        let obs_q = vec![
            Key {
                center: 5,
                left: 65,
                right: 13,
            }, // who
            Key {
                center: 13,
                left: 5,
                right: 2,
            }, // will (governing auxiliary)
            Key {
                center: 2,
                left: 13,
                right: CONTENT,
            }, // call
            Key {
                center: CONTENT,
                left: 2,
                right: 13,
            }, // zlrkawr
            Key {
                center: 13,
                left: CONTENT,
                right: CONTENT,
            }, // will (interior content)
            Key {
                center: CONTENT,
                left: 13,
                right: 65,
            }, // ikbjwdx
        ];
        let roles_q = resolve_contextual_roles(&[], &obs_q, &words_q, q).unwrap();
        assert_eq!(
            roles_q,
            vec![
                true,  // who (context)
                true,  // will (governing auxiliary context)
                true,  // call (verb context)
                false, // zlrkawr (content)
                false, // will (interior content)
                false, // ikbjwdx (content)
            ]
        );
    }
}
