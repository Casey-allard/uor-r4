#!/usr/bin/env python3
"""Supervise the source-bound Rust saved-credit constructor, without implementing a model."""
from pathlib import Path
import sys
import datetime,hashlib,json,os,resource,shutil,subprocess,time
W=Path(__file__).resolve().parents[3];R=W/'local/discrete-feedback'
B=W/'local/build/release/examples/geometric-frozen-map-fit';C=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else R/'config.json'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
config=json.loads(C.read_text());out=Path(config['out']);assert not out.exists()
assert config['coupled_episode_learning']['prefix_transaction']=='protected_discrete_feedback'
assert config['coupled_episode_learning'].get('saved_protected_credit')
assert shutil.disk_usage(W).free>(30<<30)+(128<<20)+config['maximum_report_bytes']
obs=R/'observations'/out.name;obs.mkdir(parents=True,exist_ok=False)
source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=W,text=True).strip()
files=[W/'crates/uor-r4-training/examples/geometric-frozen-map-fit.rs',*(W/'crates/uor-r4-training/examples/geometric_frozen_map_fit').glob('*.rs')]
identity=dict(source_commit=source,binary_sha256=sha(B),config_sha256=sha(C),rust_files={str(p.relative_to(W)):sha(p) for p in files})
(obs/'runtime.json').write_text(json.dumps(identity,indent=2)+'\n')
env=dict(os.environ);env.update(RAYON_NUM_THREADS='2',OMP_NUM_THREADS='2',OPENBLAS_NUM_THREADS='2',UOR_REQUIRE_CUDA='0')
started=datetime.datetime.now(datetime.timezone.utc).isoformat();start=time.monotonic()
with (obs/'execution.log').open('x') as log,(obs/'telemetry.jsonl').open('x') as tele:
 p=subprocess.Popen([str(B),str(C)],cwd=W,env=env,stdout=log,stderr=subprocess.STDOUT)
 (obs/'launch.json').write_text(json.dumps(dict(pid=p.pid,argv=[str(B),str(C)],started_utc=started,**identity),indent=2)+'\n')
 while True:
  alive=p.poll() is None;row=dict(elapsed_seconds=time.monotonic()-start,pid=p.pid,alive=alive,free_bytes=shutil.disk_usage(W).free)
  if alive:
   sample=subprocess.run(['ps','-o','rss=','-p',str(p.pid)],capture_output=True,text=True)
   if sample.stdout.strip():row['rss_bytes']=int(sample.stdout.strip())*1024
  tele.write(json.dumps(row)+'\n');tele.flush()
  if not alive:break
  if row['free_bytes']<(30<<30)+(128<<20):p.terminate();raise RuntimeError('owner disk floor reached; own constructor terminated')
  time.sleep(2)
 rc=p.wait()
receipt=dict(exit_code=rc,started_utc=started,ended_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),elapsed_seconds=time.monotonic()-start,children_max_rss_bytes=resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss,**identity)
(obs/'execution.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt));raise SystemExit(rc)
