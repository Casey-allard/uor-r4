#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path, vec::Vec};
use uor_all_starts_finite::Candidates;
use uor_r4_integer::{
    assertion_value::AssertionValue,
    discrete_d11::{ExactCopyState, IntegerDiscreteModel, ROLE_COPY_STOPS},
    paired_role::{ROLE_CONTEXT, ROLE_VOCAB},
    span_selector::{Bank, SpanChoice, CONT, START},
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const WRITER: &str = "004cc78a1d0b616c15986c2032b6c07f661536851c4d24a627d6eb6d4258e592";
const NIL: u32 = u32::MAX;

fn need(ok: bool, error: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(error.into())
    }
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn prefix_sha(prefix: &[u32]) -> String {
    let mut digest = Sha256::new();
    for id in prefix {
        digest.update(id.to_le_bytes());
    }
    format!("{:x}", digest.finalize())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BlindPrompt {
    prefix: Vec<u32>,
    #[serde(rename = "text")]
    _text: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BarePrompt {
    prefix: Vec<u32>,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Witness {
    fact_span: [usize; 2],
    assertion: AssertionValue,
    completed_source_range: Option<[u32; 2]>,
}

#[derive(Deserialize)]
struct SpanState {
    artifact_sha256: String,
    tokenizer_sha256: String,
    bank: Bank,
}

fn load<T: for<'de> Deserialize<'de>>(
    path: &Path,
    hashes: &mut BTreeMap<String, String>,
) -> Result<T> {
    let bytes = fs::read(path)?;
    hashes.insert(path.display().to_string(), sha(&bytes));
    serde_json::from_slice(&bytes).map_err(|err| format!("{}: {err}", path.display()).into())
}

fn rows(value: &Value) -> Result<&[Value]> {
    value["rows"]
        .as_array()
        .map(Vec::as_slice)
        .ok_or("missing rows".into())
}

fn get<T: for<'de> Deserialize<'de>>(value: &Value, key: &str) -> Result<T> {
    serde_json::from_value(
        value
            .get(key)
            .ok_or_else(|| format!("missing {key}"))?
            .clone(),
    )
    .map_err(|err| format!("{key}: {err}").into())
}

fn digest<'a>(hashes: &'a BTreeMap<String, String>, path: &Path) -> Result<&'a str> {
    hashes
        .get(&path.display().to_string())
        .map(String::as_str)
        .ok_or("unhashed file".into())
}

type Index = BTreeMap<(String, Option<[usize; 2]>), FixtureGroup>;

struct FixtureGroup {
    sites: Vec<usize>,
    fact_spans: Vec<[usize; 2]>,
    ranks: Vec<AssertionValue>,
    selected: Option<u32>,
    selected_index: usize,
}

fn codebook_bytes(roots: &[u32]) -> Result<Vec<u8>> {
    need(roots.len() == ROLE_VOCAB, "wrong codebook size")?;
    let mut bytes = Vec::with_capacity(2 * ROLE_VOCAB);
    for &root in roots {
        bytes.extend(u16::try_from(root)?.to_le_bytes());
    }
    Ok(bytes)
}

fn group_index(
    root: &Path,
    model: &str,
    tokenizer: &str,
    span_hash: &str,
    hashes: &mut BTreeMap<String, String>,
) -> Result<Index> {
    let mut groups = BTreeMap::new();
    let mut dirs = fs::read_dir(root)?.collect::<std::io::Result<Vec<_>>>()?;
    dirs.sort_by_key(fs::DirEntry::file_name);
    for entry in dirs {
        if !entry.file_name().to_string_lossy().starts_with("level-") {
            continue;
        }
        need(entry.file_type()?.is_dir(), "invalid chain group")?;
        let dir = entry.path();
        let stage = dir.join("primary");
        let endpoint_files: Vec<_> = fs::read_dir(stage)?
            .collect::<std::io::Result<Vec<_>>>()?
            .into_iter()
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .ends_with("-panel-nominees.json")
            })
            .collect();
        need(endpoint_files.len() == 1, "wrong endpoint file count")?;
        let file = endpoint_files
            .first()
            .ok_or("missing endpoint file")?
            .path();
        let ep: Value = load(&file, hashes)?;
        let bare_path = dir.join("bare.json");
        let bare: Vec<BarePrompt> = load(&bare_path, hashes)?;
        let bare_hash = digest(hashes, &bare_path)?.to_owned();
        let n = dir.join("nominees");
        let nom_path = n.join("nomination.json");
        let nom: Value = load(&nom_path, hashes)?;
        let nom_hash = digest(hashes, &nom_path)?.to_owned();
        let sites: Value = load(&n.join("sites.json"), hashes)?;
        let selection: Value = load(&n.join("selection.json"), hashes)?;
        let sites_by_row = get::<Vec<Vec<usize>>>(&sites, "sites")?;
        let max = get::<usize>(&sites, "max")?;
        let ep_rows = rows(&ep)?;
        let nom_rows = rows(&nom)?;
        let sel_rows = rows(&selection)?;
        need(
            ep["schema"] == "uor-r4.paired-role-decisions/1"
                && ep["artifact_sha256"] == model
                && ep["tokenizer_sha256"] == tokenizer
                && ep["bare_sha256"] == bare_hash
                && ep["state_sha256"] == sha(&serde_json::to_vec(ep_rows)?)
                && sites["artifact"] == model
                && sites["bare"] == bare_hash
                && sites["nominee"] == nom_hash
                && sites["bank"] == nom["state_sha256"]
                && nom["artifact_sha256"] == model
                && nom["bare_sha256"] == bare_hash
                && selection["artifact_sha256"] == model
                && selection["bare_sha256"] == bare_hash
                && selection["nominee_sha256"] == nom_hash
                && [
                    ep_rows.len(),
                    nom_rows.len(),
                    sel_rows.len(),
                    sites_by_row.len(),
                ] == [bare.len(); 4]
                && max == sites_by_row.iter().map(Vec::len).max().unwrap_or_default()
                && max <= ROLE_CONTEXT,
            "unbound chain endpoint, nomination, or site frame",
        )?;
        let mut tapes = Vec::with_capacity(max);
        for rank in 0..max {
            let gpu: Value = load(&n.join(format!("rank-{rank}-cont.json")), hashes)?;
            need(
                gpu["schema"] == "uor-r4.entity-pair-chain/1"
                    && gpu["artifact_sha256"] == model
                    && gpu["bare_sha256"] == bare_hash
                    && gpu["span_state_sha256"] == span_hash
                    && gpu["cuda_parity_words"] == 13 * bare.len()
                    && rows(&gpu)?.len() == bare.len(),
                "unbound CUDA assertion tape",
            )?;
            tapes.push(gpu);
        }
        for (i, b) in bare.iter().enumerate() {
            let id = prefix_sha(&b.prefix);
            let e = &ep_rows[i];
            let query = get::<Option<usize>>(e, "query_start")?
                .zip(get::<Option<usize>>(e, "query_end")?)
                .map(|(a, b)| [a, b]);
            let query_words = query.map(|[a, b]| (Some(a), Some(b))).unwrap_or_default();
            let nr = &nom_rows[i];
            let sr = &sel_rows[i];
            let found = &sites_by_row[i];
            need(
                e["prefix_sha256"] == id
                    && nr["prefix_sha256"] == id
                    && sr["prefix_sha256"] == id
                    && (
                        get::<Option<usize>>(e, "query_start")?,
                        get::<Option<usize>>(e, "query_end")?,
                    ) == query_words
                    && (
                        get::<Option<usize>>(nr, "nominee")?,
                        get::<Option<usize>>(nr, "query_end")?,
                    ) == query_words
                    && get::<Option<usize>>(sr, "nominee")? == query.map(|q| q[0])
                    && nr["count"] == found.len()
                    && sr["count"] == found.len()
                    && found.windows(2).all(|p| p[0] < p[1])
                    && found.iter().all(|&p| p < b.prefix.len())
                    && (query.is_some() || found.is_empty()),
                "inconsistent row order or site nomination",
            )?;
            let mut ranks = Vec::with_capacity(found.len());
            let mut fact_spans = Vec::with_capacity(found.len());
            for (rank, tape) in tapes.iter().enumerate() {
                let row = &rows(tape)?[i];
                let frozen: SpanChoice = get(row, "frozen")?;
                let a: AssertionValue = get(row, "bounded")?;
                let words: [u32; 13] = get(row, "words")?;
                need(
                    row["prefix_sha256"] == id
                        && a.words() == words
                        && a.source == get(row, "source")?,
                    "wrong CUDA assertion words",
                )?;
                if let Some(&site) = found.get(rank) {
                    let [qs, qe] = query.ok_or("assertion without requested endpoint")?;
                    need(
                        qe > qs
                            && frozen.fact_start == Some(u32::try_from(site)?)
                            && frozen.query_start == Some(u32::try_from(qs)?)
                            && frozen.query_end == Some(u32::try_from(qe - 1)?),
                        "stale rank coordinates",
                    )?;
                    let fact_end = frozen.fact_end.ok_or("missing GPU fact end")? as usize;
                    need(site <= fact_end && fact_end < qs, "invalid GPU fact span")?;
                    fact_spans.push([site, fact_end + 1]);
                    ranks.push(a);
                } else {
                    need(
                        a == AssertionValue::default() && frozen == SpanChoice::default(),
                        "unoccupied assertion rank carries a proposal",
                    )?;
                }
            }
            let old = groups.insert(
                (id, query),
                FixtureGroup {
                    sites: found.clone(),
                    fact_spans,
                    ranks,
                    selected: get(sr, "source")?,
                    selected_index: i,
                },
            );
            need(old.is_none(), "duplicate endpoint key across chunks")?;
        }
    }
    need(!groups.is_empty(), "no chain groups")?;
    Ok(groups)
}

