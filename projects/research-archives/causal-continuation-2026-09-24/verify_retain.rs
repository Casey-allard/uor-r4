//! Native verification and retention, with no mutation of earlier report roots.
use std::{fs,path::Path};
use sha2::{Digest,Sha256};
use serde_json::json;
use uor_r4_core::{report_output::{claim,seal,verify},native_geometric::learner::transferable_lexical::{TlModel,TlTrainer}};
fn checked(p:&Path)->Result<(),Box<dyn std::error::Error>>{let errors=verify(p)?;if !errors.is_empty(){return Err(format!("{} {errors:?}",p.display()).into());}Ok(())}
fn hash(b:&[u8])->String{format!("{:x}",Sha256::digest(b))}
fn main()->Result<(),Box<dyn std::error::Error>>{
 let arg=std::env::args().nth(1).ok_or("investigation root required")?;let root=Path::new(&arg);let mut reports=Vec::new();
 for entry in fs::read_dir(root)?{let p=entry?.path();if p.is_dir()&&p.join("manifest.json").is_file(){checked(&p)?;reports.push(json!({"root":p,"manifest_sha256":hash(&fs::read(p.join("manifest.json"))?)}));}}
 reports.sort_by_key(|r|r["root"].as_str().unwrap_or("").to_owned());
 let source=root.join("intervened-1");checked(&source)?;
 let bytes=fs::read(source.join("selected.tlx"))?;let m=TlModel::from_bytes(&bytes)?;if m.to_bytes()!=bytes{return Err("selected model roundtrip mismatch".into());}
 let expected="1ac700658a476e356ff64db95003277f30c27b3d181ca183444dd7621c5d955f";if hash(&bytes)!=expected{return Err("selected model differs from freeze".into());}
 let ck=fs::read(source.join("final.tlk"))?;let meta:serde_json::Value=serde_json::from_slice(&ck[8..8+u32::from_le_bytes(ck[4..8].try_into()?)as usize])?;
 let data: [u8;32]=serde_json::from_value(meta["cursor"]["data_sha256"].clone())?;let tokenizer:[u8;32]=serde_json::from_value(meta["cursor"]["tokenizer_sha256"].clone())?;
 let(tr,cursor)=TlTrainer::from_checkpoint(&ck,data,tokenizer)?;if tr.model()?!=m||cursor.next_batch!=256{return Err("checkpoint served state mismatch".into());}
 let retain=Path::new("/Users/casey.allard/uor-r4/.uor-models/causal-continuation-2026-09-24");claim(retain)?;
 let mappings=[("intervened-1/selected.tlx","causal-candidate.tlx"),("intervened-1/final.tlk","final.tlk"),("intervened-1/checkpoint-128.tlk","checkpoint-128.tlk"),("intervened-1/control-128.json","control-128.json"),("intervened-1/control-256.json","control-256.json"),("intervened-1/model-256.tlx","model-256.tlx"),("ordinary-1/selected.tlx","ordinary-control.tlx"),("geometry-1/relative.q8l","relative.q8l"),("fitting-protocol.json","fitting-protocol.json"),("data-plan.json","data-plan.json"),("final-evaluation-freeze.json","final-evaluation-freeze.json")];
 let mut files=Vec::new();for(src,dst)in mappings{let b=fs::read(root.join(src))?;fs::write(retain.join(dst),&b)?;if fs::read(retain.join(dst))?!=b{return Err("retained copy mismatch".into());}files.push(json!({"name":dst,"source":root.join(src),"sha256":hash(&b),"bytes":b.len()}));}
 let registry=json!({"schema":"uor-r4.causal-retained/1","status":"Research candidate: causal feedback and new-source scoring gains; generation and general coding remain unqualified. No global serving default changed.","model_sha256":expected,"files":files,"source_root":root,"data_root":root.join("data"),"checkpoint_resume":"Use the frozen causal-lexical executable and identical declared data/schedule. Step128 checkpoint has its validation companion. A new training phase is not an identical continuation of the completed256-step schedule.","reports":reports});
 fs::write(retain.join("registry.json"),serde_json::to_vec_pretty(&registry)?)?;seal(retain)?;checked(retain)?;
 fs::write(root.join("native-verification.json"),serde_json::to_vec_pretty(&json!({"verified_reports":reports,"retained_root":retain,"retained_files":files,"selected_model_sha256":expected,"checkpoint_model_equal":true,"checkpoint_step":cursor.next_batch,"retained_verified":true}))?)?;
 println!("Native verified {} report roots and retained {}",reports.len(),retain.display());Ok(())
}
