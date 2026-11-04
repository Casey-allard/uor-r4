//! Artifact-bound nearest-example content readout for FBS serving.
//!
//! The readout is a sidecar: it never changes `UORLUT01` bytes. A sidecar binds
//! its rows to one artifact's SHA-256, and `IntegerStackModel::serve_step`
//! consults it only after the native writer and point-key fallback have
//! selected a token.

use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::point_key::POINT_KEY_BOS;
use super::StackError;

/// Schema of a serving content-readout sidecar.
pub const STACK_CONTENT_READOUT_SCHEMA: &str = "uor-r4.stack-content-readout/1";
/// The only serving signature supported by this sidecar version.
pub const CONTENT_CODE_SIGNATURE_BITS: usize = 2_048;
const CONTENT_CODE_SIGNATURE_WORDS: usize = CONTENT_CODE_SIGNATURE_BITS / 64;
/// Refuse a sidecar larger than this before JSON parsing.
pub const MAX_CONTENT_READOUT_BYTES: u64 = 64 << 20;

/// A 2,048-bit content signature, packed as 32 little-endian words.
pub type ContentSignature = Vec<u64>;

/// The supported content-signature mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentSignatureMode {
    Code,
}

/// One signature and its reference-content target.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentExample {
    pub signature: ContentSignature,
    pub target: u32,
}

/// The validated nearest-example table.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentReadout {
    pub mode: ContentSignatureMode,
    pub signature_bits: usize,
    pub includes_reference_end: bool,
    pub rows: Vec<ContentExample>,
    pub max_distance: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContentReadoutSidecar {
    schema: String,
    artifact_sha256: String,
    readout: ContentReadout,
}

/// The result of applying a serving content readout to one writer query.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContentSelection {
    pub token: u32,
    pub nearest_distance: u32,
    pub readout_fired: bool,
}

