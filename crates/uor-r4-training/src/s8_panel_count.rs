//! Count-only length check of the sealed §8 panel (D18 section 9).
//!
//! The §8 panel is sealed: its text must never reach a lab. The owner runs
//! this check on `panel.json` and posts its output. The output holds token
//! counts and identities only. Errors name a category, a conversation index
//! and a turn number, never text. A JSON error reports only its line and
//! column, because serde's messages can quote the offending value.
//!
//! The panel's documented format (`uor-r4.s8-panel/1`) is: a `categories`
//! object holding exactly `responsive`, `updated_relation` and
//! `new_instruction`, each a list of conversations; each conversation has
//! `turns`; each turn has `user` and `scored`, and an unscored turn carries
//! `reference_reply`, the assistant reply used as history. Other fields
//! (ids, checks, budget) are not read.
//!
//! Histories follow the literal-role dialogue protocol exactly as
//! [`crate::stack_dialogue::reply_panel`] builds them: BOS, then for each turn
//! the user prefix (separator, `User: `, content, `Assistant: ` marker) and
//! the reply. A reference reply is followed by its EOS. A generated reply
//! that does not end in EOS is closed with EOS by the caller before a later
//! turn.

use serde_json::{json, Map, Value};
use uor_r4_tokenizer::dialogue::{DialogueEncoder, Message};

use crate::{invalid, Result};

pub const PANEL_SCHEMA: &str = "uor-r4.s8-panel/1";
pub const REPORT_SCHEMA: &str = "uor-r4.s8-panel-count/1";
pub const CATEGORIES: [&str; 3] = ["responsive", "updated_relation", "new_instruction"];

/// The reply budget named in D18 section 9, and the §8 context.
pub const REPLY_BUDGET: usize = 32;
pub const CONTEXT: usize = 256;

/// The two maxima D18 section 9 asks for, over every conversation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PanelCounts {
    pub conversations: usize,
    /// The whole conversation with every scored reply empty: BOS, each user
    /// prefix, each reference reply with its EOS, and the EOS that closes
    /// each scored reply before a later turn.
    pub max_conversation_tokens: usize,
    /// The most positions any scored reply can reach when every scored reply
    /// runs to `reply_budget` ids: the accounting of
    /// [`crate::stack_dialogue::check_panel`].
    pub max_with_reply_budget: usize,
    pub reply_budget: usize,
}

impl PanelCounts {
    /// The complete output: counts and identities, with no panel text.
    pub fn report(&self, panel_sha256: &str, tokenizer_cid: &str, protocol: &str) -> Value {
        json!({
            "schema": REPORT_SCHEMA,
            "panel_sha256": panel_sha256,
            "tokenizer_cid": tokenizer_cid,
            "dialogue_protocol": protocol,
            "conversations": self.conversations,
            "reply_budget": self.reply_budget,
            "max_conversation_tokens": self.max_conversation_tokens,
            "max_with_reply_budget": self.max_with_reply_budget,
            "context": CONTEXT,
            "fits_context": self.max_with_reply_budget <= CONTEXT,
        })
    }
}

/// Count every conversation of a `uor-r4.s8-panel/1` document.
pub fn count_panel(
    encoder: &DialogueEncoder<'_>,
    panel: &[u8],
    reply_budget: usize,
) -> Result<PanelCounts> {
    let panel: Value = serde_json::from_slice(panel).map_err(|e| {
        invalid(format!(
            "panel.json is not valid JSON (line {}, column {})",
            e.line(),
            e.column()
        ))
    })?;
    if panel.get("schema").and_then(Value::as_str) != Some(PANEL_SCHEMA) {
        return Err(invalid(format!("panel.json schema is not {PANEL_SCHEMA}")));
    }
    let categories = panel
        .get("categories")
        .and_then(Value::as_object)
        .filter(|c| c.len() == CATEGORIES.len() && CATEGORIES.iter().all(|k| c.contains_key(*k)))
        .ok_or_else(|| {
            invalid(format!(
                "panel.json categories must be exactly {}",
                CATEGORIES.join(", ")
            ))
        })?;
    let mut counts = PanelCounts {
        conversations: 0,
        max_conversation_tokens: 0,
        max_with_reply_budget: 0,
        reply_budget,
    };
    for category in CATEGORIES {
        let conversations = categories[category]
            .as_array()
            .filter(|list| !list.is_empty())
            .ok_or_else(|| invalid(format!("{category}: not a nonempty list of conversations")))?;
        for (index, conversation) in conversations.iter().enumerate() {
            let at = format!("{category} conversation {}", index + 1);
            let turns = read_turns(conversation, &at)?;
            let (whole, _) = walk(encoder, &turns, 0, &at)?;
            let (_, needed) = walk(encoder, &turns, reply_budget, &at)?;
            counts.conversations += 1;
            counts.max_conversation_tokens = counts.max_conversation_tokens.max(whole);
            counts.max_with_reply_budget = counts.max_with_reply_budget.max(needed);
        }
    }
    Ok(counts)
}

