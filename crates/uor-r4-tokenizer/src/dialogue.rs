//! Tokenizer-bound literal-role dialogue serialization shared by future corpus
//! and serving adapters. No existing model, bundle or session adopts this module
//! implicitly. Version 1 preserves the September 26 `prepare-chat-corpus` format,
//! including separate encoding of separator, marker and content segments.

use crate::ByteBpeTokenizer;
use serde::{Deserialize, Serialize};
use std::fmt;

/// V1 uses literal `System: ` / `User: ` / `Assistant: ` markers, a newline
/// before each subsequent turn, and EOS after each assistant response. It
/// normalizes CRLF/CR to LF and trims content ends, retaining interior whitespace.
/// Documents ending in a non-assistant turn receive an unmasked terminal EOS.
/// Assistant content and its EOS alone receive response-mask value 1.
pub const SCHEMA: &str = "uor-r4.literal-role-dialogue/1";
const BOS: &str = "<|bos|>";
const EOS: &str = "<|eos|>";
const UNK: &str = "<|unk|>";

/// Portable contract to bind into a *new* artifact manifest. The schema fixes
/// formatting and mask semantics; tokenizer identity fixes all tokenization.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialogueProtocol {
    pub schema: String,
    pub tokenizer_cid: String,
    pub bos_id: u32,
    pub eos_id: u32,
    pub unk_id: u32,
}

#[derive(Debug)]
pub enum DialogueError {
    Json(serde_json::Error),
    UnsupportedSchema(String),
    TokenizerMismatch {
        expected: String,
        actual: String,
    },
    InvalidSpecial {
        surface: &'static str,
    },
    SpecialIdMismatch {
        surface: &'static str,
        expected: u32,
        actual: u32,
    },
}
impl fmt::Display for DialogueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => write!(f, "dialogue protocol JSON: {e}"),
            Self::UnsupportedSchema(s) => write!(f, "unsupported dialogue protocol: {s}"),
            Self::TokenizerMismatch { expected, actual } => {
                write!(
                    f,
                    "dialogue tokenizer differs: expected {expected}, found {actual}"
                )
            }
            Self::InvalidSpecial { surface } => {
                write!(f, "dialogue requires an atomic added token for {surface}")
            }
            Self::SpecialIdMismatch {
                surface,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "dialogue {surface} ID differs: expected {expected}, found {actual}"
                )
            }
        }
    }
}
impl std::error::Error for DialogueError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}
pub type Result<T> = std::result::Result<T, DialogueError>;

fn special(tokenizer: &ByteBpeTokenizer, surface: &'static str) -> Result<u32> {
    let ids = tokenizer.encode(surface);
    match ids.as_slice() {
        [id] if tokenizer.added_ids.contains(id)
            && tokenizer.decode_bytes(&[*id]) == surface.as_bytes() =>
        {
            Ok(*id)
        }
        _ => Err(DialogueError::InvalidSpecial { surface }),
    }
}

impl DialogueProtocol {
    pub fn literal_roles_v1(tokenizer: &ByteBpeTokenizer) -> Result<Self> {
        Ok(Self {
            schema: SCHEMA.to_owned(),
            tokenizer_cid: tokenizer.address(),
            bos_id: special(tokenizer, BOS)?,
            eos_id: special(tokenizer, EOS)?,
            unk_id: special(tokenizer, UNK)?,
        })
    }

    /// Parse and validate before using a serialized contract. Direct serde
    /// deserialization is also supported, but `bind` still validates every field.
    pub fn from_json(bytes: &[u8], tokenizer: &ByteBpeTokenizer) -> Result<Self> {
        let protocol: Self = serde_json::from_slice(bytes).map_err(DialogueError::Json)?;
        protocol.bind(tokenizer)?;
        Ok(protocol)
    }

    /// Stable digest of the schema, tokenizer CID and resolved special IDs.
    pub fn identity(&self) -> Result<String> {
        let bytes = serde_json::to_vec(self).map_err(DialogueError::Json)?;
        Ok(format!("blake3:{}", blake3::hash(&bytes).to_hex()))
    }

