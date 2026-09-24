from pathlib import Path
import hashlib,json,subprocess
r=Path('/Users/casey.allard/uor-r4-investigations/causal-20260924T000007')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
a=r/'intervened-1';b=r/'intervened-replay-1'
checks={n:sha(a/n)==sha(b/n) for n in ['final.tlk','final.tlx','selected.tlx']}
assert all(checks.values()),checks
ja=json.loads((a/'receipt.json').read_text())['result'];jb=json.loads((b/'receipt.json').read_text())['result']
assert ja['records']==jb['records'];assert ja['selection']==jb['selection'];assert ja['generation']==jb['generation']
resume={'separate_process':True,'resumed_next_batch':jb['resume_started_at'],'final_batch':jb['completed_steps'],'byte_equality':checks,'validation_trajectory_identical':True,'selected_model_identical':True,'generated_outputs_identical':True,'hashes':{n:sha(a/n) for n in checks}}
(r/'resume-verification.json').write_text(json.dumps(resume,indent=2))
root=r/'generated-rust-check';root.mkdir(exist_ok=False);results=[]
for i,row in enumerate(ja['generation']):
 if row['kind']!='rust':continue
 text=row['generated'];file=root/f'model_output_{i}.rs';file.write_text(text)
 command=['/Users/casey.allard/.cargo/bin/rustc','--edition=2021','--emit=metadata','--crate-name',f'model_output_{i}',str(file),'-o',str(root/f'output_{i}.rmeta')]
 if any(s in text for s in ['include!','include_bytes!','include_str!','env!','asm!','extern crate']):raise RuntimeError('Generated code needs explicit safety review before compiler access')
 run=subprocess.run(command,capture_output=True,text=True,timeout=20)
 (root/f'diagnostics_{i}.txt').write_text(run.stdout+run.stderr)
 results.append({'case':i,'source_sha256':sha(file),'compiles':run.returncode==0,'returncode':run.returncode,'exact_authored_answer':row['exact'],'executed':False})
positive=root/'compiler_control.rs';positive.write_text('fn main() { println!("compiler control"); }\n')
run=subprocess.run(['/Users/casey.allard/.cargo/bin/rustc','--edition=2021','--emit=metadata',str(positive),'-o',str(root/'control.rmeta')],capture_output=True,text=True,timeout=20);assert run.returncode==0,run.stderr
record={'model_outputs':results,'compiled_model_outputs':sum(x['compiles'] for x in results),'compiler_positive_control':True,'positive_control_counted_as_model':False,'method':'Compile unmodified generated Rust text to metadata only. No repair, expected-code insertion, markdown stripping, or generated program execution.'}
(r/'generated-rust-results.json').write_text(json.dumps(record,indent=2));print(json.dumps({'resume':resume,'code':record},indent=2))
