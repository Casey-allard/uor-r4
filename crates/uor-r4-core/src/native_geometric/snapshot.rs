//! Host-only, artifact-bound persistence of a bounded conversation context.

use super::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Checkpoint {
    schema: String,
    artifact_cid: String,
    retained_tokens: Vec<u32>,
    previous_evicted: Option<u32>,
    previous_h4: u16,
    h4: u16,
    phases: [u16; PHASE_CHANNELS],
    control: Control,
    work: Work,
}

impl Session {
    /// Persist the ordered retained window; scored candidates are transient
    /// and are recomputed by the next predict call.
    pub fn checkpoint(&self) -> Result<Vec<u8>> {
        let retained_tokens = if self.length == self.ring.len() {
            self.ring[self.cursor..]
                .iter()
                .chain(&self.ring[..self.cursor])
                .copied()
                .collect()
        } else {
            self.ring[..self.length].to_vec()
        };
        serde_json::to_vec(&Checkpoint {
            schema: "uor-r4.native-geometric-session/1".into(),
            artifact_cid: self.artifact_cid.clone(),
            retained_tokens,
            previous_evicted: self.previous_evicted,
            previous_h4: self.previous_h4,
            h4: self.h4,
            phases: self.phases,
            control: self.control,
            work: self.work,
        })
        .map_err(|error| Error(error.to_string()))
    }

    /// Rebuild the exact current geometric state from the ordered stored
    /// window. The separately retained evicted token verifies the preceding
    /// H4 trajectory state even after the ring filled.
    pub fn from_checkpoint(model: &Model, bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 1024 * 1024 {
            return Err(Error("session checkpoint exceeds 1 MiB".into()));
        }
        let checkpoint: Checkpoint =
            serde_json::from_slice(bytes).map_err(|error| Error(error.to_string()))?;
        if checkpoint.schema != "uor-r4.native-geometric-session/1"
            || checkpoint.artifact_cid != model.artifact_cid()
            || checkpoint.retained_tokens.len() > model.config().context_tokens
            || checkpoint.work.observed_tokens < checkpoint.retained_tokens.len() as u64
            || checkpoint.retained_tokens.len()
                != (checkpoint
                    .work
                    .observed_tokens
                    .min(model.config().context_tokens as u64) as usize)
            || checkpoint.work.evictions
                != checkpoint
                    .work
                    .observed_tokens
                    .saturating_sub(model.config().context_tokens as u64)
            || checkpoint.previous_evicted.is_some() != (checkpoint.work.evictions > 0)
        {
            return Err(Error(
                "session checkpoint identity/window counters are invalid".into(),
            ));
        }
        let mut restored = model.session(checkpoint.control)?;
        for &token in &checkpoint.retained_tokens {
            restored.observe(model, token)?;
        }
        let product = |left: u16, right: u16| {
            model.geometry.products
                [model.geometry.row_bases[usize::from(left)] + usize::from(right)]
        };
        let expected_previous = if let Some(&last) = checkpoint.retained_tokens.last() {
            let mut previous = product(
                restored.h4,
                model.geometry.inverses[usize::from(model.geometry.tokens[last as usize].leaf)],
            );
            if let Some(evicted) = checkpoint.previous_evicted {
                let old = model
                    .geometry
                    .tokens
                    .get(evicted as usize)
                    .ok_or_else(|| Error("session evicted token is invalid".into()))?;
                previous = product(old.leaf, previous);
            }
            previous
        } else {
            model.geometry.identity
        };
        if checkpoint.h4 != restored.h4
            || checkpoint.phases != restored.phases
            || checkpoint.previous_h4 != expected_previous
        {
            return Err(Error(
                "session geometric state does not match its ordered token window".into(),
            ));
        }
        restored.previous_h4 = expected_previous;
        restored.previous_evicted = checkpoint.previous_evicted;
        restored.work = checkpoint.work;
        Ok(restored)
    }
}

impl Model {
    pub fn restore_session(&self, bytes: &[u8]) -> Result<Session> {
        Session::from_checkpoint(self, bytes)
    }
}