fn pair(words: [u32; 2], prefix: &[u32]) -> Result<Option<[usize; 2]>> {
    if words == [NIL; 2] {
        return Ok(None);
    }
    let [a, b] = words.map(|x| x as usize);
    need(
        a > 0 && a < b && b <= prefix.len(),
        "invalid whole-word span",
    )?;
    Ok(Some([a, b]))
}

fn package_assertion(
    ids: &[u32],
    requested: [usize; 2],
    witness: Witness,
    words: &[u32],
    roots: &[u32],
) -> Result<Value> {
    let [qs, qe] = requested;
    let [fs, fe] = witness.fact_span;
    need(
        fs > 0
            && fs < fe
            && fe < qs
            && qs < qe
            && qe <= ids.len()
            && ids[fs..fe] == ids[qs..qe]
            && words[ids[fs] as usize] == START
            && ids[fs + 1..fe].iter().all(|&id| words[id as usize] == CONT)
            && words[ids[fe] as usize] != CONT,
        "wrong or stale fact span",
    )?;
    let a = witness.assertion;
    let inv = Candidates::capture(ids, words, u32::try_from(fe - 1)?, u32::try_from(qs)?, a)
        .map_err(|e| format!("source inventory refused: {e:?}"))?;
    let mut legacy = Vec::new();
    let mut separate = Vec::new();
    for (depth, proposal) in inv.legacy.iter().copied().enumerate() {
        let Some(proposal) = proposal else {
            continue;
        };
        let src = proposal.source as usize;
        let fence = a.fence.ok_or("legacy source missing fence")? as usize;
        let end = (src + 1..fence)
            .find(|&i| words[ids[i] as usize] != CONT)
            .unwrap_or(fence);
        let mut packed = Vec::new();
        for &id in &ids[fe..end] {
            packed.push(u16::try_from(roots[id as usize])?);
        }
        let roots_le: Vec<_> = packed.iter().flat_map(|x| x.to_le_bytes()).collect();
        let sig: Vec<_> = ids[fe..end]
            .iter()
            .map(|&id| words[id as usize] as u8)
            .collect();
        legacy.push(json!({"original_start":src,"first_word_end":end,"connector_slots":depth+1,
            "word_slots":end-src,"writer_reachable":proposal.writer_reachable,
            "slot_ids":&ids[fe..end],"slot_classes":sig,"packed_roots_u16":packed,"packed_roots_le":roots_le}));
    }
    for p in inv.reachable() {
        if inv.legacy.iter().flatten().any(|l| l.source == p.source) {
            continue;
        }
        separate.push(
            json!({"original_start":p.source,"first_word_end":p.first_word_end,
            "connector_slots":p.connector_ids}),
        );
    }
    let completion = a.source.zip(a.fence).map(|(start, fence)| [start, fence]);
    if let Some([start, fence]) = completion {
        need(
            start < fence
                && fence - start <= 7
                && start as usize >= fe
                && fence as usize <= ids.len()
                && witness.completed_source_range == Some([start, fence]),
            "invalid archived completed range",
        )?;
    } else {
        need(
            witness.completed_source_range.is_none(),
            "unbound archived completion",
        )?;
    }
    Ok(
        json!({"fact_span":witness.fact_span,"fence":a.fence,"eligible_mask":a.value.eligible_mask,
        "word_start_mask":a.value.word_start_mask,"legacy_joint_mask":inv.legacy_mask,
        "legacy_candidates":legacy,"outside_legacy_inventory":separate}),
    )
}

