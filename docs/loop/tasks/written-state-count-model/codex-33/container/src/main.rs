#![forbid(unsafe_code)]

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::{BufWriter, Write};
use std::path::Path;
use uor_r4_count_runtime::{
    ngram_count::{
        context_bytes, triple_bytes, NgramContext, NgramTriple, ADJUSTED_CONTEXT_DOMAIN,
        ADJUSTED_TRIPLE_DOMAIN, CONTEXT_DOMAIN, TRIPLE_DOMAIN,
    },
    CompactCountLookup,
};
use uor_r4_integer::discrete_d11::count_deployment::{
    write_deployment, DiscreteCountModel, SCHEMA,
};
use uor_r4_integer::{IntegerError, Result};

struct Census {
    order: u32,
    triples: Sha256,
    contexts: Sha256,
    current: Option<NgramContext>,
    grams: u64,
    context_count: u64,
    count_of_counts: [u64; 4],
    targets: u64,
}

impl Census {
    fn new(order: u32, triple_domain: &[u8], context_domain: &[u8]) -> Self {
        let hash = |domain: &[u8]| {
            let mut hash = Sha256::new();
            hash.update(domain);
            hash.update([order as u8]);
            hash
        };
        Self {
            order,
            triples: hash(triple_domain),
            contexts: hash(context_domain),
            current: None,
            grams: 0,
            context_count: 0,
            count_of_counts: [0; 4],
            targets: 0,
        }
    }

    fn finish_context(&mut self) {
        if let Some(context) = self.current.take() {
            self.contexts.update(context_bytes(&context));
            self.context_count += 1;
        }
    }

    fn row(&mut self, key: u128, next: u16, count: u32) {
        if self.current.is_some_and(|context| context.key != key) {
            self.finish_context();
        }
        let context = self.current.get_or_insert(NgramContext {
            key,
            total: 0,
            followers: 0,
            n1: 0,
            n2: 0,
            n3_plus: 0,
        });
        context.total += u64::from(count);
        context.followers += 1;
        match count {
            1 => context.n1 += 1,
            2 => context.n2 += 1,
            _ => context.n3_plus += 1,
        }
        if let 1..=4 = count {
            self.count_of_counts[count as usize - 1] += 1;
        }
        self.triples
            .update(triple_bytes(&NgramTriple { key, next, count }));
        self.targets += u64::from(count);
        self.grams += 1;
    }

    fn finish(mut self) -> Value {
        self.finish_context();
        json!({
            "order": self.order,
            "targets": self.targets,
            "grams": self.grams,
            "contexts": self.context_count,
            "count_of_counts": self.count_of_counts,
            "triple_sha256": format!("{:x}", self.triples.finalize()),
            "context_sha256": format!("{:x}", self.contexts.finalize()),
        })
    }
}

fn census(counts: &CompactCountLookup<'_>) -> Result<Value> {
    let orders = 0..counts.order_count() as u32;
    let mut raw: Vec<_> = orders
        .clone()
        .map(|k| Census::new(k, TRIPLE_DOMAIN, CONTEXT_DOMAIN))
        .collect();
    let mut adjusted: Vec<_> = orders
        .map(|k| Census::new(k, ADJUSTED_TRIPLE_DOMAIN, ADJUSTED_CONTEXT_DOMAIN))
        .collect();
    counts
        .visit_counts(|k, key, next, raw_count, adjusted_count| {
            raw[k as usize].row(key, next, raw_count);
            adjusted[k as usize].row(key, next, adjusted_count);
        })
        .map_err(|error| IntegerError::Invalid(error.to_string()))?;
    Ok(json!({
        "raw": raw.into_iter().map(Census::finish).collect::<Vec<_>>(),
        "adjusted": adjusted.into_iter().map(Census::finish).collect::<Vec<_>>(),
    }))
}

fn readback(path: &Path) -> Result<()> {
    let bytes = std::fs::read(path)?;
    let deployment = DiscreteCountModel::load(&bytes, None)?;
    let (Some(identity), Some(state)) = (deployment.identity, deployment.state) else {
        return Err(IntegerError::Invalid(
            "expected an embedded count deployment".into(),
        ));
    };
    let receipt = json!({
        "schema": "uor-r4.d33-deployed-count-readback/1",
        "model_schema": SCHEMA,
        "artifact_sha256": identity.artifact_sha256,
        "p1_trunk_sha256": deployment.model.artifact_sha256(),
        "p1_state_sha256": state.artifact_sha256,
        "components": state.component_digests(),
        "bytes": {
            "total": bytes.len(),
            "header": identity.header_bytes,
            "trunk": identity.trunk_bytes,
            "p1_state": identity.state_bytes,
        },
        "counts": census(state.counts())?,
    });
    let mut output = BufWriter::new(std::io::stdout().lock());
    serde_json::to_writer_pretty(&mut output, &receipt)?;
    writeln!(output)?;
    output.flush()?;
    Ok(())
}

