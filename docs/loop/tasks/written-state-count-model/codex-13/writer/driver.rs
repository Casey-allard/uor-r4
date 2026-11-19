use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const WRITER: &str = "004cc78a1d0b616c15986c2032b6c07f661536851c4d24a627d6eb6d4258e592";
const BASE_CUBIN: &str = "b41d72ce289ff5daf29055687e2586929321fed6ddb800d10cddfc650e2eeb28";
const EOS_CUBIN: &str = "8a73fffb5edfe3f5b36b891b37a96ae5683aacde49d08ae69caab92a7cab9b85";
const EOS: u32 = 1;
const STOPS: [u32; 4] = [16, EOS, 0, 201];
const CURSOR_RELEASE_CAP: u32 = 8;
const CONSENSUS_VALUE_CAP: u32 = 7;
const CAP: usize = 16;
const X: u32 = u32::MAX;

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn file_hash(path: &Path) -> Result<String> {
    Ok(hash(&fs::read(path)?))
}
fn load<T: for<'a> Deserialize<'a>>(path: impl AsRef<Path>) -> Result<T> {
    let path = path.as_ref();
    serde_json::from_slice(&fs::read(path)?).map_err(|e| format!("{}: {e}", path.display()).into())
}
fn save(path: impl AsRef<Path>, data: &impl Serialize) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    serde_json::to_writer(&mut file, data)?;
    file.write_all(b"\n")?;
    Ok(())
}
fn run(command: &mut Command) -> Result<()> {
    eprintln!("{command:?}");
    if !command.status()?.success() {
        return Err(format!("command failed: {command:?}").into());
    }
    Ok(())
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Prompt {
    prefix: Vec<u32>,
}
impl Prompt {
    fn digest(&self) -> Result<String> {
        if self.prefix.first() != Some(&0) || self.prefix.last() == Some(&EOS) {
            return Err("missing BOS or prompt ends at EOS".into());
        }
        Ok(hash(
            &self
                .prefix
                .iter()
                .flat_map(|id| id.to_le_bytes())
                .collect::<Vec<_>>(),
        ))
    }
}
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
struct Step {
    before: [u32; 4],
    after: [u32; 4],
    copied: u32,
    emitted: u32,
    native_id: u32,
    native_top8: [u32; 8],
    release: u32,
    source_id: Option<u32>,
}
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
struct Row {
    prefix_sha256: String,
    source: Option<u32>,
    generated: Vec<u32>,
    steps: Vec<Step>,
    first_free: Option<(usize, u32)>,
}

// Verify the trace at original offsets and check all self-fed emissions and
// cursor words. CUDA's native prediction is already compared in action-writer.
fn check_row(row: &Row, prefix: &[u32], prefix_hash: &str, source: Option<u32>) -> Result<()> {
    if row.prefix_sha256 != prefix_hash
        || row.source != source
        || source.is_some_and(|i| i as usize >= prefix.len())
        || row.steps.len() != row.generated.len()
        || row.steps.len() > CAP
        || (row.steps.len() != CAP && row.generated.last() != Some(&EOS))
    {
        return Err("unbound trajectory or incomplete EOS rollout".into());
    }
    let position = source.unwrap_or(X);
    let mut state = [
        u32::from(source.is_some()),
        position,
        position,
        prefix.len() as u32,
    ];
    let mut first_free = None;
    for (i, (step, id)) in row.steps.iter().zip(&row.generated).enumerate() {
        let [active, start, cursor, end] = state;
        let release = if active == 0 {
            0
        } else if cursor - start == CURSOR_RELEASE_CAP {
            3
        } else if cursor == end {
            2
        } else if STOPS.contains(&prefix[cursor as usize]) {
            1
        } else {
            0
        };
        let copied = active == 1 && release == 0;
        let completed = release == 1 && cursor > start;
        let source_id = copied.then(|| prefix[cursor as usize]);
        let emitted = source_id.unwrap_or(if completed { EOS } else { step.native_id });
        let after = [
            u32::from(active == 1 && release == 0),
            start,
            cursor + u32::from(copied),
            end,
        ];
        if step.before != state
            || step.after != after
            || step.release != release
            || step.source_id != source_id
            || step.copied != u32::from(copied)
            || step.native_top8[0] != step.native_id
            || step.emitted != emitted
            || *id != emitted
            || (*id == EOS && i + 1 != row.steps.len())
        {
            return Err(format!("invalid original-word feedback at step {i}").into());
        }
        if !copied && !completed && first_free.is_none() {
            first_free = Some((i, emitted));
        }
        state = after;
    }
    if row.first_free != first_free {
        return Err("first-free event mismatch".into());
    }
    Ok(())
}

#[derive(Deserialize)]
struct Picks {
    artifact_sha256: String,
    bare_sha256: String,
    rows: Vec<Pick>,
}
#[derive(Deserialize)]
struct Pick {
    prefix_sha256: String,
    continuation: Continuation,
}
#[derive(Deserialize)]
struct Continuation {
    source: Option<u32>,
}
#[derive(Deserialize)]
struct Observed {
    schema: String,
    model: String,
    bare: String,
    picks: String,
    writer_cubin: String,
    completion_cubin: String,
    transitions: usize,
    rows: Vec<Row>,
}
#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Key {
    policy_sha256: String,
    prefix_sha256: String,
    source: Option<u32>,
}
struct Entry {
    row: Row,
    receipt_sha256: String,
}
struct Cache {
    model: String,
    policy_hash: String,
    entries: BTreeMap<Key, Entry>,
}
impl Cache {
    fn key(&self, pick: &Pick) -> Key {
        Key {
            policy_sha256: self.policy_hash.clone(),
            prefix_sha256: pick.prefix_sha256.clone(),
            source: pick.continuation.source,
        }
    }
    fn import(&mut self, writer: &Path, handoff: &Path, bare: &Path) -> Result<usize> {
        let bytes = fs::read(writer)?;
        let observed: Observed = serde_json::from_slice(&bytes)?;
        let receipt_sha256 = hash(&bytes);
        let picks: Picks = load(handoff)?;
        let prompts: Vec<Prompt> = load(bare)?;
        if observed.schema != "uor-r4.entity-writer/1"
            || observed.model != self.model
            || picks.artifact_sha256 != self.model
            || observed.bare != file_hash(bare)?
            || picks.bare_sha256 != observed.bare
            || observed.picks != file_hash(handoff)?
            || observed.writer_cubin != BASE_CUBIN
            || observed.completion_cubin != EOS_CUBIN
            || observed.rows.len() != prompts.len()
            || picks.rows.len() != prompts.len()
            || observed.transitions != observed.rows.iter().map(|r| r.steps.len()).sum::<usize>()
        {
            return Err(
                format!("invalid observed trajectory receipt: {}", writer.display()).into(),
            );
        }
        for ((prompt, pick), row) in prompts.iter().zip(&picks.rows).zip(observed.rows) {
            let digest = prompt.digest()?;
            if pick.prefix_sha256 != digest {
                return Err("handoff prefix hash changed".into());
            }
            let key = self.key(pick);
            check_row(&row, &prompt.prefix, &digest, key.source)?;
            if let Some(prior) = self.entries.get(&key) {
                if prior.row != row {
                    return Err("same exact writer key has different raw device words".into());
                }
            } else {
                self.entries.insert(
                    key,
                    Entry {
                        row,
                        receipt_sha256: receipt_sha256.clone(),
                    },
                );
            }
        }
        Ok(observed.transitions)
    }
}

