//! Standalone local CLI. No training/tensor library is a dependency.
use serde_json::json;
use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
    time::Instant,
};
use uor_r4_integer::{
    bundle::{self, Bundle},
    generation::Request,
    report_output, sha256_file, IntegerError, Result,
};
fn invalid(s: &str) -> IntegerError {
    IntegerError::Invalid(s.into())
}
fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str){
 Some("pack") if args.len()==5=>{bundle::pack(Path::new(&args[1]),Path::new(&args[2]),Path::new(&args[3]),Path::new(&args[4]))?;println!("{}",args[4]);Ok(())},
 Some("generate") if args.len()==4=>{
  let requests:Vec<Request>=serde_json::from_slice(&fs::read(&args[2])?)?;
  if requests.is_empty()||requests.len()>256{return Err(invalid("request batch must contain1..256 prompts"));}
  let root=Path::new(&args[3]);report_output::claim(root)?;
  let whole=Instant::now();let load=Instant::now();let model=Bundle::load(Path::new(&args[1]))?;let load_ns=load.elapsed().as_nanos();
  let mut output=BufWriter::new(fs::File::create_new(root.join("generations.jsonl"))?);let mut tokens=0usize;let mut calls=0usize;let mut model_ns=0u128;
  for (index,request) in requests.iter().enumerate(){let generation=model.generate(request)?;tokens+=generation.generated_token_ids.len();calls+=generation.incremental_step_calls;model_ns+=generation.model_step_nanoseconds;serde_json::to_writer(&mut output,&json!({"index":index,"generation":generation}))?;writeln!(output)?;output.flush()?;eprintln!("completed prompt {}/{}",index+1,requests.len());}
  let summary=json!({"schema":"uor-r4.integer-serving-generation/1","bundle_sha256":model.identity(),"requests_sha256":sha256_file(Path::new(&args[2]))?,"requests":requests.len(),"generated_tokens":tokens,"step_calls":calls,"model_step_nanoseconds":model_ns,"load_nanoseconds":load_ns,"wall_before_summary_nanoseconds":whole.elapsed().as_nanos(),"context":256,"admission":"full","numeric_boundary":"Integer model and selection; legacy metadata parsing/tokenizer/hash/addressing outside model-value instruction claim","timing_scope":"fresh bundle load plus text/token/model/selection/decision-hash/JSONL work; summary and sealing excluded; model clock is nested, not added"});
  fs::write(root.join("summary.json"),serde_json::to_vec_pretty(&summary)?)?;report_output::seal(root)?;report_output::verify(root)?;println!("{}",summary);Ok(())
 },
 _=>Err(invalid("usage: uor-r4-integer pack PACKED TABLES TOKENIZER NEW_BUNDLE | generate BUNDLE REQUESTS_JSON NEW_REPORT"))
 }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
