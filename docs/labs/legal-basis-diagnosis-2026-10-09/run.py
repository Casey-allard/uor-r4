#!/usr/bin/env python3
"""Execute one exact saved-input diagnosis and retain outside-attempt process receipt."""
from pathlib import Path
import datetime,hashlib,json,os,resource,shutil,subprocess,time,sys
w=Path(__file__).resolve().parents[3]
binary=w/'local/target/release/legal-basis-diagnosis'
attempt=int(sys.argv[1]) if len(sys.argv)==2 else 1
assert attempt in [1,2]
input=w/'local/input/input.json';out=w/f'local/replay-{attempt:04}'
suffix='' if attempt==1 else '-0002'
receipt=w/f'local/replay{suffix}-process.json'
assert not out.exists() and not receipt.exists()
assert shutil.disk_usage(w).free>(30<<30)+(128<<20)+(512<<20)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
expected='0fe91060f10c0d6107be3565f9ae0746c30e742a00d02935398f6764b84b974d'
assert sha(input)==expected
started=datetime.datetime.now(datetime.timezone.utc).isoformat();t=time.monotonic()
cmd=[str(binary),'--input',str(input),'--expected-sha256',expected,'--output',str(out)]
identity={'started_utc':started,'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=w,text=True).strip(),'binary_sha256':sha(binary),'input_sha256':expected,'command':cmd,'threads':2,'no_wall_cutoff':True}
(w/f'local/replay{suffix}-admission.json').write_text(json.dumps(identity,indent=2)+'\n')
with (w/f'local/replay{suffix}-stdout.log').open('x') as stdout,(w/f'local/replay{suffix}-stderr.log').open('x') as stderr:
 p=subprocess.run(cmd,cwd=w,stdout=stdout,stderr=stderr,env={**os.environ,'RAYON_NUM_THREADS':'2'})
r={**identity,'elapsed_seconds':time.monotonic()-t,'peak_child_rss_bytes':resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss,'exit_code':p.returncode,'free_bytes_after':shutil.disk_usage(w).free,'new_backward_calls':0,'native_score_calls':0}
receipt.write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))
raise SystemExit(p.returncode)