impl ContentReadout {
    /// Parse and validate a sidecar against the loaded artifact and vocabulary.
    pub fn from_sidecar(
        bytes: &[u8],
        artifact_sha256: &str,
        vocab: usize,
    ) -> Result<Self, StackError> {
        if bytes.len() as u64 > MAX_CONTENT_READOUT_BYTES {
            return Err(StackError::ContentReadout(format!(
                "sidecar exceeds {MAX_CONTENT_READOUT_BYTES} bytes"
            )));
        }
        let sidecar: ContentReadoutSidecar = serde_json::from_slice(bytes).map_err(|error| {
            StackError::ContentReadout(format!("invalid sidecar JSON: {error}"))
        })?;
        if sidecar.schema != STACK_CONTENT_READOUT_SCHEMA {
            return Err(StackError::ContentReadout(format!(
                "schema {:?} is not {STACK_CONTENT_READOUT_SCHEMA:?}",
                sidecar.schema
            )));
        }
        if sidecar.artifact_sha256.len() != 64
            || !sidecar
                .artifact_sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(StackError::ContentReadout(
                "artifact_sha256 is not 64 lowercase hex characters".to_owned(),
            ));
        }
        if sidecar.artifact_sha256 != artifact_sha256 {
            return Err(StackError::ContentReadout(format!(
                "sidecar binds artifact {}, loaded artifact is {artifact_sha256}",
                sidecar.artifact_sha256
            )));
        }
        sidecar.readout.validate(vocab)?;
        Ok(sidecar.readout)
    }

    /// Read and validate a sidecar file.
    pub fn load(path: &Path, artifact_sha256: &str, vocab: usize) -> Result<Self, StackError> {
        Self::read_sidecar(path, artifact_sha256, vocab).map(|(_, readout)| readout)
    }

    /// Read a sidecar without allocating beyond the size limit, returning the
    /// exact bytes alongside the validated table so callers can identify the
    /// sidecar without rereading it.
    pub fn read_sidecar(
        path: &Path,
        artifact_sha256: &str,
        vocab: usize,
    ) -> Result<(Vec<u8>, Self), StackError> {
        let file = std::fs::File::open(path).map_err(StackError::Io)?;
        let mut bytes = Vec::new();
        file.take(MAX_CONTENT_READOUT_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(StackError::Io)?;
        if bytes.len() as u64 > MAX_CONTENT_READOUT_BYTES {
            return Err(StackError::ContentReadout(format!(
                "sidecar exceeds {MAX_CONTENT_READOUT_BYTES} bytes"
            )));
        }
        let readout = Self::from_sidecar(&bytes, artifact_sha256, vocab)?;
        Ok((bytes, readout))
    }

    /// Validate rows, target ids and the learned Hamming radius.
    pub fn validate(&self, vocab: usize) -> Result<(), StackError> {
        if self.mode != ContentSignatureMode::Code
            || self.signature_bits != CONTENT_CODE_SIGNATURE_BITS
        {
            return Err(StackError::ContentReadout(
                "serving requires the 2,048-bit code signature".to_owned(),
            ));
        }
        if self.includes_reference_end {
            return Err(StackError::ContentReadout(
                "serving content readout must not include terminal BOS rows".to_owned(),
            ));
        }
        if self.rows.is_empty() {
            return Err(StackError::ContentReadout(
                "serving content readout has no rows".to_owned(),
            ));
        }
        if self.max_distance as usize > self.signature_bits {
            return Err(StackError::ContentReadout(
                "content-readout threshold exceeds the signature".to_owned(),
            ));
        }
        for row in &self.rows {
            if row.signature.len() != self.signature_bits / 64 {
                return Err(StackError::ContentReadout(
                    "content-readout row width differs".to_owned(),
                ));
            }
            if row.target as usize >= vocab {
                return Err(StackError::ContentReadout(format!(
                    "content-readout target {} is outside vocabulary {vocab}",
                    row.target
                )));
            }
            if row.target == POINT_KEY_BOS {
                return Err(StackError::ContentReadout(
                    "content-readout serving rows must not target BOS".to_owned(),
                ));
            }
        }
        Ok(())
    }

    /// Pack the low byte of each Q12 coordinate into a 2,048-bit signature.
    fn signature_array(
        query_q12: &[i32],
    ) -> Result<[u64; CONTENT_CODE_SIGNATURE_WORDS], StackError> {
        if query_q12.len() * 8 != CONTENT_CODE_SIGNATURE_BITS {
            return Err(StackError::ContentReadout(format!(
                "content readout needs a 256-coordinate query, got {}",
                query_q12.len()
            )));
        }
        let mut signature = [0u64; CONTENT_CODE_SIGNATURE_WORDS];
        for (index, &value) in query_q12.iter().enumerate() {
            let bit = index * 8;
            signature[bit / 64] |= u64::from(value as u8) << (bit % 64);
        }
        Ok(signature)
    }

    /// Pack the low byte of each Q12 coordinate into a 2,048-bit signature.
    pub fn signature(query_q12: &[i32]) -> Result<ContentSignature, StackError> {
        Ok(Self::signature_array(query_q12)?.to_vec())
    }

    /// Select the nearest target within the learned radius.
    ///
    /// Equal minimum distances vote by target. The commonest target wins, and
    /// an exact vote tie goes to the smaller token id.
    pub fn select(&self, query_q12: &[i32], fallback: u32) -> Result<ContentSelection, StackError> {
        let vocab = self
            .rows
            .iter()
            .map(|row| row.target as usize)
            .max()
            .map_or(0, |target| target + 1);
        let mut votes = vec![0; vocab];
        let mut vote_epochs = vec![0; vocab];
        let mut epoch = 0;
        self.select_into(
            query_q12,
            fallback,
            &mut votes,
            &mut vote_epochs,
            &mut epoch,
        )
    }

    /// Select using caller-owned vote scratch. The serving path calls this
    /// with buffers allocated by the session so a token step does not allocate.
    pub fn select_into(
        &self,
        query_q12: &[i32],
        fallback: u32,
        votes: &mut [u32],
        vote_epochs: &mut [u32],
        epoch: &mut u32,
    ) -> Result<ContentSelection, StackError> {
        if votes.len() != vote_epochs.len() {
            return Err(StackError::ContentReadout(
                "content-readout vote scratch widths differ".to_owned(),
            ));
        }
        let vote_len = votes.len();
        let signature = Self::signature_array(query_q12)?;
        let mut nearest = u32::MAX;
        let mut nearest_epoch = 0;
        for row in &self.rows {
            if row.signature.len() != CONTENT_CODE_SIGNATURE_WORDS {
                return Err(StackError::ContentReadout(
                    "content-readout row width differs".to_owned(),
                ));
            }
            let distance: u32 = signature
                .iter()
                .zip(&row.signature)
                .map(|(&left, &right)| (left ^ right).count_ones())
                .sum();
            match distance.cmp(&nearest) {
                std::cmp::Ordering::Less => {
                    nearest = distance;
                    nearest_epoch = next_vote_epoch(epoch, vote_epochs);
                    let target = row.target as usize;
                    let vote = votes.get_mut(target).ok_or_else(|| {
                        StackError::ContentReadout(format!(
                            "content-readout target {} exceeds vote scratch {}",
                            row.target, vote_len
                        ))
                    })?;
                    *vote = 1;
                    vote_epochs[target] = nearest_epoch;
                }
                std::cmp::Ordering::Equal => {
                    let target = row.target as usize;
                    let vote = votes.get_mut(target).ok_or_else(|| {
                        StackError::ContentReadout(format!(
                            "content-readout target {} exceeds vote scratch {}",
                            row.target, vote_len
                        ))
                    })?;
                    if vote_epochs[target] != nearest_epoch {
                        *vote = 1;
                        vote_epochs[target] = nearest_epoch;
                    } else {
                        *vote = vote.checked_add(1).ok_or_else(|| {
                            StackError::ContentReadout(
                                "content-readout target vote overflowed".to_owned(),
                            )
                        })?;
                    }
                }
                std::cmp::Ordering::Greater => {}
            }
        }
        let mut target = None;
        let mut most_votes = 0;
        for (candidate, &count) in votes.iter().enumerate() {
            if vote_epochs[candidate] == nearest_epoch && count > most_votes {
                target = Some(candidate as u32);
                most_votes = count;
            }
        }
        let target = target.ok_or_else(|| {
            StackError::ContentReadout("content readout has no effective target".to_owned())
        })?;
        let fired = nearest <= self.max_distance;
        Ok(ContentSelection {
            token: if fired { target } else { fallback },
            nearest_distance: nearest,
            readout_fired: fired,
        })
    }
}