struct Turn<'a> {
    user: &'a str,
    /// `None` for a scored turn; the reference reply otherwise.
    reference: Option<&'a str>,
}

fn read_turns<'a>(conversation: &'a Value, at: &str) -> Result<Vec<Turn<'a>>> {
    let turns = conversation
        .get("turns")
        .and_then(Value::as_array)
        .filter(|t| !t.is_empty())
        .ok_or_else(|| invalid(format!("{at}: turns is not a nonempty list")))?;
    let mut out = Vec::with_capacity(turns.len());
    for (index, turn) in turns.iter().enumerate() {
        let at = format!("{at} turn {}", index + 1);
        let turn: &Map<String, Value> = turn
            .as_object()
            .ok_or_else(|| invalid(format!("{at}: not an object")))?;
        let user = turn
            .get("user")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid(format!("{at}: user is not a string")))?;
        let scored = turn
            .get("scored")
            .and_then(Value::as_bool)
            .ok_or_else(|| invalid(format!("{at}: scored is not a boolean")))?;
        let reference = if scored {
            None
        } else {
            Some(
                turn.get("reference_reply")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        invalid(format!(
                            "{at}: an unscored turn needs a reference_reply string"
                        ))
                    })?,
            )
        };
        out.push(Turn { user, reference });
    }
    if out.iter().all(|t| t.reference.is_some()) {
        return Err(invalid(format!("{at}: no scored turn")));
    }
    Ok(out)
}

