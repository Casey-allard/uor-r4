use serde_json::{Value,json};
use std::{collections::BTreeMap,fs};
fn n(v:&Value)->u64{v.as_u64().expect("exact u64")}
fn s(v:&Value)->&str{v.as_str().expect("string")}
fn arr(v:&Value)->&Vec<Value>{v.as_array().expect("array")}
fn main(){
 let a:Vec<String>=std::env::args().collect();let root=&a[1];
 let read=|name:&str|->Value{serde_json::from_slice(&fs::read(format!("{root}/{name}")).unwrap()).unwrap()};
 let v=read("finite-output/counterfactual-reuse-controls-record.json");let pin=read("finite-pin.json");let ident=read("expected-identity.json");
 assert_eq!(v["status"],"FINITE_CONTROL_PASS");
 let records=arr(&v["records"]);let mut kinds=BTreeMap::new();
 for r in records{*kinds.entry(s(&r["kind"]).to_owned()).or_insert(0usize)+=1;}
 let select=|kind:&str|->Vec<&Value>{records.iter().filter(|r|r["kind"]==kind).collect()};
 assert_eq!(kinds,BTreeMap::from([("compiled_identity".into(),1),("control_check".into(),32),("decoded_control_audit".into(),32),("executed_inventory".into(),1),("fixture_start".into(),18),("host_admission_refusal".into(),16),("proposal_attempt".into(),64),("resources".into(),18)]));
 assert_eq!(select("compiled_identity")[0]["compiled"],ident["compiled"]);
 for (key,value) in [("binary_sha256",&ident["binary_sha256"]),("standalone_source_sha256",&ident["standalone_source_sha256"]),("fixture_inventory_sha256",&ident["fixture_inventory_sha256"]),("cuda_images",&ident["compiled"]["cuda_images"]),("baseline_reuse_image",&ident["compiled"]["baseline_reuse_image"])]{assert_eq!(&pin[key],value,"{key}");}
 assert_eq!(pin["target_source_sha256"],ident["compiled"]["source_sha256"]);
 assert_eq!(v["execution_contract"]["pin"],pin);
 assert_eq!(pin["execution_release_reference"],"https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6228335295");
 for r in select("resources") {assert_eq!(r["within_allowance"],true);for (mode,regs,shared,max) in [("U",88,0,640),("R",115,512,512)]{assert_eq!(r[mode],json!({"registers":regs,"local_bytes":21536,"shared_bytes":shared,"max_threads_per_block":max,"ptxas":{"registers":regs,"stack_bytes":21536,"spill_loads":0,"spill_stores":0}}));}}
 let policies=["nearest_address_margin_v1","occupied_cell_rank_hinge_v1"];
 let cases=["ineligible","eligible","mixed","tie","virtual_ineligible","route_endpoint","boundary16","boundary17","boundary64"];
 let props=select("proposal_attempt");let mut pairs=0;let mut success=0;let mut refused=0;let mut lanes_checked=0;let mut row_outcomes=0;
 for policy in policies {for case in cases{
 let parameters:Vec<&str>=match case{"route_endpoint"=>vec!["Route(0)"],"virtual_ineligible"=>vec!["VectorBlock(0)"],_=>vec!["Route(0)","VectorBlock(0)"]};
 let rows=match case{"mixed"|"virtual_ineligible"=>2,"boundary16"=>16,"boundary17"=>17,"boundary64"=>64,_=>1};
 let full=if rows==64{u64::MAX}else{(1u64<<rows)-1};
 for parameter in parameters{
 let get=|mode:&str|->&Value{let found:Vec<_>=props.iter().copied().filter(|r|r["policy"]==policy&&r["fixture"]==case&&r["parameter"]==parameter&&r["mode"]==mode).collect();assert_eq!(found.len(),1);found[0]};
 let u=get("U");let r=get("R");pairs+=1;
 for x in [u,r]{assert_eq!(x["before"],x["after"]);assert_eq!(x["before"]["epoch"],0);assert_eq!(x["apply_calls"],0);assert!(x["state_readback_error"].is_null());assert!(arr(&x["receipt"]["readback_errors"]).is_empty());assert_eq!(x["receipt"]["submission_evidence"],"counterfactual_status_read");assert!(x["receipt"]["trace_error"].is_null());assert_eq!(arr(&x["receipt"]["losses"]).len(),16);for loss in arr(&x["receipt"]["losses"]){n(loss);}}
 assert_eq!(u["before"],r["before"]);assert_eq!(u["fixture_snapshot_sha256"],r["fixture_snapshot_sha256"]);assert_eq!(u["input_sha256"],r["input_sha256"]);
 for field in ["status","losses","proposal","error"]{assert_eq!(u["receipt"][field],r["receipt"][field],"{policy}/{case}/{parameter}/{field}");}
 assert!(u["receipt"]["raw_trace"].is_null());
 let status=if case=="ineligible"{8}else{0};assert_eq!(n(&r["receipt"]["status"]),status);
 if status==8{refused+=2;assert!(r["receipt"]["proposal"].is_null());assert!(arr(&r["receipt"]["losses"]).iter().all(|x|n(x)==u64::MAX));assert_eq!(r["receipt"]["error"],json!({"class":"invalid","site":"proposal","message":"invalid discrete model: discrete CUDA counterfactual refused with status 8"}));}
 else {success+=2;assert!(r["receipt"]["error"].is_null());let p=arr(&r["receipt"]["proposal"]);assert_eq!(p.len(),5);assert!(n(&p[0])<16);assert_eq!(n(&p[3]),0);assert!(n(&p[4])<=1);assert_eq!(p[1],r["receipt"]["losses"][0]);assert_eq!(p[2],r["receipt"]["losses"][n(&p[0])as usize]);if case=="tie"{assert_eq!(n(&p[0]),if parameter=="Route(0)"{0}else{3});}if case=="route_endpoint"{assert_eq!(n(&p[4]),1);println!("Route sensitivity {policy}: proposal={}",r["receipt"]["proposal"]);}}
 let raw=arr(&r["receipt"]["raw_trace"]);assert_eq!(raw.len(),160);let decoded:Vec<_>=select("decoded_control_audit").into_iter().filter(|x|x["policy"]==policy&&x["fixture"]==case&&x["parameter"]==parameter).collect();assert_eq!(decoded.len(),1);let dec=&decoded[0]["audit"];assert_eq!(arr(&dec["rows"]).len(),rows);assert_eq!(arr(&dec["lanes"]).len(),16);assert_eq!(decoded[0]["timing_support"],false);
 for row in 0..64{let eligible=case!="ineligible"&&!(case=="mixed"&&row==0);let expected=if row<rows&&(eligible){1u64<<32}else{0};assert_eq!(n(&raw[row]),expected);if row<rows{row_outcomes+=1;assert_eq!(dec["rows"][row],json!({"status":0,"eligible":eligible}));}}
 let mut union=0;let mut calls=0;for lane in 0..16{
 let producer=(lane..rows).step_by(16).fold(0u64,|mask,row|mask|(1u64<<row));assert_eq!(union&producer,0);union|=producer;calls+=producer.count_ones();
 let (visited,eligible)=match case{"ineligible"=>(1,0),"mixed"=>(3,2),"virtual_ineligible" if lane!=0=>(1,1),_=>(full,full)};
 let expected=[producer,u64::from(producer.count_ones()),visited,eligible,eligible,status];
 for (i,e) in expected.iter().enumerate(){assert_eq!(n(&raw[64+lane*6+i]),*e,"raw {policy}/{case}/{parameter}/lane{lane}/word{i}");}
 assert_eq!(dec["lanes"][lane],json!({"candidate":lane,"producer_mask":producer,"producer_call_count":producer.count_ones(),"consumer_visited_mask":visited,"consumer_eligible_mask":eligible,"virtual_visited_mask":eligible,"terminal_status":status}));lanes_checked+=1;
 }assert_eq!(union,full);assert_eq!(calls as usize,rows);
 if case=="mixed"{let reference=props.iter().find(|x|x["policy"]==policy&&x["fixture"]=="eligible"&&x["parameter"]==parameter&&x["mode"]=="U").unwrap();assert_eq!(u["receipt"]["losses"],reference["receipt"]["losses"]);assert_eq!(u["receipt"]["proposal"],reference["receipt"]["proposal"]);}
 }}}
 assert_eq!((pairs,success,refused,lanes_checked),(32,56,8,512));
 let ids=["route_index_U","vector_index_U","gate_in_route_U","wrong_batch_U","wrong_batch_R","wrong_profiler_U","wrong_profiler_R","wrong_handle_R"];
 for policy in policies{for id in ids{let found:Vec<_>=select("host_admission_refusal").into_iter().filter(|r|r["policy"]==policy&&r["id"]==id).collect();assert_eq!(found.len(),1);let r=found[0];let rc=&r["receipt"];
 let (class,message)=match id{"route_index_U"|"vector_index_U"=>("invalid","invalid discrete model: proposal parameter index exceeds its declared inventory"),"gate_in_route_U"=>("unimplemented_parameter","parameter kind 3 index 0 is reserved but unimplemented"),"wrong_batch_U"|"wrong_batch_R"=>("invalid","invalid discrete model: carried batch belongs to another model/source/geometry"),"wrong_profiler_U"|"wrong_profiler_R"=>("invalid","invalid discrete model: proposal profiler belongs to another model/stream"),_=>("invalid","invalid discrete model: baseline reuse handle belongs to another model/stream")};
 assert_eq!(rc["error"],json!({"class":class,"message":message,"site":"proposal"}));for key in ["status","losses","proposal","raw_trace"]{assert!(rc[key].is_null());}assert!(arr(&rc["readback_errors"]).is_empty());
 if id.ends_with("_R"){assert_eq!(rc["trace_error"],json!({"class":"invalid","message":"invalid discrete model: baseline reuse kernel was not submitted","site":"raw_trace"}));}else{assert!(rc["trace_error"].is_null());}
 for letter in ["A","B"]{assert_eq!(r[format!("before_{letter}")],r[format!("after_{letter}")]);assert_eq!(r[format!("before_{letter}")]["epoch"],0);assert!(r[format!("state_readback_error_{letter}")].is_null());}assert!(r["admission_check_error"].is_null());assert_eq!(r["cuda_fault_parity_claim"],false);
 }}
 for r in select("control_check"){assert_eq!(r["result"],"PASS");}
 assert_eq!(select("executed_inventory")[0]["actual"],json!({"proposal_attempts":64,"successful_proposals":56,"expected_cuda_status8":8,"host_refusal_attempts":16,"endpoint_evaluations":0}));
 println!("PASS exact-u64 metadata audit: {pairs} U/R pairs, {success} success/{refused} status8 attempts, {lanes_checked} R lanes, {row_outcomes} producer rows, 16 typed refusals, 18 strict resource admissions, unchanged state/epoch/apply, zero endpoints/timing.");
}