fn cursor_words(ids: &[u32], trace: &Value, source: Option<u32>) -> Result<Vec<[u32; 8]>> {
    let generated: Vec<u32> = get(trace, "generated")?;
    let steps: Vec<Value> = get(trace, "steps")?;
    need(
        !generated.is_empty()
            && generated.len() <= 16
            && generated.len() == steps.len()
            && !generated[..generated.len() - 1].contains(&1)
            && source == get(trace, "source")?,
        "invalid archived writer shape",
    )?;
    let mut cursor = ExactCopyState::new(source, u32::try_from(ids.len())?);
    let mut prefix = ids.to_vec();
    let mut result = Vec::with_capacity(steps.len());
    for (emitted, step) in generated.into_iter().zip(steps) {
        let fallback: u32 = get(&step, "native_id")?;
        let before: [u32; 4] = get(&step, "before")?;
        let after: [u32; 4] = get(&step, "after")?;
        let native_top8: [u32; 8] = get(&step, "native_top8")?;
        let w = cursor
            .write(&prefix, fallback, ROLE_COPY_STOPS)?
            .finish_with_eos(true, 1);
        let archived_id: Option<u32> = get(&step, "source_id")?;
        need(
            before == cursor.words()
                && after == w.state.words()
                && emitted == w.emitted
                && step["emitted"] == emitted
                && step["copied"] == w.copied
                && step["release"] == w.release
                && archived_id == (w.exact_id != NIL).then_some(w.exact_id)
                && native_top8[0] == fallback
                && fallback < ROLE_VOCAB as u32,
            "archived frozen cursor replay differs",
        )?;
        result.push(w.words());
        cursor = w.state;
        prefix.push(emitted);
    }
    Ok(result)
}

