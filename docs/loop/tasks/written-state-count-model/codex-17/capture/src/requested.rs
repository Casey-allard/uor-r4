use super::*;
use uor_r4_integer::paired_role::{PairedRoleDecision, RoleSpan};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

const RAW_SCHEMA: &str = "uor-r4.sealed-requested-names/1";
const RESOLVED_SCHEMA: &str = "uor-r4.resolved-requested-spans/1";
const SELECT_SCHEMA: &str = "uor-r4.ordered-role.selections/1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRow {
    index: usize,
    text: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Endpoint {
    bytes: [usize; 2],
    name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SealedRow {
    index: usize,
    text_sha256: String,
    requested: Option<Endpoint>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSeal {
    schema: String,
    raw_sha256: String,
    rows: Vec<SealedRow>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ResolvedRow {
    pub index: usize,
    pub prefix_sha256: String,
    pub requested_span: Option<[usize; 2]>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Resolved {
    schema: String,
    pub blind_sha256: String,
    pub tokenizer_sha256: String,
    raw_sha256: String,
    name_seal_sha256: String,
    pub rows: Vec<ResolvedRow>,
}

fn pinned(path: &Path, expected: &str) -> Result<Vec<u8>> {
    let bytes = fs::read(path)?;
    need(
        expected.len() == 64 && sha(&bytes) == expected,
        "sealed input checksum mismatch",
    )?;
    Ok(bytes)
}

fn original_span(
    text: &str,
    ids: &[u32],
    tokenizer: &ByteBpeTokenizer,
    endpoint: &Endpoint,
) -> Result<[usize; 2]> {
    let [start, end] = endpoint.bytes;
    need(
        start < end
            && text.get(start..end) == Some(endpoint.name.as_str())
            && endpoint.name.trim() == endpoint.name
            && !endpoint.name.chars().any(char::is_control)
            && text
                .get(..start)
                .and_then(|s| s.chars().next_back())
                .is_none_or(|c| !c.is_alphanumeric())
            && text
                .get(end..)
                .and_then(|s| s.chars().next())
                .is_none_or(|c| !c.is_alphanumeric()),
        "requested span does not name the sealed complete word",
    )?;
    let head = "<|bos|>User: ";
    let tail = "\nAssistant:";
    let full = format!("{head}{text}{tail}");
    let mut offsets = vec![0usize];
    for &id in ids {
        offsets.push(
            offsets.last().ok_or("missing byte offset")?
                + tokenizer
                    .token_byte_len(id)
                    .ok_or("undefined or empty original token")?,
        );
    }
    let absolute = head.len() + start;
    let right = offsets
        .binary_search(&(head.len() + end))
        .map_err(|_| "name end is not an original token boundary")?;
    let left = offsets
        .binary_search(&absolute)
        .or_else(|_| {
            if full.as_bytes().get(absolute - 1) == Some(&b' ') {
                offsets.binary_search(&(absolute - 1))
            } else {
                Err(absolute)
            }
        })
        .map_err(|_| "name start is not an original token boundary")?;
    let value = tokenizer.decode_bytes(&ids[left..right]);
    need(
        left < right
            && (value == endpoint.name.as_bytes()
                || value.strip_prefix(b" ") == Some(endpoint.name.as_bytes())),
        "name overflows the original token span",
    )?;
    Ok([left, right])
}

pub fn prepare(args: &[String]) -> Result<()> {
    let [tokenizer, tokenizer_sha, raw, raw_sha, names, names_sha, out] = args else {
        return Err("prepare TOKENIZER TOKENIZER_SHA RAW_TEXT_ONLY RAW_SHA REQUESTED_NAMES REQUESTED_SHA NEW_DIR".into());
    };
    let tokenizer_bytes = pinned(Path::new(tokenizer), tokenizer_sha)?;
    let raw_bytes = pinned(Path::new(raw), raw_sha)?;
    let names_bytes = pinned(Path::new(names), names_sha)?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or("invalid tokenizer JSON")?;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer)?;
    let encoder = protocol.bind(&tokenizer)?;
    let raw: Vec<RawRow> = serde_json::from_slice(&raw_bytes)?;
    let seal: RawSeal = serde_json::from_slice(&names_bytes)?;
    need(
        seal.schema == RAW_SCHEMA
            && seal.raw_sha256 == *raw_sha
            && !raw.is_empty()
            && raw.len() == seal.rows.len(),
        "requested-name seal does not bind the raw panel",
    )?;
    let mut prompts = Vec::with_capacity(raw.len());
    let mut requests = Vec::with_capacity(raw.len());
    for (index, (text, supplied)) in raw.into_iter().zip(seal.rows).enumerate() {
        need(
            text.index == index
                && supplied.index == index
                && sha(text.text.as_bytes()) == supplied.text_sha256
                && !text.text.is_empty()
                && text.text.trim() == text.text
                && !text
                    .text
                    .chars()
                    .any(|ch| ch.is_control() && ch != '\n' && ch != '\t'),
            "stale, reordered, or noncanonical raw text",
        )?;
        let ids = encoder
            .encode_assistant_prefix(&[Message {
                role: "user",
                content: &text.text,
            }])
            .tokens;
        need(
            ids.len() <= ROLE_CONTEXT
                && ids.first() == Some(&0)
                && protocol.bos_id == 0
                && protocol.eos_id == 1
                && ids
                    .iter()
                    .all(|&id| id < ROLE_VOCAB as u32 && id != protocol.unk_id),
            "raw text does not fit the exact original vocabulary and context",
        )?;
        need(
            tokenizer.decode_bytes(&ids)
                == format!("<|bos|>User: {}\nAssistant:", text.text).as_bytes(),
            "protocol-2 prefix bytes changed",
        )?;
        let requested_span = supplied
            .requested
            .as_ref()
            .map(|item| original_span(&text.text, &ids, &tokenizer, item))
            .transpose()?;
        requests.push(ResolvedRow {
            index,
            prefix_sha256: prefix_sha(&ids),
            requested_span,
        });
        prompts.push(json!({"prefix":ids,"text":text.text}));
    }
    // Preserve the frozen blind writer's pretty JSON and final LF too.
    let mut bytes = serde_json::to_vec_pretty(&prompts)?;
    bytes.push(b'\n');
    let resolved = Resolved {
        schema: RESOLVED_SCHEMA.into(),
        blind_sha256: sha(&bytes),
        tokenizer_sha256: tokenizer_sha.clone(),
        raw_sha256: raw_sha.clone(),
        name_seal_sha256: names_sha.clone(),
        rows: requests,
    };
    fs::create_dir(out)?;
    fs::write(Path::new(out).join("blind.json"), bytes)?;
    fs::write(
        Path::new(out).join("resolved-requested.json"),
        serde_json::to_vec(&resolved)?,
    )?;
    Ok(())
}

pub(super) fn verify_inputs(
    model: &IntegerDiscreteModel,
    prompts: &[BlindPrompt],
    blind_hash: &str,
    public: &Value,
    resolved: &Resolved,
) -> Result<()> {
    need(
        public["schema"] == SELECT_SCHEMA
            && public["arm"] == "public"
            && public["blind"] == blind_hash
            && public["artifact"] == model.artifact_sha256()
            && public["bounds"].is_null()
            && resolved.schema == RESOLVED_SCHEMA
            && resolved.blind_sha256 == blind_hash
            && model.tokenizer_sha256() == Some(&resolved.tokenizer_sha256)
            && rows(public)?.len() == prompts.len()
            && resolved.rows.len() == prompts.len(),
        "public selector or independently supplied endpoints do not bind this full prompt tape",
    )?;
    for (i, ((p, s), r)) in prompts
        .iter()
        .zip(rows(public)?)
        .zip(&resolved.rows)
        .enumerate()
    {
        let key = prefix_sha(&p.prefix);
        need(
            s["index"] == i
                && r.index == i
                && s["prefix_sha256"] == key
                && r.prefix_sha256 == key
                && get::<[usize; 2]>(s, "region")? == [0, p.prefix.len()]
                && s["paired"]["role_words"] == s["original_role_words"],
            "requested diagnostic attempted to alter public role words or bounds",
        )?;
    }
    Ok(())
}

fn role_decision(ids: &[u32], words: [u32; 8]) -> Result<PairedRoleDecision> {
    let span = |w| -> Result<Option<RoleSpan>> {
        Ok(pair(w, ids)?.map(|[start, end]| RoleSpan { start, end }))
    };
    Ok(PairedRoleDecision {
        requested: span([words[0], words[1]])?,
        excluded: span([words[2], words[3]])?,
        requested_score: words[4] as i32,
        excluded_score: words[5] as i32,
        requested_runner_up: words[6] as i32,
        excluded_runner_up: words[7] as i32,
    })
}

pub(super) fn expected_roles(old: [u32; 8], requested: Option<[usize; 2]>) -> Result<[u32; 8]> {
    let mut new = old;
    (new[0], new[1]) = match requested {
        Some([a, b]) => (u32::try_from(a)?, u32::try_from(b)?),
        None => (NIL, NIL),
    };
    Ok(new)
}

pub fn requested(args: &[String]) -> Result<()> {
    let [model, blind, public, resolved, resolved_sha, out] = args else {
        return Err("requested PAIRED_D11 BLIND_JSON PUBLIC_SELECTION RESOLVED_REQUESTED EXPECTED_SHA OUT_JSON".into());
    };
    let model = IntegerDiscreteModel::load(Path::new(model))?;
    let mut hashes = BTreeMap::new();
    let (blind, public) = (Path::new(blind), Path::new(public));
    let prompts: Vec<BlindPrompt> = load(blind, &mut hashes)?;
    let original: Value = load(public, &mut hashes)?;
    let sealed: Resolved = serde_json::from_slice(&pinned(Path::new(resolved), resolved_sha)?)?;
    let blind_hash = digest(&hashes, blind)?;
    verify_inputs(&model, &prompts, blind_hash, &original, &sealed)?;
    let mut output = Vec::with_capacity(prompts.len());
    for ((p, old), r) in prompts.iter().zip(rows(&original)?).zip(&sealed.rows) {
        let original_words = get(old, "original_role_words")?;
        let mut roles = role_decision(&p.prefix, original_words)?;
        let requested = r.requested_span.map(|[start, end]| RoleSpan { start, end });
        // Clearing the requested span must not clear exclusion or role scores.
        if requested.is_none() {
            roles.requested = None;
        }
        let (paired, assertions) = model.paired_copy_witness(&p.prefix, roles, requested)?;
        need(
            paired.role_words == Some(expected_roles(original_words, r.requested_span)?),
            "frozen assertion operator changed another role word",
        )?;
        let mut row = old.clone();
        row["paired"] = serde_json::to_value(paired)?;
        row["assertions"] = serde_json::to_value(assertions)?;
        output.push(row);
    }
    let result = json!({"schema":SELECT_SCHEMA,"arm":"requested-diagnostic",
        "artifact":model.artifact_sha256(),"bounds":null,"blind":blind_hash,
        "public_selection_sha256":digest(&hashes,public)?,
        "resolved_request_sha256":resolved_sha,"rows":output});
    use std::io::Write;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out)?;
    f.write_all(&serde_json::to_vec(&result)?)?;
    f.sync_all()?;
    Ok(())
}

pub(super) fn read_resolved(path: &Path, expected: &str) -> Result<Resolved> {
    Ok(serde_json::from_slice(&pinned(path, expected)?)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sealed_name_schema_cannot_carry_item_labels_source_or_bounds() {
        let endpoint = json!({"schema":RAW_SCHEMA,"raw_sha256":"digest","rows":[
          {"index":0,"text_sha256":"row","requested":{"bytes":[4,10],"name":"Monroe"}}]});
        assert!(serde_json::from_value::<RawSeal>(endpoint.clone()).is_ok());
        for (level, field) in [
            ("top", "grades"),
            ("row", "expected"),
            ("row", "bounds"),
            ("request", "gold_start"),
            ("request", "answer"),
        ] {
            let mut filthy = endpoint.clone();
            let object = match level {
                "top" => &mut filthy,
                "row" => &mut filthy["rows"][0],
                _ => &mut filthy["rows"][0]["requested"],
            };
            object[field] = json!(true);
            assert!(serde_json::from_value::<RawSeal>(filthy).is_err());
        }
    }

    #[test]
    fn request_replacement_preserves_six_public_words_including_signed_scores() {
        let original = [4, 6, 8, 10, 93, (-41i32) as u32, 47, 22];
        let changed = expected_roles(original, Some([16, 18])).unwrap();
        assert_eq!(changed[..2], [16, 18]);
        assert_eq!(changed[2..], original[2..]);
        let absent = expected_roles(original, None).unwrap();
        assert_eq!(absent[..2], [NIL, NIL]);
        assert_eq!(absent[2..], original[2..]);
    }
}
