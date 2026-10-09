//! Input-only paired earliest-query intervention. No model or prediction access.
use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EarlyQueryConfig {
    inputs: PathBuf,
    expected_inputs_sha256: String,
    labels: PathBuf,
    expected_labels_sha256: String,
    tokenizer: PathBuf,
    expected_tokenizer_sha256: String,
    original_row_indices: [usize; 2],
    query_strings: [String; 2],
    output: PathBuf,
}

fn pair_contract(a: &Packet, b: &Packet, queries: &[Vec<u32>; 2], targets: [u32; 2]) -> Result<()> {
    if serde_json::to_value(&a.segments)? != serde_json::to_value(&b.segments)? {
        return Err(bad("paired Source/Context segments differ"));
    }
    if !a.actual_prefix_ids.is_empty() || !b.actual_prefix_ids.is_empty() {
        return Err(bad(
            "early-query intervention requires empty actual prefixes",
        ));
    }
    if queries[0].is_empty()
        || queries[0].len() != queries[1].len()
        || queries[0][0] == queries[1][0]
        || queries[0][1..] != queries[1][1..]
    {
        return Err(bad(
            "queries must differ solely at token index zero with identical length/suffix",
        ));
    }
    if targets[0] == targets[1] {
        return Err(bad(
            "paired accepted answers must have distinct first tokens",
        ));
    }
    Ok(())
}