fn bind_assertions(
    ids: &[u32],
    requested: Option<[usize; 2]>,
    row: &Value,
    group: &FixtureGroup,
    classes: &[u32],
    roots: &[u32],
) -> Result<Vec<Value>> {
    let pairs = &row["paired"];
    let witnesses: Vec<Witness> = get(row, "assertions")?;
    let source: Option<u32> = get(&pairs["choice"], "source")?;
    need(
        witnesses.len() == group.sites.len()
            && witnesses.len() == group.fact_spans.len()
            && witnesses.len() == group.ranks.len()
            && pairs["supporting_facts"] == witnesses.len()
            && source == group.selected
            && get::<Option<usize>>(&pairs["choice"], "query")? == requested.map(|q| q[0]),
        "raw role, site, or selected-source disagreement",
    )?;
    let mut complete = 0;
    let mut disagreement = false;
    let mut first: Option<(usize, [u32; 2])> = None;
    let mut included = Vec::with_capacity(witnesses.len());
    for (slot, witness) in witnesses.into_iter().enumerate() {
        need(
            witness.fact_span == group.fact_spans[slot] && witness.assertion == group.ranks[slot],
            "retained fact or fence differs from CUDA trace",
        )?;
        let q = requested.ok_or("retained assertion without requested endpoint")?;
        included.push(package_assertion(ids, q, witness, classes, roots)?);
        if let Some(range) = witness.completed_source_range {
            complete += 1;
            let value = |[l, r]: [u32; 2]| &ids[l as usize..r as usize];
            if let Some((_, observed)) = first {
                disagreement |= value(range) != value(observed);
            } else {
                first = Some((witness.fact_span[0], range));
            }
        }
    }
    let expected = if complete == included.len() && !disagreement {
        first
    } else {
        None
    };
    need(
        pairs["complete_values"] == complete
            && pairs["values_disagree"] == disagreement
            && get::<Option<usize>>(&pairs["choice"], "fact")? == expected.map(|p| p.0)
            && source == expected.map(|p| p.1[0]),
        "archived consensus does not bind completed assertions",
    )?;
    Ok(included)
}

