use uor_r4_discrete::{Error, Result};
fn invalid(s: impl Into<String>) -> Error {
    Error::Invalid(s.into())
}
fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T> {
    serde_json::from_slice(&std::fs::read(path).map_err(|e| invalid(e.to_string()))?)
        .map_err(|e| invalid(e.to_string()))
}
fn report_command(path: &str, action: impl FnOnce() -> Result<serde_json::Value>) -> Result<()> {
    let directory = std::path::Path::new(path);
    std::fs::create_dir(directory).map_err(|e| invalid(format!("exclusive report: {e}")))?;
    let result = action();
    let payload = match &result {
        Ok(value) => value.clone(),
        Err(error) => {
            serde_json::json!({"status":"FAILED","error":error.to_string(),"training":"HELD"})
        }
    };
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join("report.json"))
        .map_err(|e| invalid(e.to_string()))?;
    serde_json::to_writer_pretty(&file, &payload).map_err(|e| invalid(e.to_string()))?;
    file.sync_all().map_err(|e| invalid(e.to_string()))?;
    println!("{}", directory.join("report.json").display());
    result.map(|_| ())
}
#[cfg(feature = "cuda")]
fn device_command(args: &[String]) -> Result<()> {
    use std::path::Path;
    use uor_r4_discrete::{
        device::{Device, Snapshot},
        Identity, InitConfig, Parameter, ReadSpec, Spec,
    };
    let read = |p: &str| std::fs::read(p).map_err(|e| invalid(e.to_string()));
    match args.first().map(String::as_str) {
        Some("attention-audit-qualify") if args.len()==4=>{
            let config=read_json::<uor_r4_discrete::training::DriverConfig>(&args[1])?;
            let admission=read_json::<uor_r4_discrete::controller::attention_audited::qualification::QualificationAdmission>(&args[2])?;
            uor_r4_discrete::controller::attention_audited::qualification::run(config,admission,Path::new(&args[3]))
        }
        Some("attention-audit") if args.len()==4=>{
            let config=read_json::<uor_r4_discrete::training::DriverConfig>(&args[1])?;
            let admission=read_json::<uor_r4_discrete::controller::attention_audited::AuditedAdmission>(&args[2])?;
            uor_r4_discrete::controller::attention_audited::run(config,admission,Path::new(&args[3]))
        }
        Some("attention") | Some("attention-resume") if args.len()==4=>{
            let config=read_json::<uor_r4_discrete::training::DriverConfig>(&args[1])?;
            let admission=read_json::<uor_r4_discrete::controller::attention_learning::AttentionAdmission>(&args[2])?;
            uor_r4_discrete::controller::attention_learning::run(config,admission,Path::new(&args[3]),args[0]=="attention-resume")
        }
        Some("completion") | Some("completion-resume") if args.len()==4=>{
            let config=read_json::<uor_r4_discrete::training::DriverConfig>(&args[1])?;
            let admission=read_json::<uor_r4_discrete::controller::completion::CompletionAdmission>(&args[2])?;
            uor_r4_discrete::controller::completion::run(config,admission,Path::new(&args[3]),args[0]=="completion-resume")
        }
        Some("pilot") | Some("pilot-resume") if args.len()==4=>{
            let config=read_json::<uor_r4_discrete::training::DriverConfig>(&args[1])?;
            let admission=read_json::<uor_r4_discrete::controller::ReleaseManifest>(&args[2])?;
            uor_r4_discrete::controller::run_pilot(config,admission,Path::new(&args[3]),args[0]=="pilot-resume")
        }
        Some("eval-pilot") if args.len()==4=>{
            let config=read_json::<uor_r4_discrete::training::DriverConfig>(&args[1])?;
            let admission=read_json::<uor_r4_discrete::controller::ReleaseManifest>(&args[2])?;
            uor_r4_discrete::controller::evaluate_pilot(config,admission,Path::new(&args[3]))
        }
        Some("build-identity") | Some("completion-build-identity") | Some("attention-build-identity") | Some("attention-audit-build-identity") if (args.len()==1 && args[0]!="attention-audit-build-identity") || args.len()==2=>{
            let executable=std::env::current_exe().map_err(|e|invalid(e.to_string()))?;
            let mut identity=serde_json::json!({"compiled":uor_r4_discrete::device::compiled_identity(),"driver_source_sha256":uor_r4_discrete::training::driver_source_sha256(),"executable_sha256":uor_r4_discrete::corpus::sha256(&std::fs::read(executable).map_err(|e|invalid(e.to_string()))?),"cuda_context_created":false,"training":"HELD"});
            if args.len()==2 {
                let config=read_json::<uor_r4_discrete::training::DriverConfig>(&args[1])?;
                if args[0]=="attention-audit-build-identity" {
                    uor_r4_discrete::controller::attention_audited::validate_config(&config)?;
                    identity["audited_scope"]=serde_json::json!(uor_r4_discrete::controller::attention_audited::scope(&config)?);
                }
                else if args[0]=="completion-build-identity" {uor_r4_discrete::controller::completion::validate_config(&config)?;}
                else if args[0]=="attention-build-identity" {uor_r4_discrete::controller::attention_learning::validate_config(&config)?;}
                else {uor_r4_discrete::controller::validate_pilot_config(&config)?;}
                let driver=uor_r4_discrete::training::PreparedDriver::load(config)?;
                identity["config_sha256"]=serde_json::json!(uor_r4_discrete::corpus::sha256(&serde_json::to_vec(driver.config()).map_err(|e|invalid(e.to_string()))?));
                identity["lineage_sha256"]=serde_json::json!(driver.lineage_sha256()?);
            }
            println!("{identity}");
            Ok(())
        }
        Some("qualify") if args.len()==2=>uor_r4_discrete::qualification::run(Path::new(&args[1])),
        Some("qualify-flock") if args.len()==2=>uor_r4_discrete::flock_qualification::run(Path::new(&args[1])),
        Some("qualify-relation") if args.len()==2=>uor_r4_discrete::relation_qualification::run(Path::new(&args[1])),
        Some("qualify-driver") if args.len()==2=>uor_r4_discrete::driver_qualification::run(Path::new(&args[1])),
        Some("measure-cost") if args.len()==2=>uor_r4_discrete::measurement::run(Path::new(&args[1])),
        Some("init-corpus") if args.len()==3=>{
            let config=read_json::<uor_r4_discrete::training::DriverConfig>(&args[1])?;
            config.validate()?;
            report_command(&args[2],|| {
                let driver=uor_r4_discrete::training::PreparedDriver::load(config)?;
                let model=driver.new_model(0)?;
                serde_json::to_value(driver.checkpoint(&model,driver.initial_cursor())?)
                    .map_err(|e|invalid(e.to_string()))
            })
        }
        Some("eval-corpus") if args.len()==4=>{
            let config=read_json::<uor_r4_discrete::training::DriverConfig>(&args[1])?;
            config.validate()?;
            report_command(&args[3],|| {
                let driver=uor_r4_discrete::training::PreparedDriver::load(config)?;
                let checkpoint=read_json::<uor_r4_discrete::training::DriverCheckpoint>(&args[2])?;
                if checkpoint.cursor.accepted_updates!=0 {
                    return Err(Error::TrainingHeld);
                }
                let (model,_cursor)=driver.restore(checkpoint.clone(),0)?;
                driver.evaluate_checkpoint(&model,&checkpoint)
            })
        }
        Some("init") if args.len()==4=>{
            let seed:u32=args[3].parse().map_err(|_|invalid("seed must be an explicit unsigned32 integer"))?;
            let spec:Spec=serde_json::from_slice(&read(&args[1])?).map_err(|e|invalid(e.to_string()))?;
            spec.validate()?;
            // Exclusive output is reserved before model initialization.
            let output=std::fs::OpenOptions::new().write(true).create_new(true).open(&args[2]).map_err(|e|invalid(e.to_string()))?;
            let model=Device::new(spec,seed,0)?;
            serde_json::to_writer(output,&model.snapshot()?).map_err(|e|invalid(e.to_string()))
        }
        Some("init-read") if args.len()==4=>{
            let seed:u32=args[3].parse().map_err(|_|invalid("seed must be an explicit unsigned32 integer"))?;
            let config:InitConfig=serde_json::from_slice(&read(&args[1])?).map_err(|e|invalid(e.to_string()))?;
            config.read_spec.validate(config.spec)?;
            let output=std::fs::OpenOptions::new().write(true).create_new(true).open(&args[2]).map_err(|e|invalid(e.to_string()))?;
            let model=Device::new_with_objective(config.spec,config.read_spec,config.objective_policy,seed,0)?;
            serde_json::to_writer(output,&model.snapshot()?).map_err(|e|invalid(e.to_string()))
        }
        Some("infer")|Some("propose") if args.len()>=3=>{
            let parameter = if args[0] == "propose" {
                if args.len() != 5 { return Err(invalid("propose requires ARTIFACT REQUEST route|vector|gate INDEX")); }
                let index = args[4].parse().map_err(|_| invalid("parameter index must be unsigned"))?;
                Some(match args[3].as_str() {
                    "route" => Parameter::Route(index),
                    "vector" => Parameter::VectorBlock(index),
                    "gate" => Parameter::Gate(index),
                    _ => return Err(invalid("parameter must be route, vector or gate")),
                })
            } else {
                if args.len() != 3 { return Err(invalid("infer requires ARTIFACT REQUEST")); }
                None
            };
            #[derive(serde::Deserialize)] #[serde(deny_unknown_fields)]
            struct Request{windows:Vec<Vec<Identity>>,targets:Vec<u32>}
            let snapshot_bytes=read(&args[1])?;
            let request_bytes=read(&args[2])?;
            let snapshot:Snapshot=serde_json::from_slice(&snapshot_bytes).map_err(|e|invalid(e.to_string()))?;
            let request:Request=serde_json::from_slice(&request_bytes).map_err(|e|invalid(e.to_string()))?;
            snapshot.read_spec.validate(snapshot.spec)?;
            match (snapshot.read_spec,parameter) {
                (ReadSpec::Route,Some(Parameter::Gate(index)))=>return Err(Error::UnimplementedParameter {kind:3,index}),
                (ReadSpec::Flock{..},Some(Parameter::Route(_)))=>return Err(invalid("route parameters are inactive in a flock model")),
                (ReadSpec::Relation{..},Some(Parameter::Route(_)))=>return Err(invalid("route parameters are inactive in a relation model")),
                _=>{}
            }
            if request.windows.is_empty() || request.windows.len()>64 || request.windows.iter().any(|w|w.is_empty() || w.len()>snapshot.spec.context as usize || w.iter().any(|id| id.token>=snapshot.spec.vocabulary)){
                return Err(invalid("each of1..64 windows must be a nonempty caller-bounded causal prefix of length1..context with valid IDs"));
            }
            if request.targets.len()!=request.windows.len() || request.targets.iter().any(|&id| id>=snapshot.spec.vocabulary){return Err(invalid("one valid next-token target is required per window"));}
            let model=Device::from_snapshot(snapshot,0)?;
            let input=model.carry_windows(&request.windows)?;
            if let Some(parameter)=parameter {let proposal=model.propose(&input,&request.targets,parameter)?;let p=proposal.observation()?;let candidate_losses=proposal.candidate_losses()?;
                let (parameter_kind,parameter_index)=match parameter {Parameter::Route(i)=>("route",i),Parameter::VectorBlock(i)=>("vector",i),Parameter::Gate(i)=>("gate",i)};
                let executing_elf=std::env::current_exe().map_err(|e|invalid(e.to_string()))?;
                println!("{}",serde_json::json!({"candidate":p.candidate,"candidate_is_ordinal":true,"read_spec":model.read_spec(),"before":p.before,"after":p.after,"any_candidate_cost_differs_from_identity":p.any_candidate_cost_differs_from_identity,"candidate_losses":candidate_losses,"epoch":p.epoch,"valid_lengths":input.valid_lengths(),"applied":false,
                    "parameter_kind":parameter_kind,"parameter_index":parameter_index,"snapshot_file_sha256":uor_r4_discrete::corpus::sha256(&snapshot_bytes),"request_file_sha256":uor_r4_discrete::corpus::sha256(&request_bytes),
                    "producer_compiled_identity":uor_r4_discrete::device::compiled_identity(),"producer_driver_source_sha256":uor_r4_discrete::training::driver_source_sha256(),"producer_executable_sha256":uor_r4_discrete::corpus::sha256(&std::fs::read(executing_elf).map_err(|e|invalid(e.to_string()))?)}));Ok(())}
            else {let result=model.forward(&input,&request.targets)?;println!("{}",serde_json::json!({"tokens":result.predictions,"read_spec":model.read_spec(),"loss_numerators":result.losses,"token_rivals":result.token_rivals,"address_rivals":result.address_rivals,"admitted_cells":result.admitted_cells,"vectors":result.vectors()?,"epoch":result.epoch(),"valid_lengths":input.valid_lengths(),"training":"HELD"}));Ok(())}
        }
        _=>Err(invalid("CUDA commands: pilot|pilot-resume|eval-pilot CONFIG ADMISSION NEW_REPORT_DIR | init SPEC.json NEW_ARTIFACT.json SEED | init-read CONFIG.json NEW_ARTIFACT.json SEED | infer ARTIFACT REQUEST | propose ARTIFACT REQUEST route|vector|gate INDEX | init-corpus CONFIG NEW_REPORT_DIR | eval-corpus CONFIG DRIVER_CHECKPOINT NEW_REPORT_DIR | qualify NEW_REPORT_DIR | qualify-flock NEW_REPORT_DIR | qualify-relation NEW_REPORT_DIR | qualify-driver NEW_REPORT_DIR | measure-cost NEW_REPORT_DIR")),
    }
}
fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--help") | Some("help") => {
            println!("owner-release PILOT_PLAN.json NEW_RELEASE_DIR — Casey at a live terminal (UOR_LAB unset) with typed confirmation, or the D33-named delegate UOR_LAB=claude for the pinned A2 pilot only; no other lab may invoke this or create/copy/edit owner-release files.
pilot-unlock RUN_STATE EXPECTED_LOCK_SHA256 NEW_RECEIPT_DIR — metadata-only recovery, refuses live/foreign/ambiguous holders and preserves the old lock.
pilot|pilot-resume|eval-pilot CONFIG ADMISSION NEW_REPORT_DIR — CUDA-only, explicit owner release required; training remains HELD by default.
build-identity [CONFIG] — compiled/executable/config pins without a CUDA context.
completion-build-identity [CONFIG] — compiled K8 completion or K16 redirect identity without CUDA context.
completion-check-plan PLAN — read-only plan metadata validation, no release.
completion-owner-release PLAN NEW_RELEASE_DIR — Casey personally, separate arm/seed scope; no D33 delegation.
completion|completion-resume CONFIG ADMISSION NEW_REPORT_DIR — separately released arm/seed, cap16404; never chains scopes.");
            println!("attention-audit-build-identity CONFIG — new audited scope compiled/config/lineage identity; no CUDA context.
attention-audit-check-plan PLAN — new FIT/anchor/source/qualification/cost/review metadata validation; no release.
attention-audit-owner-release PLAN NEW_RELEASE_DIR — Casey personally, exact CUDA executable and new scope; no delegated/old release.
attention-audit CONFIG ADMISSION NEW_REPORT_DIR — separately released whole-interval rejection; no resume or fresh execution.
attention-audit-qualification-check-plan PLAN — distinct finite qualification metadata chain, never dose release.
attention-audit-qualification-owner-release PLAN NEW_RELEASE_DIR — Casey personally, separately FIT-qualified finite scope only.
attention-audit-qualify CONFIG QUALIFICATION_ADMISSION NEW_REPORT_DIR — bounded actual source controls/calibration, no complete dose.
attention-build-identity [CONFIG] — compiled/config/lineage identity without a CUDA context.
attention-check-plan PLAN — metadata validation only; no release.
attention-owner-release PLAN NEW_RELEASE_DIR — Casey personally at a terminal, UOR_LAB unset; separate attention arm/seed scope, no D33 delegation.
attention-panel CONFIG NEW_REPORT_DIR — canonical DEV row metadata and first64 successor probe request, no model.
attention-export-token-cells CONFIG DRIVER_CHECKPOINT NEW_REPORT_DIR — read-only CUDA token-to-cell map from an exact checkpoint; no forward or apply.
attention-capture-dev CONFIG DRIVER_CHECKPOINT DEV_EVALUATION NEW_REPORT_DIR — separately released zero-apply GPU Forward slot capture.
attention-dev-b3 CONFIG DRIVER_CHECKPOINT DEV_EVALUATION NEW_REPORT_DIR — exact immutable DEV B3 metadata, no model.
attention-transfer PAIRED_DEV_DECISION NEW_REPORT_DIR — CUDA-only once-only paired fresh evaluation after sealed DEV H_geom.
attention|attention-resume CONFIG ADMISSION NEW_REPORT_DIR — CUDA-only matched active-leaf Flock/Relation learning, cap16404, separately released; training remains HELD by default.");
            Ok(())
        }
        Some("completion-check-plan") if args.len() == 2 => {
            let plan =
                read_json::<uor_r4_discrete::controller::completion::CompletionPlan>(&args[1])?;
            println!(
                "{}",
                uor_r4_discrete::controller::completion::check_plan(&plan)?
            );
            Ok(())
        }
        Some("attention-check-plan") if args.len() == 2 => {
            println!(
                "{}",
                uor_r4_discrete::controller::attention_learning::check_plan(std::path::Path::new(
                    &args[1]
                ))?
            );
            Ok(())
        }
        Some("attention-audit-check-plan") if args.len() == 2 => {
            println!(
                "{}",
                uor_r4_discrete::controller::attention_audited::check_plan(std::path::Path::new(
                    &args[1]
                ))?
            );
            Ok(())
        }
        Some("attention-audit-qualification-check-plan") if args.len() == 2 => {
            println!(
                "{}",
                uor_r4_discrete::controller::attention_audited::qualification::check_plan(
                    std::path::Path::new(&args[1])
                )?
            );
            Ok(())
        }
        Some("attention-audit-qualification-owner-release") if args.len() == 3 => {
            uor_r4_discrete::controller::attention_audited::qualification::owner_release(
                std::path::Path::new(&args[1]),
                std::path::Path::new(&args[2]),
            )
        }
        Some("attention-audit-owner-release") if args.len() == 3 => {
            uor_r4_discrete::controller::attention_audited::owner_release(
                std::path::Path::new(&args[1]),
                std::path::Path::new(&args[2]),
            )
        }
        Some("attention-panel") if args.len() == 3 => report_command(&args[2], || {
            uor_r4_discrete::controller::attention_learning::preflight::panel(std::path::Path::new(
                &args[1],
            ))
        }),
        Some("attention-export-token-cells") if args.len() == 4 => report_command(&args[3], || {
            uor_r4_discrete::controller::attention_learning::preflight::export_token_cells(
                std::path::Path::new(&args[1]),
                std::path::Path::new(&args[2]),
                std::path::Path::new(&args[3]),
            )
        }),
        Some("attention-capture-dev") if args.len() == 5 => report_command(&args[4], || {
            uor_r4_discrete::controller::attention_learning::preflight::capture_dev(
                std::path::Path::new(&args[1]),
                std::path::Path::new(&args[2]),
                std::path::Path::new(&args[3]),
            )
        }),
        Some("attention-dev-b3") if args.len() == 5 => report_command(&args[4], || {
            uor_r4_discrete::controller::attention_learning::preflight::dev_b3(
                std::path::Path::new(&args[1]),
                std::path::Path::new(&args[2]),
                std::path::Path::new(&args[3]),
            )
        }),
        Some("attention-transfer") if args.len() == 3 => {
            uor_r4_discrete::controller::attention_learning::transfer::run(
                std::path::Path::new(&args[1]),
                std::path::Path::new(&args[2]),
            )
        }
        Some("attention-owner-release") if args.len() == 3 => {
            uor_r4_discrete::controller::attention_learning::owner_release(
                std::path::Path::new(&args[1]),
                std::path::Path::new(&args[2]),
            )
        }
        Some("completion-owner-release") if args.len() == 3 => {
            uor_r4_discrete::controller::completion::owner_release(
                std::path::Path::new(&args[1]),
                std::path::Path::new(&args[2]),
            )
        }
        Some("owner-release") if args.len() == 3 => uor_r4_discrete::controller::owner_release(
            std::path::Path::new(&args[1]),
            std::path::Path::new(&args[2]),
        ),
        Some("pilot-unlock") if args.len() == 4 => uor_r4_discrete::controller::unlock_pilot(
            std::path::Path::new(&args[1]),
            &args[2],
            std::path::Path::new(&args[3]),
        ),
        Some("train") => uor_r4_discrete::training_admission(),
        Some("prepare-plan") if args.len() == 3 => {
            let config = read_json::<uor_r4_discrete::training::DriverConfig>(&args[1])?;
            config.validate()?;
            report_command(&args[2], || {
                uor_r4_discrete::training::PreparedDriver::load(config)?.plan()
            })
        }
        Some("schema") => {
            println!(
                "{}; integer CUDA only; training HELD",
                uor_r4_discrete::SCHEMA
            );
            Ok(())
        }
        Some("list-controls") => {
            #[cfg(feature = "cuda")]
            {
                for name in uor_r4_discrete::qualification::NAMES {
                    println!("{name}");
                }
                Ok(())
            }
            #[cfg(not(feature = "cuda"))]
            Err(invalid(
                "CUDA qualification inventory is not compiled in this metadata-only build",
            ))
        }
        Some("list-flock-controls") => {
            #[cfg(feature = "cuda")]
            {
                for name in uor_r4_discrete::flock_qualification::NAMES {
                    println!("{name}");
                }
                Ok(())
            }
            #[cfg(not(feature = "cuda"))]
            Err(invalid(
                "CUDA flock inventory is not compiled in this metadata-only build",
            ))
        }
        Some("list-admission-rank-controls") => {
            println!("{}", uor_r4_discrete::objective::control_scaffold());
            Ok(())
        }
        Some("list-relation-controls") => {
            #[cfg(feature = "cuda")]
            {
                for name in uor_r4_discrete::relation_qualification::NAMES {
                    println!("{name}");
                }
                Ok(())
            }
            #[cfg(not(feature = "cuda"))]
            Err(invalid(
                "CUDA relation inventory is not compiled in this metadata-only build",
            ))
        }
        Some("list-driver-controls") => {
            #[cfg(feature = "cuda")]
            {
                for name in uor_r4_discrete::driver_qualification::NAMES {
                    println!("{name}");
                }
                Ok(())
            }
            #[cfg(not(feature = "cuda"))]
            Err(invalid(
                "CUDA driver inventory is not compiled in this metadata-only build",
            ))
        }
        _ => {
            #[cfg(feature = "cuda")]
            return device_command(&args);
            #[cfg(not(feature = "cuda"))]
            Err(invalid(
                "CUDA build required for model commands; no host fallback",
            ))
        }
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(2);
    }
}