struct Run {
    stage: String,
    split: String,
    seed: String,
    out: PathBuf,
    prior: PathBuf,
    bare: PathBuf,
    model: PathBuf,
    model_hash: String,
    bare_hash: String,
    tokenizer: PathBuf,
    role_bank: PathBuf,
    span: PathBuf,
    exact: PathBuf,
    content: PathBuf,
    prior_bank: PathBuf,
    fence: PathBuf,
    fence_gpu: PathBuf,
    agree: PathBuf,
    agree_gpu: PathBuf,
    chain: PathBuf,
    writer: PathBuf,
    region: PathBuf,
    lock: PathBuf,
    script: PathBuf,
}
impl Run {
    fn gpu(&self, binary: &Path, args: &[&Path]) -> Result<()> {
        run(Command::new("flock")
            .arg(&self.lock)
            .arg("bash")
            .arg(&self.script)
            .arg("gpu")
            .arg(binary)
            .args(args))
    }
    fn prior_arm(&self) -> PathBuf {
        self.prior.join(format!(
            "fence/results/{}/paired-role/seed{}-{}/fenced",
            self.stage, self.seed, self.split
        ))
    }
    fn rollout(
        &self,
        cache: &mut Cache,
        dir: &Path,
        rank: &str,
        picks: &Picks,
        prompts: &[Prompt],
    ) -> Result<(usize, usize)> {
        let mut missing = BTreeMap::new();
        for (pick, prompt) in picks.rows.iter().zip(prompts) {
            let key = cache.key(pick);
            if !cache.entries.contains_key(&key) {
                missing.insert(key, prompt);
            }
        }
        let mut transitions = 0;
        if !missing.is_empty() {
            let bare = dir.join(format!("{rank}-actual-bare.json"));
            save(&bare, &missing.values().collect::<Vec<_>>())?;
            let actual = dir.join(format!("{rank}-actual-picks.json"));
            save(
                &actual,
                &json!({"schema":"uor-r4.entity-pair-chain/1", "arm":"action",
                "artifact_sha256":self.model_hash, "bare_sha256":file_hash(&bare)?,
                "rows":missing.keys().map(|k| json!({"prefix_sha256":k.prefix_sha256, "continuation":{
                    "source":k.source, "reason":0, "connector_mask":0, "onset_mask":0}})).collect::<Vec<_>>() }),
            )?;
            let manifest = dir.join(format!("{rank}-actual-manifest.json"));
            save(
                &manifest,
                &json!([{"model":self.model_hash,"bare":file_hash(&bare)?,"picks": file_hash(&actual)?}]),
            )?;
            let output = dir.join(format!("{rank}-actual-writer.json"));
            self.gpu(
                &self.writer,
                &[
                    Path::new("entity-writer-action"),
                    &manifest,
                    Path::new(&file_hash(&manifest)?),
                    &self.model,
                    &bare,
                    &actual,
                    &output,
                ],
            )?;
            transitions = cache.import(&output, &actual, &bare)?;
        }
        Ok((missing.len(), transitions))
    }
    fn arm(
        &self,
        arm: &str,
        cache: &mut Cache,
        prompts: &[Prompt],
        inputs: &[String],
    ) -> Result<()> {
        let dir = self.out.join(arm);
        fs::create_dir(&dir)?;
        let request = self.region.join(format!(
            "{}/seed{}-{}-{arm}.json",
            self.stage, self.seed, self.split
        ));
        let nomination = dir.join("nomination.json");
        let source: Value = load(&request)?;
        if source["artifact_sha256"] != self.model_hash
            || source["bare_sha256"] != self.bare_hash
            || source["tokenizer_sha256"] != inputs[0]
            || source["state_sha256"] != inputs[1]
        {
            return Err(format!(
                "role output bound to unexpected model, tokenizer or bank: {}",
                request.display()
            )
            .into());
        }
        run(Command::new(&self.fence).arg("nominate").args([
            &self.bare,
            &request,
            &self.span,
            &self.prior_bank,
            &nomination,
        ]))?;
        self.gpu(
            &self.agree,
            &[
                Path::new("enumerate"),
                &self.agree_gpu,
                &self.span,
                &self.prior_bank,
                &self.bare,
                &nomination,
                &dir,
            ],
        )?;
        let sites: Value = load(dir.join("sites.json"))?;
        let ranks = sites["max"].as_u64().ok_or("missing occurrence count")?;
        let mut raw = Vec::new();
        for k in 0..ranks {
            let rank = format!("rank-{k}");
            let chain = dir.join(format!("{rank}-chain.json"));
            let cont = dir.join(format!("{rank}-cont.json"));
            self.gpu(
                &self.chain,
                &[
                    Path::new("cross"),
                    Path::new("role"),
                    &self.model,
                    &self.content,
                    &self.span,
                    &self.exact,
                    &self.bare,
                    &dir.join(format!("{rank}-override.json")),
                    &chain,
                ],
            )?;
            self.gpu(
                &self.fence,
                &[
                    Path::new("apply"),
                    &self.fence_gpu,
                    &self.model,
                    &self.span,
                    &self.exact,
                    &self.bare,
                    &chain,
                    &cont,
                ],
            )?;
            let picks: Picks = load(&cont)?;
            self.verify_picks(&picks, prompts)?;
            let (actual, actual_steps) = self.rollout(cache, &dir, &rank, &picks, prompts)?;
            let entries = picks
                .rows
                .iter()
                .map(|pick| {
                    cache
                        .entries
                        .get(&cache.key(pick))
                        .ok_or("unresolved trajectory")
                })
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let rows = entries.iter().map(|e| &e.row).collect::<Vec<_>>();
            save(
                dir.join(format!("{rank}-writer.json")),
                &json!({"schema":"uor-r4.writer-cache-join/1",
                "model":self.model_hash,"bare":self.bare_hash,"fence_sha256":file_hash(&cont)?,
                "keys":picks.rows.iter().map(|p| cache.key(p)).collect::<Vec<_>>(),
                "trajectory_receipts":entries.iter().map(|e| &e.receipt_sha256).collect::<Vec<_>>(), "rows":rows}),
            )?;
            raw.push(
                json!({"rank":k,"actual":actual,"actual_steps":actual_steps,"uses":rows.len()}),
            );
        }
        let selection = dir.join("selection.json");
        self.gpu(
            &self.agree,
            &[
                Path::new("adjudicate"),
                &self.agree_gpu,
                &self.bare,
                &nomination,
                &dir,
                Path::new("agree"),
                &selection,
            ],
        )?;
        let selection: Value = load(selection)?;
        let pseudo_picks: Picks = serde_json::from_value(
            json!({"artifact_sha256":selection["artifact_sha256"],
            "bare_sha256":selection["bare_sha256"],"rows":selection["rows"].as_array().ok_or("missing consensus rows")?.iter()
                .map(|r| json!({"prefix_sha256":r["prefix_sha256"],"continuation":{"source":r["source"]}})).collect::<Vec<_>>() }),
        )?;
        self.verify_picks(&pseudo_picks, prompts)?;
        let (actual_finals, final_steps) =
            self.rollout(cache, &dir, "chosen", &pseudo_picks, prompts)?;
        let rows = pseudo_picks
            .rows
            .iter()
            .map(|pick| {
                cache
                    .entries
                    .get(&cache.key(pick))
                    .map(|e| &e.row)
                    .ok_or("missing chosen trajectory")
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        save(
            dir.join("chosen.json"),
            &json!({"schema":"uor-r4.chosen-cuda-trajectories/1","bare":self.bare_hash,"model":self.model_hash,
            "transitions":rows.iter().map(|r| r.steps.len()).sum::<usize>(),"rows":rows}),
        )?;
        save(
            dir.join("work.json"),
            &json!({"ranks":raw,"actual_finals":actual_finals,"actual_final_steps":final_steps}),
        )?;
        Ok(())
    }
    fn verify_picks(&self, picks: &Picks, prompts: &[Prompt]) -> Result<()> {
        if picks.artifact_sha256 != self.model_hash
            || picks.bare_sha256 != self.bare_hash
            || picks.rows.len() != prompts.len()
        {
            return Err("unbound fence or agreement rows".into());
        }
        for (pick, prompt) in picks.rows.iter().zip(prompts) {
            if pick.prefix_sha256 != prompt.digest()? {
                return Err("fence or agreement row prefix mismatch".into());
            }
        }
        Ok(())
    }
}

fn baseline(run: &Run) -> Result<()> {
    let tag = format!("seed{}-{}", run.seed, run.split);
    let synthetic = run.split == "nuisance";
    if !synthetic {
        let learned: Value = load(run.prior.join(format!("role/{}/{tag}.json", run.stage)))?;
        let original: Value = load(
            run.region
                .join(format!("{}/{tag}-original.json", run.stage)),
        )?;
        if learned["rows"] != original["rows"] {
            return Err("original paired-role words differ from codex-12".into());
        }
        let old = run.prior.join(format!(
            "fence/cross-grades/{}/{tag}-paired-role-fenced-writer.json",
            run.stage
        ));
        let prior: Value = load(old)?;
        let current: Value = load(run.out.join("original/chosen.json"))?;
        if current != prior {
            return Err("original chosen writer trajectories differ from codex-12".into());
        }
        let prior_select: Value = load(run.prior_arm().join("selection.json"))?;
        let original_select: Value = load(run.out.join("original/selection.json"))?;
        let prior_sites: Value = load(run.prior_arm().join("sites.json"))?;
        let original_sites: Value = load(run.out.join("original/sites.json"))?;
        if prior_select["rows"] != original_select["rows"]
            || prior_sites["sites"] != original_sites["sites"]
        {
            return Err("original consensus membership, selection or source differs".into());
        }
    }
    let make_comparison = |arm: &str| -> Result<(Value, Value, Value, Value)> {
        let req = load(run.region.join(format!("{}/{tag}-{arm}.json", run.stage)))?;
        let select = load(run.out.join(arm).join("selection.json"))?;
        let sites = load(run.out.join(arm).join("sites.json"))?;
        let writer = load(run.out.join(arm).join("chosen.json"))?;
        Ok((req, select, sites, writer))
    };
    let (req, select, sites, writer) = make_comparison("original")?;
    let (identity, its_selection, its_sites, its_writer) = make_comparison("identity")?;
    if req["rows"] != identity["rows"]
        || select["rows"] != its_selection["rows"]
        || sites["sites"] != its_sites["sites"]
        || writer != its_writer
    {
        return Err("identity interval altered words, consensus or writer".into());
    }
    let len = req["rows"].as_array().ok_or("missing request rows")?.len();
    let mut identical = BTreeMap::new();
    for arm in ["region", "adverse", "role-oracle"] {
        let (other, os, oi, ow) = make_comparison(arm)?;
        let mut count = 0;
        for i in 0..len {
            let a = &req["rows"][i];
            let b = &other["rows"][i];
            if a["query_start"] == b["query_start"] && a["query_end"] == b["query_end"] {
                if select["rows"][i] != os["rows"][i]
                    || sites["sites"][i] != oi["sites"][i]
                    || writer["rows"][i] != ow["rows"][i]
                {
                    return Err(format!(
                        "equal request changed consensus or complete writer at {arm} row {i}"
                    )
                    .into());
                }
                count += 1;
            }
        }
        identical.insert(arm, count);
    }
    save(
        run.out.join("identity.json"),
        &json!({"prior_original":!synthetic,"identity":len,"same_request":identical}),
    )
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [stage, split, seed, top, study, script] = args.as_slice() else {
        return Err("expected STAGE SPLIT SEED WICK_ROOT STUDY_ROOT RUN_SCRIPT".into());
    };
    if !matches!(stage.as_str(), "primary" | "repeat")
        || !matches!(split.as_str(), "held" | "fresh" | "nuisance")
        || !matches!(seed.as_str(), "1" | "2")
    {
        return Err("invalid writer panel identity".into());
    }
    let top = Path::new(top);
    let study = Path::new(study);
    let prior = top.join("20261118-paired-role-fence");
    let old = top.join("20261118-role-boundary-count");
    let bank = top.join("20261118-content-support/nominee");
    let value = top.join("20261118-value-connectors");
    let model = value.join(format!("inputs/seed{seed}.d11"));
    let tokenizer = value.join(if seed == "1" {
        "inputs/tokenizer.json"
    } else {
        "inputs/tokenizer-seed-2.json"
    });
    let region = study.join("region");
    let bare = region.join(format!("panels/{split}-blind.json"));
    let out = study.join(format!("writer/{stage}/seed{seed}-{split}"));
    let r = Run {
        stage: stage.clone(),
        split: split.clone(),
        seed: seed.clone(),
        out,
        model_hash: file_hash(&model)?,
        bare_hash: file_hash(&bare)?,
        model,
        bare,
        tokenizer,
        role_bank: prior.join(format!("role/{stage}/seed{seed}.role")),
        span: top.join(format!(
            "20261118-span-connectors/results/span-01/seed{seed}-state.json"
        )),
        exact: value.join(format!("results/value-01/seed{seed}-state.json")),
        content: top.join(format!(
            "20261118-content-selector/results/selector-run-01/seed{seed}/trained.json"
        )),
        prior_bank: bank.join(format!("results/seed{seed}-bank.json")),
        fence: prior.join("fence/bin/assertion-fence-final"),
        fence_gpu: prior.join("fence/bin/fence-device"),
        agree: bank.join("agreement/bin/agree"),
        agree_gpu: bank.join("agreement/bin/agree-device"),
        chain: old.join("bin/chain"),
        writer: old.join("bin/action-writer"),
        region,
        prior,
        lock: top.join("20261118-diagnostics/wick-codex-gpu.lock"),
        script: PathBuf::from(script),
    };
    let tokenizer = file_hash(&r.tokenizer)?;
    let inputs = [tokenizer.clone(), file_hash(&r.role_bank)?];
    if file_hash(&r.writer)? != WRITER {
        return Err("changed exact writer executable".into());
    }
    let policy = json!({"model_sha256":r.model_hash,"tokenizer_sha256":tokenizer,
        "writer_sha256":WRITER,"writer_cubin_sha256":BASE_CUBIN,"completion_cubin_sha256":EOS_CUBIN,
        "stop_ids":STOPS,"cursor_release_cap":CURSOR_RELEASE_CAP,"consensus_value_capacity":CONSENSUS_VALUE_CAP,
        "eos":EOS,"max_output_tokens":CAP,"completion":"delimiter after nonempty exact copy"});
    let mut cache = Cache {
        model: r.model_hash.clone(),
        policy_hash: hash(&serde_json::to_vec(&policy)?),
        entries: BTreeMap::new(),
    };
    fs::create_dir(&r.out)?;
    save(
        r.out.join("policy.json"),
        &json!({"policy_sha256":cache.policy_hash,"policy":policy}),
    )?;
    let prompts: Vec<Prompt> = load(&r.bare)?;
    let mut sources = Vec::new();
    if split != "nuisance" {
        let mut receipt = |writer: &Path, picks: &Path| -> Result<()> {
            let steps = cache.import(writer, picks, &r.bare)?;
            sources.push(json!({"trajectory_sha256":file_hash(writer)?,"handoff_sha256":file_hash(picks)?, "steps":steps}));
            Ok(())
        };
        let dir = r
            .prior
            .join(format!("writer/results/{split}/none/seed{seed}"));
        receipt(&dir.join(format!("{stage}.json")), &dir.join("picks.json"))?;
        let dir = r.prior_arm();
        let sites: Value = load(dir.join("sites.json"))?;
        let ranks = sites["max"].as_u64().ok_or("prior ranks missing")?;
        for k in 0..ranks {
            receipt(
                &dir.join(format!("rank-{k}-writer.json")),
                &dir.join(format!("rank-{k}-picks.json")),
            )?;
        }
    }
    save(
        r.out.join("predecessors.json"),
        &json!({"receipts":sources,"unique_trajectories":cache.entries.len()}),
    )?;
    for arm in ["original", "identity", "region", "adverse", "role-oracle"] {
        r.arm(arm, &mut cache, &prompts, &inputs)?;
    }
    baseline(&r)
}