pub(super) fn prepare(raw: &[u8]) -> Result<()> {
    let mut c: EarlyQueryConfig = serde_json::from_slice(raw)?;
    for path in [&c.inputs, &c.labels, &c.tokenizer, &c.output] {
        if !path.is_absolute()
            || path
                .components()
                .any(|p| matches!(p, std::path::Component::ParentDir))
        {
            return Err(bad("absolute nontraversing early-query paths required"));
        }
    }
    if c.original_row_indices[0] == c.original_row_indices[1]
        || c.query_strings.iter().any(String::is_empty)
        || [
            &c.expected_inputs_sha256,
            &c.expected_labels_sha256,
            &c.expected_tokenizer_sha256,
        ]
        .iter()
        .any(|s| !hex_identity(s, 64))
    {
        return Err(bad(
            "distinct row indices, nonempty query strings and exact SHA256 pins required",
        ));
    }
    c.output = output_support::prospective_output(&c.output)?;
    for path in [&mut c.inputs, &mut c.labels, &mut c.tokenizer] {
        *path = fs::canonicalize(&*path)?;
        if path.starts_with(&c.output) || c.output.starts_with(&*path) {
            return Err(bad("early-query output/input ancestry overlaps"));
        }
    }
    let input_root = seal_for(&c.inputs)?;
    let label_root = seal_for(&c.labels)?;
    let tokenizer_root = c
        .tokenizer
        .ancestors()
        .find(|p| p.join("manifest.json").is_file())
        .map(Path::to_path_buf);
    for root in [&input_root, &label_root]
        .into_iter()
        .chain(tokenizer_root.iter())
    {
        if c.output.starts_with(root) || root.starts_with(&c.output) {
            return Err(bad("early-query output overlaps sealed input ancestry"));
        }
    }
    // Path validation above is read-only; claim precedes input content access.
    report_output::claim(&c.output)?;
    let result = (|| -> Result<Value> {
        write(&c.output.join("config.json"), &serde_json::from_slice(raw)?)?;
        for root in [&input_root, &label_root]
            .into_iter()
            .chain(tokenizer_root.iter())
        {
            report_output::verify(root)?;
        }
        for path in [&c.inputs, &c.labels, &c.tokenizer] {
            if !fs::metadata(path)?.is_file() || fs::metadata(path)?.len() > 64 * 1024 * 1024 {
                return Err(bad(
                    "early-query inputs must be regular files of at most 64MiB",
                ));
            }
        }
        if file_hash(&c.inputs)? != c.expected_inputs_sha256
            || file_hash(&c.labels)? != c.expected_labels_sha256
            || file_hash(&c.tokenizer)? != c.expected_tokenizer_sha256
        {
            return Err(bad(
                "early-query pinned input/label/tokenizer identity differs",
            ));
        }
        let panel: Panel = serde_json::from_slice(&bytes(&c.inputs)?)?;
        let labels: Labels = serde_json::from_slice(&bytes(&c.labels)?)?;
        if panel.schema != "uor-r4.native-source-bank-probe-input/1"
            || labels.schema != "uor-r4.native-source-bank-labels/1"
            || labels.protocol != "uor-r4.literal-role-dialogue/2"
            || !labels.membership_only
            || panel.cases.len() != labels.cases.len()
        {
            return Err(bad("early-query original panel/label schema differs"));
        }
        let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes(&c.tokenizer)?)
            .ok_or_else(|| bad("early-query ByteBPE unavailable"))?;
        let queries = c.query_strings.clone().map(|s| tok.encode(&s));
        for (text, ids) in c.query_strings.iter().zip(&queries) {
            if tok.decode_bytes(ids) != text.as_bytes() {
                return Err(bad("intervention query does not roundtrip tokenizer"));
            }
        }
        let mut packets = Vec::new();
        let mut label_rows = Vec::new();
        let mut authorities = Vec::new();
        let mut selected = Vec::new();
        let mut target_first = Vec::new();
        for &index in &c.original_row_indices {
            let p = panel
                .cases
                .get(index)
                .ok_or_else(|| bad("original input row index out of range"))?;
            let l = labels
                .cases
                .get(index)
                .ok_or_else(|| bad("original label row index out of range"))?;
            if p.id.is_empty() || p.id != l.id {
                return Err(bad("original selected input/label ID differs"));
            }
            if !p
                .segments
                .iter()
                .any(|s| matches!(s, Segment::Source { .. }))
            {
                return Err(bad("selected original packet has no Source record"));
            }
            l.answers.validate()?;
            let mut first = None;
            for answer in &l.answers.accepted {
                let ids = tok.encode(answer);
                if tok.decode_bytes(&ids) != answer.as_bytes() {
                    return Err(bad("original answer does not roundtrip tokenizer"));
                }
                let token = ids
                    .first()
                    .copied()
                    .ok_or_else(|| bad("accepted answer empty"))?;
                if first.is_some_and(|old| old != token) {
                    return Err(bad("accepted answer first-token membership is ambiguous"));
                }
                first = Some(token);
            }
            target_first.push(first.ok_or_else(|| bad("accepted answer membership empty"))?);
            selected.push(p);
        }
        pair_contract(
            selected[0],
            selected[1],
            &queries,
            [target_first[0], target_first[1]],
        )?;
        if selected[0].id == selected[1].id {
            return Err(bad("selected original IDs duplicate"));
        }
        for intervention in [false, true] {
            for (arm, &index) in c.original_row_indices.iter().enumerate() {
                let p = selected[arm];
                let l = &labels.cases[index];
                let mut packet = p.clone();
                packet.id = format!(
                    "{}-{}",
                    if intervention {
                        "early-query"
                    } else {
                        "original-query"
                    },
                    p.id
                );
                if intervention {
                    packet.query_ids = queries[arm].clone();
                }
                let old_bytes = tok.decode_bytes(&p.query_ids);
                let old_text = std::str::from_utf8(&old_bytes)
                    .map_err(|_| bad("original query is not UTF8"))?;
                if tok.encode(old_text) != p.query_ids {
                    return Err(bad(
                        "original query tokenization does not roundtrip canonically",
                    ));
                }
                authorities.push(json!({"id":packet.id,"original_input_id":p.id,"original_row_index":index,
                    "intervention":intervention,"original_query_ids":p.query_ids,"original_query_text":old_text,
                    "query_ids":packet.query_ids,"query_text":if intervention { &c.query_strings[arm] } else { old_text },
                    "target_first_token_id":target_first[arm]}));
                label_rows.push(json!({"id":packet.id,"answers":l.answers,
                    "pair_id":if intervention { "early-query-intervention" } else { "original-query-controls" }}));
                packets.push(packet);
            }
        }
        if serde_json::to_vec(&packets)?.len()
            + serde_json::to_vec(&label_rows)?.len()
            + serde_json::to_vec(&authorities)?.len()
            > 1024 * 1024
        {
            return Err(bad(
                "early-query compact output payload exceeds 1MiB reserve",
            ));
        }
        write(
            &c.output.join("inputs.json"),
            &json!({"schema":panel.schema,"cases":packets}),
        )?;
        write(
            &c.output.join("labels.json"),
            &json!({"schema":labels.schema,"protocol":labels.protocol,"membership_only":true,"cases":label_rows}),
        )?;
        Ok(
            json!({"schema":"uor-r4.early-query-preparation/1","status":"COMPLETED",
            "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":file_hash(&std::env::current_exe()?)?,
            "config_sha256":file_hash(&c.output.join("config.json"))?,"original_inputs_sha256":c.expected_inputs_sha256,
            "original_labels_sha256":c.expected_labels_sha256,"tokenizer_sha256":c.expected_tokenizer_sha256,
            "input_manifest_sha256":file_hash(&input_root.join("manifest.json"))?,"label_manifest_sha256":file_hash(&label_root.join("manifest.json"))?,
            "inputs_sha256":file_hash(&c.output.join("inputs.json"))?,"labels_sha256":file_hash(&c.output.join("labels.json"))?,
            "rows":authorities,"changed_token_index":0,"changed_token_ids":[queries[0][0],queries[1][0]],
            "fixed_query_suffix_ids":&queries[0][1..],"fixed_source_context_segments":true,"actual_prefix_ids":[],
            "model_calls":0,"scope":"novel input-only diagnostic; original answer memberships cloned, no model/prediction access or capability result"}),
        )
    })();
    let report = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.early-query-preparation/1","status":"FAILED","error":e.to_string(),"model_calls":0})
        }
    };
    write(&c.output.join("report.json"), &report)?;
    report_output::seal(&c.output)?;
    report_output::verify(&c.output)?;
    result.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn packet() -> Packet {
        Packet {
            id: "fixture".into(),
            segments: vec![Segment::Context {
                event: 1,
                role: 0,
                token_ids: vec![3],
            }],
            query_ids: vec![1],
            actual_prefix_ids: vec![],
        }
    }
    #[test]
    fn exact_early_query_pair_accepts_and_rejects_suffix_and_length() -> Result<()> {
        let a = packet();
        let b = packet();
        pair_contract(&a, &b, &[vec![4, 6, 7], vec![5, 6, 7]], [8, 9])?;
        for q in [
            [vec![4, 6, 7], vec![5, 6, 8]],
            [vec![4, 6], vec![5, 6, 7]],
            [vec![4], vec![4]],
            [vec![], vec![]],
        ] {
            assert!(pair_contract(&a, &b, &q, [8, 9]).is_err());
        }
        assert!(pair_contract(&a, &b, &[vec![4], vec![5]], [8, 8]).is_err());
        Ok(())
    }
    #[test]
    fn exact_early_query_pair_rejects_source_context_and_prefix_changes() -> Result<()> {
        let a = packet();
        let mut b = packet();
        b.segments.push(Segment::Source {
            event: 2,
            record: 1,
            commit: 1,
            scope: "s".into(),
            entity: vec![1],
            relation: 0,
            view: 0,
            original_source_ids: vec![2],
        });
        assert!(pair_contract(&a, &b, &[vec![4], vec![5]], [8, 9]).is_err());
        let mut b = packet();
        b.actual_prefix_ids.push(2);
        assert!(pair_contract(&a, &b, &[vec![4], vec![5]], [8, 9]).is_err());
        Ok(())
    }
}