pub fn capture(args: &[String]) -> Result<()> {
    let [base, paired, span, blind, select, chain, out, diagnostics @ ..] = args else {
        return Err("capture BASE_D11 PAIRED_D11 FROZEN_SPAN_STATE BLIND_JSON SELECTION CHAIN_DIR NEW_DIR [PUBLIC_SELECTION RESOLVED_REQUESTED EXPECTED_SHA]".into());
    };
    let (span, blind, select, chain, out) = (
        Path::new(span),
        Path::new(blind),
        Path::new(select),
        Path::new(chain),
        Path::new(out),
    );
    let base_d11 = IntegerDiscreteModel::load(Path::new(base))?;
    let paired_d11 = IntegerDiscreteModel::load(Path::new(paired))?;
    let model = base_d11.artifact_sha256();
    let tokenizer = base_d11
        .tokenizer_sha256()
        .ok_or("D11 lacks bound tokenizer")?;
    let roots = base_d11.packed_codebook_roots();
    let root_bytes = codebook_bytes(&roots)?;
    need(
        paired_d11.packed_codebook_roots() == roots
            && paired_d11.tokenizer_sha256() == Some(tokenizer),
        "paired D11 codebook or tokenizer differs from frozen writer trunk",
    )?;
    let role_sites = paired_d11.ordered_role_sites()?;
    let mut hashes = BTreeMap::new();
    let spans: SpanState = load(span, &mut hashes)?;
    spans.bank.validate()?;
    need(
        spans.artifact_sha256 == model && spans.tokenizer_sha256 == tokenizer,
        "word-class table belongs to a different D11 or tokenizer",
    )?;
    let span_hash = digest(&hashes, span)?.to_owned();
    let prompts: Vec<BlindPrompt> = load(blind, &mut hashes)?;
    let blind_hash = digest(&hashes, blind)?.to_owned();
    let selections: Value = load(select, &mut hashes)?;
    let arm = selections["arm"].as_str().ok_or("missing selection arm")?;
    let enabled = match arm {
        "public" | "requested-diagnostic" => true,
        "no-read" => false,
        _ => return Err("unrecognized or grade-derived selection arm".into()),
    };
    let diagnostic = match (arm, diagnostics) {
        ("requested-diagnostic", [public, resolved, expected_sha]) => {
            let (public_path, resolved_path) = (Path::new(public), Path::new(resolved));
            let public: Value = load(public_path, &mut hashes)?;
            let parsed = requested::read_resolved(resolved_path, expected_sha)?;
            hashes.insert(resolved.clone(), expected_sha.clone());
            requested::verify_inputs(&paired_d11, &prompts, &blind_hash, &public, &parsed)?;
            need(
                selections["public_selection_sha256"] == digest(&hashes, public_path)?
                    && selections["resolved_request_sha256"] == *expected_sha,
                "diagnostic does not bind the untouched public selection and sealed endpoints",
            )?;
            Some((public, parsed))
        }
        ("requested-diagnostic", _) | (_, [_, ..]) => {
            return Err("wrong requested capture inputs".into())
        }
        (_, []) => None,
    };
    need(
        selections["schema"] == "uor-r4.ordered-role.selections/1"
            && selections["artifact"] == paired_d11.artifact_sha256()
            && selections["blind"] == blind_hash
            && selections["bounds"].is_null()
            && rows(&selections)?.len() == prompts.len()
            && !prompts.is_empty(),
        "invalid blind selection receipt",
    )?;
    let selection_hash = digest(&hashes, select)?.to_owned();
    let actual_path = chain.join("nominees/chosen.json");
    let chosen: Value = load(&actual_path, &mut hashes)?;
    let receipt: Value = load(&chain.join("receipt.json"), &mut hashes)?;
    let gpu_hash = digest(&hashes, &actual_path)?.to_owned();
    need(
        receipt["schema"] == "uor-r4.ordered-role.replay/1"
            && chosen["schema"] == "uor-r4.ordered-role.joined-trajectories/1"
            && receipt["blind"] == blind_hash
            && receipt["gpu"] == gpu_hash
            && receipt["model"] == model
            && chosen["model"] == model
            && receipt["writer"] == WRITER
            && receipt["unique"] == rows(&chosen)?.len()
            && receipt["transitions"] == chosen["transitions"],
        "invalid chosen-writer receipt",
    )?;
    let mut maps = receipt["arms"]
        .as_array()
        .ok_or("missing selection-to-writer maps")?
        .iter()
        .filter(|r| r["selection_sha256"] == selection_hash);
    let mapped: Vec<[usize; 2]> = get(
        maps.next().ok_or("selection has no writer receipt")?,
        "rows",
    )?;
    need(
        maps.next().is_none() && mapped.len() == prompts.len(),
        "ambiguous writer mapping",
    )?;
    let groups = group_index(chain, model, tokenizer, &span_hash, &mut hashes)?;
    let mut captures = Vec::with_capacity(prompts.len());
    let mut audits = Vec::with_capacity(prompts.len());
    for (i, ((prompt, row), mapping)) in prompts
        .iter()
        .zip(rows(&selections)?)
        .zip(mapped)
        .enumerate()
    {
        let ids = &prompt.prefix;
        let key = prefix_sha(ids);
        need(
            ids.first() == Some(&0)
                && ids.len() <= ROLE_CONTEXT
                && ids.iter().all(|&id| id < ROLE_VOCAB as u32)
                && row["index"] == i
                && mapping[0] == i
                && row["prefix_sha256"] == key
                && get::<[usize; 2]>(row, "region")? == [0, ids.len()],
            "invalid, stale or reordered original tape",
        )
        .map_err(|e| format!("row {i}: {e}"))?;
        let old: [u32; 8] = get(row, "original_role_words")?;
        let raw_pair: Option<[u32; 8]> = get(&row["paired"], "role_words")?;
        let expected = if let Some((public, resolved)) = &diagnostic {
            let untouched = &rows(public)?[i];
            let original = untouched.as_object().ok_or("wrong raw public row")?;
            let emitted = row.as_object().ok_or("wrong diagnostic row")?;
            need(
                original.len() == emitted.len()
                    && original.iter().all(|(key, value)| {
                        matches!(key.as_str(), "paired" | "assertions")
                            || emitted.get(key) == Some(value)
                    }),
                "diagnostic changed immutable public row fields",
            )?;
            Some(requested::expected_roles(
                old,
                resolved.rows[i].requested_span,
            )?)
        } else {
            enabled.then_some(old)
        };
        need(
            raw_pair == expected,
            "non-request role words changed, requested endpoint differs from seal, or disabled role remains",
        )?;
        let (q, e) = if let Some(effective) = raw_pair {
            (
                pair([effective[0], effective[1]], ids)?,
                pair([effective[2], effective[3]], ids)?,
            )
        } else {
            (None, None)
        };
        let content = match paired_d11.ordered_role_framing() {
            Some(framing) => framing.content(ids)?,
            None => 1..ids.len(),
        };
        let roles = role_sites.prompt(ids, content)?;
        for [s, t] in [q, e].into_iter().flatten() {
            need(
                roles
                    .candidates()
                    .any(|span| span.start == s && span.end == t),
                "role is not a complete original word",
            )?;
        }
        let group = groups
            .get(&(key.clone(), q))
            .ok_or("requested endpoint missing from frozen chain")?;
        let assertions = bind_assertions(ids, q, row, group, &spans.bank.classes, &roots)
            .map_err(|e| format!("row {i}: {e}"))?;
        let actual = rows(&chosen)?
            .get(mapping[1])
            .ok_or("writer row outside joined tape")?;
        need(
            actual["prefix_sha256"] == key,
            "chosen writer belongs to another original tape",
        )?;
        let writes =
            cursor_words(ids, actual, group.selected).map_err(|e| format!("row {i}: {e}"))?;
        captures.push(
            json!({"ordinal":i,"prefix_sha256":key,"original_tape_len":ids.len(),
            "requested_span":q,"excluded_span":e,"assertions":assertions}),
        );
        audits.push(
            json!({"ordinal":i,"prefix_sha256":key,"selected_source":group.selected,
            "writer_joined_ordinal":mapping[1],"endpoint_group_ordinal":group.selected_index,
            "role_words":raw_pair,"frozen_cursor_words":writes}),
        );
    }
    let class_bytes: Vec<_> = spans
        .bank
        .classes
        .iter()
        .map(|&class| class as u8)
        .collect();
    let mut captures = json!({"schema":"uor-r4.legacy-assertion-candidates/1",
        "base_d11":model,"role_d11":paired_d11.artifact_sha256(),
        "tokenizer_sha256":tokenizer,"codebook_sha256":sha(&root_bytes),
        "span_state_sha256":span_hash,"classes_sha256":sha(&class_bytes),
        "copy_stops":ROLE_COPY_STOPS,"blind_sha256":blind_hash,"selection_sha256":selection_hash,
        "arm":arm,"rows":captures});
    if diagnostic.is_some() {
        captures["requested_receipt"] = json!({
            "public_selection_sha256":selections["public_selection_sha256"],
            "resolved_request_sha256":selections["resolved_request_sha256"]
        });
    }
    let payload = serde_json::to_vec(&captures)?;
    let audit = json!({"schema":"uor-r4.legacy-assertion-capture-audit/1",
        "candidate_capture_sha256":sha(&payload),"writer_sha256":WRITER,
        "writer_tape_sha256":gpu_hash,"reads_sha256":hashes,"rows":audits});
    fs::create_dir(out)?;
    fs::write(out.join("candidates.json"), payload)?;
    fs::write(out.join("archive-audit.json"), serde_json::to_vec(&audit)?)?;
    fs::write(out.join("packed-codebook.u16le"), root_bytes)?;
    fs::write(out.join("legacy-word-classes.u8"), class_bytes)?;
    Ok(())
}

mod synthetic;

pub use synthetic::fixture as synthetic_fixture;
pub mod requested;