    pub fn bind<'a>(&self, tokenizer: &'a ByteBpeTokenizer) -> Result<DialogueEncoder<'a>> {
        if self.schema != SCHEMA {
            return Err(DialogueError::UnsupportedSchema(self.schema.clone()));
        }
        let actual = tokenizer.address();
        if self.tokenizer_cid != actual {
            return Err(DialogueError::TokenizerMismatch {
                expected: self.tokenizer_cid.clone(),
                actual,
            });
        }
        for (surface, expected) in [(BOS, self.bos_id), (EOS, self.eos_id), (UNK, self.unk_id)] {
            let actual = special(tokenizer, surface)?;
            if expected != actual {
                return Err(DialogueError::SpecialIdMismatch {
                    surface,
                    expected,
                    actual,
                });
            }
        }
        Ok(DialogueEncoder {
            tokenizer,
            protocol: self.clone(),
        })
    }
}

/// Raw message view. Corpus callers map absent role/content fields to `""`.
/// Unknown roles and blank content are skipped and counted, as in the original
/// corpus serializer. This layer imposes no turn-alternation policy.
#[derive(Clone, Copy, Debug)]
pub struct Message<'a> {
    pub role: &'a str,
    pub content: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedDialogue {
    pub tokens: Vec<u32>,
    pub response_mask: Vec<u8>,
    pub emitted_turns: usize,
    pub skipped_empty: usize,
    pub skipped_unknown_role: usize,
    /// BOS/EOS/UNK occurrences encoded from literal text segments. Delimiters
    /// inserted by the protocol are excluded, matching the corpus receipt.
    pub special_token_occurrences: usize,
}

pub struct DialogueEncoder<'a> {
    tokenizer: &'a ByteBpeTokenizer,
    protocol: DialogueProtocol,
}

#[derive(Clone, Copy)]
enum Ending {
    Document,
    AssistantPrefix,
    OpenHistory,
}