fn check_finite(path: &Path, trunk: &Path, state: &Path) -> Result<()> {
    let mut bytes = std::fs::read(path)?;
    let trunk = std::fs::read(trunk)?;
    let state = std::fs::read(state)?;
    if format!("{:x}", Sha256::digest(&state))
        != "c2bbb1141fe973db6804eda6d3f10135de4adf2791e94820dfd5024f7803d62c"
    {
        return Err(IntegerError::Invalid(
            "expected the D31 planted C128 fixture".into(),
        ));
    }
    let loaded = DiscreteCountModel::load(&bytes, None)?;
    let identity = loaded
        .identity
        .ok_or_else(|| IntegerError::Invalid("expected an embedded count deployment".into()))?;
    let state_digest = loaded
        .state
        .ok_or_else(|| IntegerError::Invalid("expected embedded P1 state".into()))?
        .artifact_sha256;
    let digest_offset = |digest: &str| {
        bytes[..identity.header_bytes]
            .windows(digest.len())
            .position(|window| window == digest.as_bytes())
            .ok_or_else(|| IntegerError::Invalid("missing deployment digest".into()))
    };
    let trunk_digest_offset = digest_offset(loaded.model.artifact_sha256())?;
    let state_digest_offset = digest_offset(&state_digest)?;
    let state_start = identity.header_bytes + identity.trunk_bytes;
    if bytes[identity.header_bytes..state_start] != trunk || bytes[state_start..] != state {
        return Err(IntegerError::Invalid(
            "finite packing changed component bytes".into(),
        ));
    }
    let rejection = |name, buffer: &[u8], sidecar| -> Result<String> {
        match DiscreteCountModel::load(buffer, sidecar) {
            Ok(_) => Err(IntegerError::Invalid(format!("accepted planted {name}"))),
            Err(error) => Ok(error.to_string()),
        }
    };
    let conflict = rejection("sidecar conflict", &bytes, Some(state.as_slice()))?;
    let truncation = rejection("truncation", &bytes[..bytes.len() - 1], None)?;
    let size_word = |range: std::ops::Range<usize>| -> Result<usize> {
        let word = state[range]
            .try_into()
            .map_err(|_| IntegerError::Invalid("invalid finite P1 header size".into()))?;
        Ok(u32::from_le_bytes(word) as usize)
    };
    let fit_len = size_word(8..12)?;
    let weight_len = size_word(12..16)?;
    let mut corruptions = Vec::new();
    let last = bytes.len() - 1;
    for (component, offset) in [
        ("trunk", state_start - 1),
        ("fit_manifest", state_start + 48 + fit_len - 1),
        ("weights", state_start + 48 + fit_len + weight_len - 1),
        ("counts", last),
        ("header_trunk_digest", trunk_digest_offset),
        ("header_state_digest", state_digest_offset),
    ] {
        bytes[offset] ^= 1;
        let error = rejection(component, &bytes, None)?;
        let corrupt_sha = format!("{:x}", Sha256::digest(&bytes));
        if corrupt_sha == identity.artifact_sha256 {
            return Err(IntegerError::Invalid(format!(
                "{component} corruption kept identity"
            )));
        }
        bytes[offset] ^= 1;
        corruptions.push(json!({
            "component": component, "offset": offset,
            "corrupted_sha256": corrupt_sha, "error": error,
        }));
    }
    // A structurally valid trunk can be rehashed by an untrusted packer. P1
    // must still reject it against the original fit binding.
    bytes[state_start - 1] ^= 1;
    let changed_trunk = format!(
        "{:x}",
        Sha256::digest(&bytes[identity.header_bytes..state_start])
    );
    bytes[trunk_digest_offset..trunk_digest_offset + changed_trunk.len()]
        .copy_from_slice(changed_trunk.as_bytes());
    let rehashed_trunk = rejection("rehashed wrong trunk", &bytes, None)?;
    if !rehashed_trunk.contains("P1 fit or trunk identity mismatch") {
        return Err(IntegerError::Invalid(format!(
            "wrong-trunk control did not reach the fit binding: {rehashed_trunk}"
        )));
    }
    println!(
        "{}",
        json!({
            "schema": "uor-r4.d33-deployment-finite-controls/1",
            "artifact_sha256": identity.artifact_sha256,
            "original_components_byte_equal": true,
            "sidecar_conflict": conflict,
            "truncation": truncation,
            "rehashed_trunk": {
                "trunk_sha256": changed_trunk,
                "artifact_sha256": format!("{:x}", Sha256::digest(&bytes)),
                "error": rehashed_trunk,
            },
            "corruptions": corruptions,
        })
    );
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match args.as_slice() {
        [command, trunk, state, output] if command == "pack" => {
            let trunk = std::fs::read(trunk)?;
            let state = std::fs::read(state)?;
            write_deployment(&trunk, &state, Path::new(output))
        }
        [command, input] if command == "readback" => readback(Path::new(input)),
        [command, input, trunk, state] if command == "check-finite" => {
            check_finite(Path::new(input), Path::new(trunk), Path::new(state))
        }
        _ => Err(IntegerError::Invalid(
            "usage: d33-count-container pack <trunk> <p1> <output> | readback <deployment> | check-finite <deployment> <trunk> <p1>".into(),
        )),
    }
}