/// Build the history length turn by turn with every scored reply at `budget`
/// ids. Returns the length after the last turn and the largest length any
/// scored reply reaches.
fn walk(
    encoder: &DialogueEncoder<'_>,
    turns: &[Turn<'_>],
    budget: usize,
    at: &str,
) -> Result<(usize, usize)> {
    // The marker `Assistant: ` after BOS alone; subtracted from a one-message
    // history to leave a reference reply and its EOS, encoded as the protocol
    // encodes assistant content.
    let marker = encoder.encode_assistant_prefix(&[]).tokens.len();
    let mut length = 1usize;
    let mut needed = 0usize;
    for (index, turn) in turns.iter().enumerate() {
        let prefix = encoder.encode_user_prefix(turn.user, index != 0);
        if prefix.emitted_turns != 1 || prefix.special_token_occurrences != 0 {
            return Err(invalid(format!(
                "{at} turn {}: the user text is not one plain protocol turn",
                index + 1
            )));
        }
        length += prefix.tokens.len();
        let later = index + 1 < turns.len();
        match turn.reference {
            Some(reference) => {
                let reply = encoder.encode_open_history(&[Message {
                    role: "assistant",
                    content: reference,
                }]);
                if reply.emitted_turns != 1 || reply.special_token_occurrences != 0 {
                    return Err(invalid(format!(
                        "{at} turn {}: the reference reply is not one plain protocol turn",
                        index + 1
                    )));
                }
                length += reply.tokens.len() - marker;
            }
            None => {
                length += budget;
                needed = needed.max(length);
                if later {
                    length += 1;
                }
            }
        }
    }
    Ok((length, needed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack_dialogue::{check_panel, Request};
    use uor_r4_tokenizer::dialogue::DialogueProtocol;
    use uor_r4_tokenizer::ByteBpeTokenizer;

    fn alphabet() -> Vec<char> {
        let mut printable: Vec<u32> = (u32::from(b'!')..=u32::from(b'~')).collect();
        printable.extend(0xA1..=0xAC);
        printable.extend(0xAE..=0xFF);
        let mut extra = 0;
        (0u32..256)
            .map(|byte| {
                if printable.contains(&byte) {
                    char::from_u32(byte).unwrap()
                } else {
                    extra += 1;
                    char::from_u32(255 + extra).unwrap()
                }
            })
            .collect()
    }

    /// A byte-level tokenizer with the three dialogue specials at ids 0-2.
    fn tokenizer() -> ByteBpeTokenizer {
        let mut vocab = Map::new();
        for (id, surface) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
            vocab.insert((*surface).to_owned(), json!(id));
        }
        for (byte, ch) in alphabet().iter().enumerate() {
            vocab.insert(ch.to_string(), json!(byte + 3));
        }
        let added: Vec<Value> = ["<|bos|>", "<|eos|>", "<|unk|>"]
            .iter()
            .enumerate()
            .map(|(id, surface)| json!({"id": id, "content": surface}))
            .collect();
        ByteBpeTokenizer::from_tokenizer_json_bytes(
            json!({
                "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
                "added_tokens": added,
                "model": {"type": "BPE", "vocab": vocab, "merges": []},
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap()
    }

    fn scored(user: &str) -> Value {
        json!({"user": user, "scored": true, "checks": []})
    }

    fn unscored(user: &str, reply: &str) -> Value {
        json!({"user": user, "scored": false, "reference_reply": reply})
    }

    /// A fixture in the panel's format. None of it is panel text.
    fn panel(responsive: Value, relation: Value, instruction: Value) -> Vec<u8> {
        json!({
            "schema": PANEL_SCHEMA,
            "budget": {"max_words": 140},
            "categories": {
                "responsive": [{"id": "X1", "turns": responsive}],
                "updated_relation": [{"id": "X2", "turns": relation}],
                "new_instruction": [{"id": "X3", "turns": instruction}],
            },
        })
        .to_string()
        .into_bytes()
    }

    fn user(content: &str) -> Message<'_> {
        Message {
            role: "user",
            content,
        }
    }

    fn assistant(content: &str) -> Message<'_> {
        Message {
            role: "assistant",
            content,
        }
    }

    #[test]
    fn an_all_scored_conversation_counts_as_check_panel_accounts() {
        let tokenizer = tokenizer();
        let protocol = DialogueProtocol::literal_roles_v1(&tokenizer).unwrap();
        let encoder = protocol.bind(&tokenizer).unwrap();
        let turns = [
            "how was the market?",
            "and the bakery?",
            "what did you buy?",
        ];
        let bytes = panel(
            json!(turns.iter().map(|t| scored(t)).collect::<Vec<_>>()),
            json!([scored("x")]),
            json!([scored("y")]),
        );
        let counts = count_panel(&encoder, &bytes, REPLY_BUDGET).unwrap();
        let request = [Request {
            id: "X1".into(),
            category: "responsive".into(),
            user_turns: turns.iter().map(|t| (*t).to_owned()).collect(),
        }];
        // check_panel admits exactly this count as the context, and no less.
        let needed = counts.max_with_reply_budget;
        assert!(check_panel(&encoder, &request, needed, REPLY_BUDGET).is_ok());
        assert!(check_panel(&encoder, &request, needed - 1, REPLY_BUDGET).is_err());
        // With empty replies: BOS, the three prefixes and the two closing EOS.
        let prefixes: usize = turns
            .iter()
            .enumerate()
            .map(|(i, t)| encoder.encode_user_prefix(t, i != 0).tokens.len())
            .sum();
        assert_eq!(counts.max_conversation_tokens, 1 + prefixes + 2);
        assert_eq!(counts.conversations, 3);
    }

    #[test]
    fn reference_replies_count_as_the_protocol_serializes_them() {
        let tokenizer = tokenizer();
        let protocol = DialogueProtocol::literal_roles_v1(&tokenizer).unwrap();
        let encoder = protocol.bind(&tokenizer).unwrap();
        let relation = json!([
            unscored("my cat is called Pip.", "  Nice name!\r\n "),
            unscored("actually it is Tam now.", "Got it."),
            scored("what is my cat called?"),
        ]);
        let bytes = panel(json!([scored("a")]), relation, json!([scored("b")]));
        let counts = count_panel(&encoder, &bytes, REPLY_BUDGET).unwrap();
        // The canonical serializer's prompt for the scored reply, then the
        // budget: the relation conversation is the longest here.
        let prompt = encoder.encode_assistant_prefix(&[
            user("my cat is called Pip."),
            assistant("  Nice name!\r\n "),
            user("actually it is Tam now."),
            assistant("Got it."),
            user("what is my cat called?"),
        ]);
        assert_eq!(
            counts.max_with_reply_budget,
            prompt.tokens.len() + REPLY_BUDGET
        );
        assert_eq!(counts.max_conversation_tokens, prompt.tokens.len());
    }

    #[test]
    fn a_conversation_ending_in_a_reference_reply_counts_the_whole_document() {
        let tokenizer = tokenizer();
        let protocol = DialogueProtocol::literal_roles_v1(&tokenizer).unwrap();
        let encoder = protocol.bind(&tokenizer).unwrap();
        let turns = json!([scored("q1"), unscored("q2", "a long reference reply here")]);
        let bytes = panel(turns, json!([scored("x")]), json!([scored("y")]));
        let counts = count_panel(&encoder, &bytes, 0).unwrap();
        // An empty scored reply is closed by EOS, as the document would
        // encode an empty assistant turn if it were emitted.
        let document = encoder.encode_open_history(&[
            user("q1"),
            assistant("-"),
            user("q2"),
            assistant("a long reference reply here"),
        ]);
        let dash = tokenizer.encode("-").len();
        assert_eq!(counts.max_conversation_tokens, document.tokens.len() - dash);
    }

    #[test]
    fn errors_never_quote_panel_text() {
        let tokenizer = tokenizer();
        let protocol = DialogueProtocol::literal_roles_v1(&tokenizer).unwrap();
        let encoder = protocol.bind(&tokenizer).unwrap();
        let secret = "ZebraQuartzSentinel";
        let mut cases: Vec<Vec<u8>> = vec![
            // Invalid JSON right after the secret, and a wrong JSON type.
            format!(r#"{{"schema": "{secret}" "#).into_bytes(),
            format!(r#"{{"schema": ["{secret}"]}}"#).into_bytes(),
            // Wrong schema and an unknown category.
            json!({"schema": secret, "categories": {}})
                .to_string()
                .into_bytes(),
            json!({"schema": PANEL_SCHEMA, "categories": {secret: []}})
                .to_string()
                .into_bytes(),
        ];
        let bad_turns = [
            json!([{"user": secret, "scored": secret}]),
            json!([{"user": 7, "scored": true, "note": secret}]),
            json!([{"user": secret, "scored": false}]),
            json!([unscored(secret, secret)]),
            json!([scored(&format!("{secret} <|eos|>"))]),
            json!([unscored("q", &format!("{secret} <|bos|>")), scored("q")]),
            json!(secret),
        ];
        for turns in bad_turns {
            cases.push(panel(turns, json!([scored("x")]), json!([scored("y")])));
        }
        for bytes in cases {
            let error = count_panel(&encoder, &bytes, REPLY_BUDGET)
                .expect_err("a malformed panel is refused")
                .to_string();
            assert!(!error.contains(secret), "{error}");
            assert!(!error.contains("Zebra"), "{error}");
        }
    }

    #[test]
    fn the_report_holds_counts_and_identities_only() {
        let counts = PanelCounts {
            conversations: 100,
            max_conversation_tokens: 200,
            max_with_reply_budget: 300,
            reply_budget: REPLY_BUDGET,
        };
        let report = counts.report("aa", "blake3:bb", "blake3:cc");
        let mut keys: Vec<&str> = report
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "context",
                "conversations",
                "dialogue_protocol",
                "fits_context",
                "max_conversation_tokens",
                "max_with_reply_budget",
                "panel_sha256",
                "reply_budget",
                "schema",
                "tokenizer_cid",
            ]
        );
        assert_eq!(report["fits_context"], false);
    }
}