fn next_vote_epoch(epoch: &mut u32, vote_epochs: &mut [u32]) -> u32 {
    if *epoch == u32::MAX {
        vote_epochs.fill(0);
        *epoch = 1;
    } else {
        *epoch += 1;
    }
    *epoch
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signature(bit: usize) -> ContentSignature {
        let mut signature = vec![0; CONTENT_CODE_SIGNATURE_BITS / 64];
        signature[bit / 64] |= 1u64 << (bit % 64);
        signature
    }

    fn readout(rows: Vec<ContentExample>, max_distance: u32) -> ContentReadout {
        ContentReadout {
            mode: ContentSignatureMode::Code,
            signature_bits: CONTENT_CODE_SIGNATURE_BITS,
            includes_reference_end: false,
            rows,
            max_distance,
        }
    }

    #[test]
    fn minimum_rows_use_commonest_target_then_smaller_token() {
        let query = vec![0; CONTENT_CODE_SIGNATURE_BITS / 64];
        let table = readout(
            vec![
                ContentExample {
                    signature: query.clone(),
                    target: 9,
                },
                ContentExample {
                    signature: query.clone(),
                    target: 7,
                },
                ContentExample {
                    signature: query.clone(),
                    target: 7,
                },
                ContentExample {
                    signature: signature(1),
                    target: 3,
                },
            ],
            0,
        );
        assert_eq!(
            table.select(&vec![0; 256], 99).expect("select"),
            ContentSelection {
                token: 7,
                nearest_distance: 0,
                readout_fired: true,
            }
        );

        let tie = readout(
            vec![
                ContentExample {
                    signature: query.clone(),
                    target: 9,
                },
                ContentExample {
                    signature: query,
                    target: 7,
                },
            ],
            0,
        );
        assert_eq!(tie.select(&vec![0; 256], 99).expect("select").token, 7);
    }

    #[test]
    fn beyond_the_radius_returns_the_native_fallback() {
        let table = readout(
            vec![ContentExample {
                signature: signature(0),
                target: 7,
            }],
            0,
        );
        assert_eq!(
            table.select(&vec![0; 256], 42).expect("select"),
            ContentSelection {
                token: 42,
                nearest_distance: 1,
                readout_fired: false,
            }
        );
    }

    #[test]
    fn served_selection_reuses_scratch_and_ignores_stale_votes() {
        let mut query = vec![0; 256];
        query[0] = 1;
        let table = readout(
            vec![
                ContentExample {
                    signature: signature(0),
                    target: 9,
                },
                ContentExample {
                    signature: signature(0),
                    target: 9,
                },
                ContentExample {
                    signature: signature(1),
                    target: 3,
                },
            ],
            0,
        );
        let mut votes = [0; 10];
        let mut vote_epochs = [0; 10];
        let mut epoch = u32::MAX;

        assert_eq!(
            table
                .select_into(&query, 99, &mut votes, &mut vote_epochs, &mut epoch)
                .expect("first selection")
                .token,
            9
        );
        let mut second_query = query;
        second_query[0] = 2;
        assert_eq!(
            table
                .select_into(&second_query, 99, &mut votes, &mut vote_epochs, &mut epoch)
                .expect("second selection")
                .token,
            3
        );
    }

    #[test]
    fn sidecar_is_artifact_bound_and_strict() {
        let sha = "a".repeat(64);
        let sidecar = serde_json::json!({
            "schema": STACK_CONTENT_READOUT_SCHEMA,
            "artifact_sha256": sha,
            "readout": readout(
                vec![ContentExample {
                    signature: signature(0),
                    target: 1,
                }],
                8,
            ),
        });
        let bytes = serde_json::to_vec(&sidecar).expect("json");
        assert!(ContentReadout::from_sidecar(&bytes, &"a".repeat(64), 2).is_ok());

        let mut wrong_artifact = sidecar.clone();
        wrong_artifact["artifact_sha256"] = serde_json::json!("b".repeat(64));
        assert!(ContentReadout::from_sidecar(
            &serde_json::to_vec(&wrong_artifact).expect("json"),
            &"a".repeat(64),
            2
        )
        .is_err());

        for field in ["schema", "artifact_sha256", "readout"] {
            let mut missing = sidecar.clone();
            missing.as_object_mut().expect("object").remove(field);
            assert!(ContentReadout::from_sidecar(
                &serde_json::to_vec(&missing).expect("json"),
                &"a".repeat(64),
                2
            )
            .is_err());
        }
        let mut missing_terminal_flag = sidecar.clone();
        missing_terminal_flag["readout"]
            .as_object_mut()
            .expect("readout object")
            .remove("includes_reference_end");
        assert!(ContentReadout::from_sidecar(
            &serde_json::to_vec(&missing_terminal_flag).expect("json"),
            &"a".repeat(64),
            2
        )
        .is_err());
        let mut unknown = sidecar;
        unknown["extra"] = serde_json::json!(true);
        assert!(ContentReadout::from_sidecar(
            &serde_json::to_vec(&unknown).expect("json"),
            &"a".repeat(64),
            2
        )
        .is_err());
    }

    #[test]
    fn parser_refuses_every_remaining_registered_sidecar_fault() {
        let sha = "a".repeat(64);
        let valid = serde_json::json!({
            "schema": STACK_CONTENT_READOUT_SCHEMA,
            "artifact_sha256": sha,
            "readout": readout(
                vec![ContentExample {
                    signature: signature(0),
                    target: 1,
                }],
                8,
            ),
        });
        let mut cases = Vec::new();

        let mut wrong_schema = valid.clone();
        wrong_schema["schema"] = serde_json::json!("uor-r4.stack-content-readout/2");
        cases.push(wrong_schema);

        let mut sign_mode = valid.clone();
        sign_mode["readout"]["mode"] = serde_json::json!("sign");
        cases.push(sign_mode);

        let mut no_rows = valid.clone();
        no_rows["readout"]["rows"] = serde_json::json!([]);
        cases.push(no_rows);

        let mut wide_threshold = valid.clone();
        wide_threshold["readout"]["max_distance"] =
            serde_json::json!(CONTENT_CODE_SIGNATURE_BITS as u32 + 1);
        cases.push(wide_threshold);

        let mut malformed_threshold = valid.clone();
        malformed_threshold["readout"]["max_distance"] = serde_json::json!("many");
        cases.push(malformed_threshold);

        let mut unknown_readout = valid.clone();
        unknown_readout["readout"]["extra"] = serde_json::json!(true);
        cases.push(unknown_readout);

        let mut unknown_row = valid.clone();
        unknown_row["readout"]["rows"][0]["extra"] = serde_json::json!(true);
        cases.push(unknown_row);

        for case in cases {
            assert!(ContentReadout::from_sidecar(
                &serde_json::to_vec(&case).expect("json"),
                &"a".repeat(64),
                2
            )
            .is_err());
        }

        let mut trailing = serde_json::to_vec(&valid).expect("json");
        trailing.extend_from_slice(b"x");
        assert!(ContentReadout::from_sidecar(&trailing, &"a".repeat(64), 2).is_err());

        let oversized = vec![b' '; MAX_CONTENT_READOUT_BYTES as usize + 1];
        assert!(ContentReadout::from_sidecar(&oversized, &"a".repeat(64), 2).is_err());
    }

    #[test]
    fn validation_refuses_unsafe_serving_tables() {
        let valid = readout(
            vec![ContentExample {
                signature: signature(0),
                target: 1,
            }],
            8,
        );
        assert!(valid.validate(2).is_ok());

        let mut wrong_width = valid.clone();
        wrong_width.rows[0].signature.pop();
        assert!(wrong_width.validate(2).is_err());

        let mut wrong_target = valid.clone();
        wrong_target.rows[0].target = 2;
        assert!(wrong_target.validate(2).is_err());

        let mut bos = valid.clone();
        bos.rows[0].target = POINT_KEY_BOS;
        assert!(bos.validate(2).is_err());

        let mut terminal = valid.clone();
        terminal.includes_reference_end = true;
        assert!(terminal.validate(2).is_err());

        let mut sign = valid.clone();
        sign.signature_bits = 256;
        assert!(sign.validate(2).is_err());

        let mut wide = valid;
        wide.max_distance = CONTENT_CODE_SIGNATURE_BITS as u32 + 1;
        assert!(wide.validate(2).is_err());
    }
}
