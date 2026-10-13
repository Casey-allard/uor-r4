use uor_r4_discrete::{Error, Result};
fn invalid(s: impl Into<String>) -> Error {
    Error::Invalid(s.into())
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
        Some("qualify") if args.len()==2=>uor_r4_discrete::qualification::run(Path::new(&args[1])),
        Some("qualify-flock") if args.len()==2=>uor_r4_discrete::flock_qualification::run(Path::new(&args[1])),
        Some("init") if args.len()==3=>{
            let spec:Spec=serde_json::from_slice(&read(&args[1])?).map_err(|e|invalid(e.to_string()))?;
            spec.validate()?;
            // Exclusive output is reserved before model initialization.
            let output=std::fs::OpenOptions::new().write(true).create_new(true).open(&args[2]).map_err(|e|invalid(e.to_string()))?;
            let model=Device::new(spec,7,0)?;
            serde_json::to_writer(output,&model.snapshot()?).map_err(|e|invalid(e.to_string()))
        }
        Some("init-read") if args.len()==3=>{
            let config:InitConfig=serde_json::from_slice(&read(&args[1])?).map_err(|e|invalid(e.to_string()))?;
            config.read_spec.validate(config.spec)?;
            let output=std::fs::OpenOptions::new().write(true).create_new(true).open(&args[2]).map_err(|e|invalid(e.to_string()))?;
            let model=Device::new_with_read(config.spec,config.read_spec,7,0)?;
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
            let snapshot:Snapshot=serde_json::from_slice(&read(&args[1])?).map_err(|e|invalid(e.to_string()))?;
            let request:Request=serde_json::from_slice(&read(&args[2])?).map_err(|e|invalid(e.to_string()))?;
            snapshot.read_spec.validate(snapshot.spec)?;
            match (snapshot.read_spec,parameter) {
                (ReadSpec::Route,Some(Parameter::Gate(index)))=>return Err(Error::UnimplementedParameter {kind:3,index}),
                (ReadSpec::Flock{..},Some(Parameter::Route(_)))=>return Err(invalid("route parameters are inactive in a flock model")),
                _=>{}
            }
            if request.windows.is_empty() || request.windows.len()>64 || request.windows.iter().any(|w|w.is_empty() || w.len()>snapshot.spec.context as usize || w.iter().any(|id| id.token>=snapshot.spec.vocabulary)){
                return Err(invalid("each of1..64 windows must be a nonempty caller-bounded causal prefix of length1..context with valid IDs"));
            }
            if request.targets.len()!=request.windows.len() || request.targets.iter().any(|&id| id>=snapshot.spec.vocabulary){return Err(invalid("one valid next-token target is required per window"));}
            let model=Device::from_snapshot(snapshot,0)?;
            let input=model.carry_windows(&request.windows)?;
            if let Some(parameter)=parameter {let p=model.propose(&input,&request.targets,parameter)?.observation()?;println!("{}",serde_json::json!({"candidate":p.candidate,"candidate_is_ordinal":true,"read_spec":model.read_spec(),"before":p.before,"after":p.after,"epoch":p.epoch,"valid_lengths":input.valid_lengths(),"applied":false}));Ok(())}
            else {let result=model.forward(&input,&request.targets)?;println!("{}",serde_json::json!({"tokens":result.predictions,"read_spec":model.read_spec(),"loss_numerators":result.losses,"token_rivals":result.token_rivals,"address_rivals":result.address_rivals,"admitted_cells":result.admitted_cells,"vectors":result.vectors()?,"epoch":result.epoch(),"valid_lengths":input.valid_lengths(),"training":"HELD"}));Ok(())}
        }
        _=>Err(invalid("CUDA commands: init SPEC.json NEW_ARTIFACT.json | init-read CONFIG.json NEW_ARTIFACT.json | infer ARTIFACT REQUEST | propose ARTIFACT REQUEST route|vector|gate INDEX | qualify NEW_REPORT_DIR")),
    }
}
fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("train") => uor_r4_discrete::training_admission(),
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
