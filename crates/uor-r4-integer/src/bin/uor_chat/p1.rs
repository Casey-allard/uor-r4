use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_count_runtime::{CompactCountLookup, FrozenPickMixture};
use uor_r4_integer::{discrete_d11::IntegerDiscreteModel, IntegerError, Result};

const A1_TRUNK: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";

fn invalid(s: impl ToString) -> IntegerError {
    IntegerError::Invalid(s.to_string())
}

#[derive(Deserialize)]
struct Fit {
    schema: String,
    prepared: Preparation,
}

#[derive(Deserialize)]
struct Preparation {
    format: String,
    c_sha256: String,
    trunk_sha256: String,
    compact_sha256: String,
}

fn digest(hex_string: &str) -> Result<[u8; 32]> {
    let mut bytes = [0; 32];
    hex::decode_to_slice(hex_string, &mut bytes).map_err(invalid)?;
    Ok(bytes)
}

pub struct P1<'a> {
    counts: CompactCountLookup<'a>,
    weights: FrozenPickMixture,
    codes: Vec<[u16; 4]>,
    pub artifact_sha256: String,
    count_sha256: String,
    weights_sha256: String,
    fit_manifest_sha256: String,
}

impl<'a> P1<'a> {
    pub fn load(bytes: &'a [u8], model: &IntegerDiscreteModel) -> Result<Self> {
        let (header, payload) = bytes
            .split_at_checked(48)
            .ok_or_else(|| invalid("truncated P1 package"))?;
        if &header[..8] != b"UORP1A01" {
            return Err(invalid("unknown P1 package format"));
        }
        let size = |word: &[u8]| -> Result<usize> {
            Ok(u32::from_le_bytes(word.try_into().map_err(invalid)?) as usize)
        };
        let (fit, rest) = payload
            .split_at_checked(size(&header[8..12])?)
            .ok_or_else(|| invalid("truncated P1 fit manifest"))?;
        let (weights, counts) = rest
            .split_at_checked(size(&header[12..16])?)
            .ok_or_else(|| invalid("truncated P1 weights"))?;
        if [counts, weights]
            .iter()
            .any(|part| part.get(8..12) != Some(10u32.to_le_bytes().as_slice()))
        {
            return Err(invalid(
                "P1 requires order ten for both count and fit components",
            ));
        }
        let weight_digest = Sha256::digest(weights);
        if weight_digest[..] != header[16..] {
            return Err(invalid("P1 weight digest mismatch"));
        }
        let manifest: Fit = serde_json::from_slice(fit)?;
        let prepared = manifest.prepared;
        if manifest.schema != "uor-r4.d31.a1-fit/1"
            || prepared.format != "uor-r4.d31.a1-preparation/1"
            || prepared.trunk_sha256 != A1_TRUNK
            || prepared.trunk_sha256 != model.artifact_sha256()
            || model.shape().context != 192
            || model.shape().vocabulary != 4096
            || model.shape().blocks != 4
        {
            return Err(invalid("P1 fit or trunk identity mismatch"));
        }
        let fit_digest = Sha256::digest(fit);
        let weights = FrozenPickMixture::from_bytes(weights, fit_digest.into()).map_err(invalid)?;
        let counts = CompactCountLookup::from_bytes(
            counts,
            digest(&prepared.c_sha256)?,
            digest(&prepared.compact_sha256)?,
        )
        .map_err(invalid)?;
        Ok(Self {
            counts,
            weights,
            codes: model.identity_pick_codes()?,
            artifact_sha256: hex::encode(Sha256::digest(bytes)),
            count_sha256: prepared.compact_sha256,
            weights_sha256: hex::encode(weight_digest),
            fit_manifest_sha256: hex::encode(fit_digest),
        })
    }

    pub fn component_digests(&self) -> Value {
        json!({
            "count_sha256": self.count_sha256,
            "weights_sha256": self.weights_sha256,
            "fit_manifest_sha256": self.fit_manifest_sha256
        })
    }

    pub fn next(
        &self,
        context: &[u32],
        read_enabled: bool,
        trace: bool,
    ) -> Result<(u32, Option<Value>)> {
        // The client can slide a full window. Raw-id prompts may also include
        // completed records, so the native read begins at the last BOS.
        let start = context.iter().rposition(|&id| id == 0).unwrap_or(0);
        let history = &context[start..];
        if history.is_empty() || history.len() > 192 || history.contains(&1) {
            return Err(invalid("P1 decoding needs an open causal record"));
        }
        let mut ids = [0u16; 192];
        for (out, &id) in ids.iter_mut().zip(history) {
            *out = u16::try_from(id).map_err(invalid)?;
        }
        let ids = &ids[..history.len()];
        if ids.iter().any(|&id| usize::from(id) >= self.codes.len()) {
            return Err(invalid("P1 prefix contains an invalid original token ID"));
        }
        let pick = read_enabled
            .then(|| uor_r4_count_runtime::code_pick(&self.codes, ids))
            .flatten();
        let choice = pick.map(|(position, distance, _)| (ids[position + 1], distance));
        let mut best = (0u16, 0u16, 0u64);
        let mut ties = 0u32;
        let mut masses = trace.then(Sha256::new);
        self.counts
            .visit_next_masses(ids, &self.weights, choice, |id, bucket, mass| {
                if let Some(masses) = &mut masses {
                    masses.update(mass.to_le_bytes());
                }
                if mass > best.2 {
                    best = (id, bucket, mass);
                    ties = 1;
                } else if mass == best.2 {
                    ties += 1;
                }
            })
            .map_err(invalid)?;
        let (id, bucket, mass) = best;
        if id == 0 {
            return Err(invalid("P1 read found no positive legal mass"));
        }
        Ok((u32::from(id), masses.map(|masses| json!({"token":id,"bucket":bucket,"mass_q48":mass,"top_ties":ties,"mass_sha256":hex::encode(masses.finalize()),
            "prefix_sha256":({
                let mut hash = Sha256::new();
                for word in ids { hash.update(word.to_le_bytes()); }
                hex::encode(hash.finalize())
            }),"context_tokens":ids.len(),"pick_successor":choice.map(|(s,_)|s),"pick_words":pick.map(|(position,distance,ties)| [position as u32,distance,ties])}))))
    }
}
