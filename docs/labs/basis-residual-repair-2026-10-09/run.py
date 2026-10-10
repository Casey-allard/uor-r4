#!/usr/bin/env python3
"""Run a source-bound saved-basis qualification or gated saved constructor."""
from pathlib import Path
import datetime,hashlib,json,os,resource,shutil,subprocess,sys,time
w=Path(__file__).resolve().parents[3]
kind=sys.argv[1] if len(sys.argv)==2 else ''
assert kind in ('basis1','basis2','constructor')
binary=w/'local/target/release/basis-residual-repair'
old=w/'local/restore/codex-legal-basis-diagnosis-20261009'
bases=['c5c73e1b9f7baf598cdda33e4a458f9208f0c943b4d471ba646eb129c495a794','55d8c2eadbcfc072d86a6310602184603030a318986ebeb2a2d885958f10ae05']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
if kind=='constructor':
 for i in (1,2):
  r=json.loads((w/f'local/qualification-basis{i}/report.json').read_text())
  assert r['status']=='QUALIFIED'
  admission=json.loads((w/f'local/receipts/basis{i}-process.json').read_text())
  root=w/f'local/qualification-basis{i}'
  subprocess.run([str(binary),'--verify-report',str(root)],check=True)
  assert r['basis_sha256']==bases[i-1] and admission['input_sha256']==bases[i-1]
  assert admission['binary_sha256']==sha(binary) and admission['exit_code']==0
  assert admission['report_sha256']==sha(root/'report.json')
  assert admission['manifest_sha256']==sha(root/'manifest.json')
  assert r['verification']['dimension']==440 and r['verification']['rhs_checked_per_direction']==440
  assert admission['command'][1]=='--basis'
 input=w/'local/input/input.json';expected='0fe91060f10c0d6107be3565f9ae0746c30e742a00d02935398f6764b84b974d';mode='--input'
else:
 i=int(kind[-1]);input=old/f'replay-0002/backend-observations/singular-{i:04}/basis-csc.tsv';expected=bases[i-1];mode='--basis'
out=w/f'local/qualification-{kind}'
receipt=w/f'local/receipts/{kind}-process.json'
assert not out.exists() and not receipt.exists()
assert sha(input)==expected
assert shutil.disk_usage(w).free>(30<<30)+(128<<20)+(512<<20)
cmd=[str(binary),mode,str(input),'--expected-sha256',expected,'--output',str(out)]
start=time.monotonic()
identity={'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=w,text=True).strip(),'binary_sha256':sha(binary),'input_sha256':expected,'command':cmd,'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'threads':2,'no_wall_cutoff':True}
(w/f'local/receipts/{kind}-admission.json').write_text(json.dumps(identity,indent=2)+'\n')
with (w/f'local/receipts/{kind}-stdout.log').open('x') as stdout,(w/f'local/receipts/{kind}-stderr.log').open('x') as stderr:
 p=subprocess.Popen(cmd,cwd=w,stdout=stdout,stderr=stderr,env={**os.environ,'RAYON_NUM_THREADS':'2'})
 stop=None
 while p.poll() is None:
  if shutil.disk_usage(w).free<(30<<30)+(128<<20):
   stop='OWNER_DISK_FLOOR';p.terminate();break
  time.sleep(1)
 rc=p.wait()
r={**identity,'elapsed_seconds':time.monotonic()-start,'peak_child_rss_bytes':resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss,'exit_code':rc,'stop_reason':stop,'free_bytes_after':shutil.disk_usage(w).free,'new_backward_calls':0,'native_score_calls':0}
if rc==0:
 r.update(report_sha256=sha(out/'report.json'),manifest_sha256=sha(out/'manifest.json'))
receipt.write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))
raise SystemExit(rc if rc else (1 if stop else 0))