impl DialogueEncoder<'_> {
    /// Complete corpus document. An all-skipped row contains BOS only and has
    /// `emitted_turns == 0`; the corpus owner retains its row-admission policy.
    pub fn encode_document(&self, messages: &[Message<'_>]) -> EncodedDialogue {
        self.encode(messages, Ending::Document, true, false)
    }

    /// Prompt for the next assistant response. Preserve all completed assistant
    /// EOS tokens, omit the non-assistant document terminator, then encode the
    /// separator and literal `Assistant: ` marker independently. Do not pass a
    /// previously rendered prompt as message content. The response mask covers
    /// known history only; every added prompt-marker token has mask 0.
    pub fn encode_assistant_prefix(&self, messages: &[Message<'_>]) -> EncodedDialogue {
        self.encode(messages, Ending::AssistantPrefix, true, false)
    }

    /// Initial completed history for a persistent conversation. Include BOS
    /// and completed assistant EOS, but no corpus-only terminal EOS after a
    /// non-assistant message and no next assistant marker. Empty/all-skipped
    /// history contains BOS alone; emitted_turns determines the next separator.
    pub fn encode_open_history(&self, messages: &[Message<'_>]) -> EncodedDialogue {
        self.encode(messages, Ending::OpenHistory, true, false)
    }

    /// Append messages and the next assistant marker to already closed history,
    /// without re-encoding its token IDs. `has_history` adds a separator before
    /// the first emitted message. No BOS or document-terminal EOS is inserted;
    /// assistant messages in this suffix still receive their own closing EOS.
    /// Separators, markers and normalized content retain their segment boundaries
    /// and response masks. Empty/all-skipped input emits only the assistant
    /// marker; callers must check `emitted_turns` when a new turn is required.
    pub fn encode_assistant_suffix(
        &self,
        messages: &[Message<'_>],
        has_history: bool,
    ) -> EncodedDialogue {
        self.encode(messages, Ending::AssistantPrefix, false, has_history)
    }

    /// Append a user message and next assistant marker after already closed
    /// history. No BOS, assistant closure or document-terminal EOS is inserted.
    /// Each separator, marker and normalized content is encoded independently.
    /// The caller must reject emitted_turns == 0 for a required user request.
    pub fn encode_user_prefix(&self, content: &str, has_history: bool) -> EncodedDialogue {
        self.encode_assistant_suffix(
            &[Message {
                role: "user",
                content,
            }],
            has_history,
        )
    }

    fn append(&self, out: &mut EncodedDialogue, text: &str, mask: u8) {
        for id in self.tokenizer.encode(text) {
            if [
                self.protocol.bos_id,
                self.protocol.eos_id,
                self.protocol.unk_id,
            ]
            .contains(&id)
            {
                out.special_token_occurrences += 1;
            }
            out.tokens.push(id);
            out.response_mask.push(mask);
        }
    }

    fn encode(
        &self,
        messages: &[Message<'_>],
        ending: Ending,
        include_bos: bool,
        leading_separator: bool,
    ) -> EncodedDialogue {
        let mut out = EncodedDialogue {
            tokens: if include_bos {
                vec![self.protocol.bos_id]
            } else {
                Vec::new()
            },
            response_mask: if include_bos { vec![0] } else { Vec::new() },
            emitted_turns: 0,
            skipped_empty: 0,
            skipped_unknown_role: 0,
            special_token_occurrences: 0,
        };
        let mut last_assistant = false;
        for message in messages {
            let content = message.content.replace("\r\n", "\n").replace('\r', "\n");
            let content = content.trim();
            if content.is_empty() {
                out.skipped_empty += 1;
                continue;
            }
            let (marker, assistant) = match message.role {
                "system" => ("System: ", false),
                "user" => ("User: ", false),
                "assistant" => ("Assistant: ", true),
                _ => {
                    out.skipped_unknown_role += 1;
                    continue;
                }
            };
            if out.emitted_turns != 0 || leading_separator {
                self.append(&mut out, "\n", 0);
            }
            self.append(&mut out, marker, 0);
            self.append(&mut out, content, u8::from(assistant));
            if assistant {
                out.tokens.push(self.protocol.eos_id);
                out.response_mask.push(1);
            }
            out.emitted_turns += 1;
            last_assistant = assistant;
        }
        if matches!(ending, Ending::AssistantPrefix) {
            if out.emitted_turns != 0 {
                self.append(&mut out, "\n", 0);
            }
            self.append(&mut out, "Assistant: ", 0);
        } else if matches!(ending, Ending::Document) && out.emitted_turns != 0 && !last_assistant {
            out.tokens.push(self.protocol.eos_id);
            out.response_mask.push(0);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture(merge_space_h: bool, eos_added: bool) -> ByteBpeTokenizer {
        let mut vocab = serde_json::Map::new();
        for (id, surface) in [BOS, EOS, UNK].iter().enumerate() {
            vocab.insert((*surface).to_owned(), json!(id));
        }
        let alphabet = crate::bytes_to_unicode();
        for (byte, ch) in alphabet.iter().enumerate() {
            vocab.insert(ch.to_string(), json!(byte + 3));
        }
        let mut merges = Vec::new();
        if merge_space_h {
            vocab.insert(format!("{}H", alphabet[32]), json!(259));
            merges.push(format!("{} H", alphabet[32]));
        }
        let mut added = vec![json!({"id":0,"content":BOS}), json!({"id":2,"content":UNK})];
        if eos_added {
            added.push(json!({"id":1,"content":EOS}));
        }
        ByteBpeTokenizer::from_tokenizer_json_bytes(
            json!({
                "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
                "added_tokens":added,
                "model":{"type":"BPE","vocab":vocab,"merges":merges}
            })
            .to_string()
            .as_bytes(),
        )
        .expect("fixture tokenizer")
    }

    fn bytes(text: &str) -> Vec<u32> {
        text.bytes().map(|b| u32::from(b) + 3).collect()
    }

    #[test]
    fn incremental_prefix_retains_segment_boundaries_and_history_eos() -> Result<()> {
        let tokenizer = fixture(true, true);
        let encoder = DialogueProtocol::literal_roles_v1(&tokenizer)?.bind(&tokenizer)?;
        let history = [
            Message {
                role: "system",
                content: " S\r! ",
            },
            Message {
                role: "assistant",
                content: "A<|bos|>",
            },
        ];
        let open = encoder.encode_open_history(&history);
        let mut expected = vec![0];
        expected.extend(bytes("System: S\n!\nAssistant: A"));
        expected.extend([0, 1]);
        assert_eq!(open.tokens, expected);
        let user = encoder.encode_user_prefix(" \r\nHi\r世界 \t", true);
        assert_eq!(user.tokens, bytes("\nUser: Hi\n世界\nAssistant: "));
        assert!(user.response_mask.iter().all(|&mask| mask == 0));
        let mut combined = open.tokens;
        combined.extend(user.tokens);
        assert_eq!(
            combined,
            encoder
                .encode_assistant_prefix(&[
                    history[0],
                    history[1],
                    Message {
                        role: "user",
                        content: " \r\nHi\r世界 \t"
                    },
                ])
                .tokens
        );
        // No newline before the first emitted turn; no document EOS after a
        // system/user-ending open history. Consecutive users remain supported.
        let skipped = encoder.encode_open_history(&[Message {
            role: "tool",
            content: "skip",
        }]);
        assert_eq!(skipped.tokens, [0]);
        assert_eq!(skipped.emitted_turns, 0);
        assert_eq!(
            encoder.encode_user_prefix("Hi", false).tokens,
            bytes("User: Hi\nAssistant: ")
        );
        for role in ["system", "user"] {
            let open = encoder.encode_open_history(&[Message {
                role,
                content: "old",
            }]);
            assert!(!open.tokens.contains(&1));
        }
        assert_eq!(encoder.encode_user_prefix("\r\n\t", true).emitted_turns, 0);
        Ok(())
    }

    #[test]
    fn assistant_suffix_keeps_user_and_memory_segments_without_extra_delimiters() -> Result<()> {
        let tokenizer = fixture(true, true);
        let encoder = DialogueProtocol::literal_roles_v1(&tokenizer)?.bind(&tokenizer)?;
        let messages = [
            Message {
                role: "user",
                content: " \r\nHi\r世界 \t",
            },
            Message {
                role: "system",
                content: " Hello ",
            },
        ];
        for has_history in [false, true] {
            let suffix = encoder.encode_assistant_suffix(&messages, has_history);
            let text = if has_history {
                "\nUser: Hi\n世界\nSystem: Hello\nAssistant: "
            } else {
                "User: Hi\n世界\nSystem: Hello\nAssistant: "
            };
            assert_eq!(suffix.tokens, bytes(text));
            assert_ne!(
                suffix.tokens,
                tokenizer.encode(text),
                "BPE boundaries matter"
            );
            assert_eq!(suffix.response_mask, vec![0; suffix.tokens.len()]);
            assert_eq!(suffix.emitted_turns, 2);
            assert_eq!(suffix.special_token_occurrences, 0);
            assert!(!suffix.tokens.contains(&0), "no inserted BOS");
            assert!(!suffix.tokens.contains(&1), "no inserted EOS");
            assert_eq!(
                encoder.encode_assistant_suffix(&messages[..1], has_history),
                encoder.encode_user_prefix(messages[0].content, has_history)
            );
        }
        Ok(())
    }

    #[test]
    fn assistant_suffix_extends_closed_history_like_a_complete_prefix() -> Result<()> {
        let tokenizer = fixture(true, true);
        let encoder = DialogueProtocol::literal_roles_v1(&tokenizer)?.bind(&tokenizer)?;
        let history = [
            Message {
                role: "user",
                content: "old",
            },
            Message {
                role: "assistant",
                content: "Hello",
            },
        ];
        let messages = [
            Message {
                role: "user",
                content: "Hi",
            },
            Message {
                role: "system",
                content: "Memory: old",
            },
        ];
        for retained in [&history[..0], &history[..]] {
            let open = encoder.encode_open_history(retained);
            let suffix = encoder.encode_assistant_suffix(&messages, open.emitted_turns != 0);
            let whole = encoder.encode_assistant_prefix(&[retained, &messages].concat());
            assert_eq!([open.tokens, suffix.tokens].concat(), whole.tokens);
            assert_eq!(
                [open.response_mask, suffix.response_mask].concat(),
                whole.response_mask
            );
            assert_eq!(
                open.emitted_turns + suffix.emitted_turns,
                whole.emitted_turns
            );
            assert_eq!(whole.tokens.iter().filter(|&&id| id == 0).count(), 1);
            assert_eq!(
                whole.tokens.iter().filter(|&&id| id == 1).count(),
                usize::from(!retained.is_empty()),
                "the completed history's EOS is preserved exactly once"
            );
        }
        Ok(())
    }

    #[test]
    fn assistant_suffix_preserves_skips_response_masks_and_literal_specials() -> Result<()> {
        let tokenizer = fixture(false, true);
        let encoder = DialogueProtocol::literal_roles_v1(&tokenizer)?.bind(&tokenizer)?;
        let skipped = [
            Message {
                role: "unknown",
                content: " \r\n ",
            },
            Message {
                role: "tool",
                content: "ignored",
            },
        ];
        let messages = [
            skipped[0],
            skipped[1],
            Message {
                role: "system",
                content: "<|bos|>",
            },
            Message {
                role: "assistant",
                content: "A<|unk|>",
            },
            Message {
                role: "user",
                content: "<|eos|>",
            },
        ];
        let out = encoder.encode_assistant_suffix(&messages, true);
        let prefix = [bytes("\nSystem: "), vec![0], bytes("\nAssistant: ")].concat();
        let response = [bytes("A"), vec![2, 1]].concat();
        let tail = [bytes("\nUser: "), vec![1], bytes("\nAssistant: ")].concat();
        assert_eq!(
            out.tokens,
            [prefix.clone(), response.clone(), tail.clone()].concat()
        );
        assert_eq!(
            out.response_mask,
            [
                vec![0; prefix.len()],
                vec![1; response.len()],
                vec![0; tail.len()]
            ]
            .concat()
        );
        assert_eq!(out.emitted_turns, 3);
        assert_eq!(out.skipped_empty, 1);
        assert_eq!(out.skipped_unknown_role, 1);
        assert_eq!(out.special_token_occurrences, 3);
        for messages in [&skipped[..0], &skipped[..]] {
            let empty = encoder.encode_assistant_suffix(messages, true);
            assert_eq!(empty.tokens, bytes("Assistant: "));
            assert_eq!(empty.response_mask, vec![0; empty.tokens.len()]);
            assert_eq!(empty.emitted_turns, 0);
            assert_eq!(empty.special_token_occurrences, 0);
        }
        Ok(())
    }

    #[test]
    fn corpus_reference_bytes_masks_and_skips() -> Result<()> {
        let tokenizer = fixture(false, true);
        let encoder = DialogueProtocol::literal_roles_v1(&tokenizer)?.bind(&tokenizer)?;
        let out = encoder.encode_document(&[
            Message {
                role: "system",
                content: " S\r\n ",
            },
            Message {
                role: "unknown",
                content: "\t",
            },
            Message {
                role: "tool",
                content: "ignored",
            },
            Message {
                role: "user",
                content: " \r\nHé\r世界  !\t ",
            },
            Message {
                role: "assistant",
                content: " Oui\t! ",
            },
            Message {
                role: "user",
                content: " Again ",
            },
        ]);
        let prompt = bytes("System: S\nUser: Hé\n世界  !\nAssistant: ");
        let answer = bytes("Oui\t!");
        let tail = bytes("\nUser: Again");
        let expected = [
            vec![0],
            prompt.clone(),
            answer.clone(),
            vec![1],
            tail.clone(),
            vec![1],
        ]
        .concat();
        let mask = [
            vec![0; 1 + prompt.len()],
            vec![1; answer.len() + 1],
            vec![0; tail.len() + 1],
        ]
        .concat();
        assert_eq!(out.tokens, expected);
        assert_eq!(out.response_mask, mask);
        assert_eq!(
            (
                out.emitted_turns,
                out.skipped_empty,
                out.skipped_unknown_role
            ),
            (4, 1, 1)
        );
        assert_eq!(out.special_token_occurrences, 0);
        Ok(())
    }

    #[test]
    fn assistant_prefix_matches_training_segments_even_at_bpe_boundaries() -> Result<()> {
        let tokenizer = fixture(true, true);
        let encoder = DialogueProtocol::literal_roles_v1(&tokenizer)?.bind(&tokenizer)?;
        let history = [
            Message {
                role: "user",
                content: "Hi",
            },
            Message {
                role: "assistant",
                content: "Hello",
            },
            Message {
                role: "user",
                content: "Hi",
            },
        ];
        let prefix = encoder.encode_assistant_prefix(&history);
        let document = encoder.encode_document(&[
            history[0],
            history[1],
            history[2],
            Message {
                role: "assistant",
                content: "Hello",
            },
        ]);
        let expected = [
            vec![0],
            bytes("User: Hi\nAssistant: Hello"),
            vec![1],
            bytes("\nUser: Hi\nAssistant: "),
        ]
        .concat();
        assert_eq!(prefix.tokens, expected);
        assert_eq!(&document.tokens[..prefix.tokens.len()], prefix.tokens);
        assert_eq!(
            &document.response_mask[..prefix.response_mask.len()],
            prefix.response_mask
        );
        assert_eq!(
            &document.tokens[prefix.tokens.len()..],
            [bytes("Hello"), vec![1]].concat()
        );
        // Merging the whole rendered text changes the training token sequence.
        assert_ne!(
            tokenizer.encode("<|bos|>User: Hi\nAssistant: Hello<|eos|>\nUser: Hi\nAssistant: "),
            prefix.tokens
        );
        Ok(())
    }

    #[test]
    fn literal_control_tags_preserve_corpus_semantics() -> Result<()> {
        let tokenizer = fixture(false, true);
        let encoder = DialogueProtocol::literal_roles_v1(&tokenizer)?.bind(&tokenizer)?;
        let out = encoder.encode_document(&[Message {
            role: "assistant",
            content: "<|eos|><|unk|>",
        }]);
        assert_eq!(
            out.tokens,
            [vec![0], bytes("Assistant: "), vec![1, 2, 1]].concat()
        );
        assert_eq!(out.special_token_occurrences, 2);
        assert_eq!(&out.response_mask[out.response_mask.len() - 3..], [1, 1, 1]);
        let empty = encoder.encode_document(&[Message {
            role: "user",
            content: " ",
        }]);
        assert_eq!(empty.tokens, [0]);
        assert_eq!(empty.emitted_turns, 0);
        Ok(())
    }

    #[test]
    fn serialized_contract_rejects_unknown_schema_tokenizer_and_eos() -> Result<()> {
        let tokenizer = fixture(false, true);
        let protocol = DialogueProtocol::literal_roles_v1(&tokenizer)?;
        let encoded = serde_json::to_vec(&protocol).map_err(DialogueError::Json)?;
        let restored = DialogueProtocol::from_json(&encoded, &tokenizer)?;
        assert_eq!(restored, protocol);
        assert_eq!(restored.identity()?, protocol.identity()?);
        let mut bad = protocol.clone();
        bad.schema.push_str("-unknown");
        assert!(matches!(
            bad.bind(&tokenizer),
            Err(DialogueError::UnsupportedSchema(_))
        ));
        assert!(matches!(
            protocol.bind(&fixture(true, true)),
            Err(DialogueError::TokenizerMismatch { .. })
        ));
        bad = protocol;
        bad.eos_id = 2;
        assert!(matches!(
            bad.bind(&tokenizer),
            Err(DialogueError::SpecialIdMismatch { surface: EOS, .. })
        ));
        assert!(matches!(
            DialogueProtocol::literal_roles_v1(&fixture(false, false)),
            Err(DialogueError::InvalidSpecial { surface: EOS })
        ));
        assert!(matches!(
            DialogueProtocol::from_json(b"{}", &tokenizer),
            Err(DialogueError::Json(_))
        ));
        Ok(())
    }
}
