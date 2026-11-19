use super::{hash, load, save, Error};
use serde_json::{json, Value};
use std::{collections::BTreeMap, path::Path};
fn indexed<'a>(v: &'a Value, key: &str) -> Result<&'a Vec<Value>, Error> {
    Ok(v[key]
        .as_array()
        .ok_or_else(|| format!("missing array {key}"))?)
}
fn span(words: &Value) -> Option<[u64; 2]> {
    let s = words.get(0)?.as_u64()?;
    let e = words.get(1)?.as_u64()?;
    (s != u32::MAX as u64).then_some([s, e])
}
pub fn grade(args: &[String]) -> Result<(), Error> {
    let [selection, run, grades, public, out] = args else {
        return Err("grade SELECTION CHAIN_DIR GRADES PUBLIC_DIR|- OUT".into());
    };
    let run = Path::new(run);
    let selection_sha = hash(Path::new(selection))?;
    let picks = load(Path::new(selection))?;
    let labels = load(Path::new(grades))?;
    let receipts = load(&run.join("receipt.json"))?;
    let gpu_path = run.join("nominees/chosen.json");
    if receipts["gpu"] != hash(&gpu_path)? || receipts["blind"] != picks["blind"] {
        return Err("unbound trajectory".into());
    }
    let g = load(&gpu_path)?;
    let trajectories = indexed(&g, "rows")?;
    let arms = indexed(&receipts, "arms")?;
    let maps = indexed(
        arms.iter()
            .find(|a| a["selection_sha256"] == selection_sha)
            .ok_or("selection not run")?,
        "rows",
    )?;
    let sel = indexed(&picks, "rows")?;
    let mut rows = Vec::new();
    let mut histogram: BTreeMap<String, u32> = BTreeMap::new();
    let pubseal = if public != "-" {
        Some(load(&Path::new(public).join("seal.json"))?)
    } else {
        None
    };
    if let Some(seal) = &pubseal {
        if seal["blind"] != picks["blind"]
            || seal["model"] != picks["artifact"]
            || seal["row_sha256"].as_array().map(Vec::len) != Some(sel.len())
        {
            return Err("public seal differs from selections".into());
        }
    }
    if maps.len() != sel.len() {
        return Err("unpaired role and writer traces".into());
    }
    for (mapping, s) in maps.iter().zip(sel) {
        let [i, j]: [usize; 2] = serde_json::from_value(mapping.clone())?;
        let t = trajectories.get(j).ok_or("invalid replay row")?;
        let label = &labels[i];
        if s["index"] != i
            || t["prefix_sha256"] != s["prefix_sha256"]
            || s["prefix_sha256"] != label["prefix_sha256"]
            || t["source"] != s["paired"]["choice"]["source"]
        {
            return Err("row identity or source mismatch".into());
        }
        let ids: Vec<u32> = serde_json::from_value(t["generated"].clone())?;
        let mut text = None;
        if let Some(seal) = &pubseal {
            let path = Path::new(public).join(format!("row-{i:04}.json"));
            if hash(&path)? != seal["row_sha256"][i] {
                return Err("public receipt digest failed".into());
            }
            let p = load(&path)?;
            let mut words = Vec::new();
            for step in indexed(t, "steps")? {
                let mut w: Vec<u32> = serde_json::from_value(step["after"].clone())?;
                for key in ["emitted", "source_id", "copied", "release"] {
                    w.push(match key {
                        "source_id" => step[key].as_u64().map_or(u32::MAX, |x| x as u32),
                        _ => serde_json::from_value(step[key].clone())?,
                    });
                }
                words.push(w);
            }
            if p["generated_ids"] != t["generated"]
                || p["paired"] != s["paired"]
                || p["copy_step_words"] != json!(words)
            {
                return Err(format!("public/device divergence at row {i}").into());
            }
            text = Some(p["text"].clone());
        }
        let want: Option<Vec<u32>> = serde_json::from_value(label["expected"].clone())?;
        let requested = span(&s["paired"]["role_words"]);
        let role_correct = json!(requested) == label["requested"];
        let first = want
            .as_ref()
            .and_then(|w| (0..w.len().max(ids.len())).find(|&k| w.get(k) != ids.get(k)));
        let exact = want.as_ref().map(|_| first.is_none());
        let correct_source = label["facts"]
            .as_array()
            .ok_or("missing facts")?
            .iter()
            .any(|f| f["name"] == label["requested_name"] && f["value_span"][0] == t["source"]);
        if want.is_some() {
            *histogram
                .entry(first.map_or_else(|| "exact_eos".into(), |x| x.to_string()))
                .or_default() += 1;
        }
        rows.push(json!({"index":i,"id":label["id"],"family":label["family"],"pair":label["pair"],"direction":label["direction"],"stratum":label["stratum"],"counterpart":label["counterpart"],"requested":requested,"gold_requested":label["requested"],"endpoint_exact":role_correct,"excluded":s["paired"]["role_words"].as_array().and_then(|v|if v[2]!=u32::MAX{Some([v[2].clone(),v[3].clone()])}else{None}),
            "source":t["source"],"source_matches_requested_value":correct_source,"supporting_facts":s["paired"]["supporting_facts"],"complete_values":s["paired"]["complete_values"],"values_disagree":s["paired"]["values_disagree"],
            "expected":want,"generated":ids,"text":text,"acceptable_prose":label["prose_acceptable"],"terminal_eos":ids.last()==Some(&1),"exact":exact,"first_wrong":first,"expected_wrong_id":first.and_then(|k|want.as_ref()?.get(k)),"actual_wrong_id":first.and_then(|k|ids.get(k)),"first_free":t["first_free"]}));
    }
    save(
        Path::new(out),
        &json!({"schema":"uor-r4.ordered-role.grades/1","selection":selection_sha,"gpu":receipts["gpu"],"grades":hash(Path::new(grades))?,"first_wrong":histogram,"rows":rows}),
    )
}
