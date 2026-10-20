#!/usr/bin/env python3
"""Normative verification and supervised independent saved arithmetic only."""
from pathlib import Path
import datetime,hashlib,json,subprocess,time,resource,os
R=Path('/workspace/uor-r4/codex/sol-bounded-pair-20261009')
MODEL=R/'runs/pair-0001-attempt1';OBS=R/'observations/pair-0001-attempt1'
READER=R/'pair-saved-audit.py';AUDIT=R/'audits/pair-0001-attempt1'
VERIFIER=Path('/workspace/uor-r4/codex/sol-intermediate-word-boundary/publications/runtime-76e10b53f7-attempt2/native-reached-prefix-attribution')
CLAIM=Path('/workspace/uor-r4/codex/sol-prefix-margin-boundary/claim-report')
SEAL=Path('/root/codex/prototype-target/release/examples/native_historical_version')
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_bytes())
def need(v,m):
 if not v:raise ValueError(m)
need(h(READER)=='f757f7c809eac385eadff7b0141eb13306f2aac92c03f2f9e6a47de6809764b1','reviewed reader changed')
need(h(VERIFIER)=='d90411116c702dc4149fc055706f34c276c1ead29c5b0ec95600c30cc976326d','verifier changed')
need(h(CLAIM)=='394ad004859ae8d9baaf04510f230fb725bbc2f7caabe3c7abe26b1cf70ea208','claim tool changed')
need(h(SEAL)=='2691e33e5b1ce9c29b12ddbf5e30a3d857d30185d69db49402ed1a53f7c4320a','seal tool changed')
need(not AUDIT.exists(),'fresh audit root')
subprocess.run([str(CLAIM),str(AUDIT)],check=True)
start=time.monotonic();started=datetime.datetime.now(datetime.timezone.utc).isoformat();rc=1
try:
 roots=[MODEL,OBS];report=read(MODEL/'report.json');need(report['status']=='COMPLETED','terminal successful model execution')
 argv=['python3',str(READER),str(MODEL),'--config',str(R/'configs/pair-0001-attempt1.json'),'--config-sha256','7c15dd4956770c0f008644f8ac4b99d60b6b2bf085847377d0fadbfa41c45740','--binary',str(R/'runtime/geometric-frozen-map-fit'),'--binary-sha256','f37b39134e865f6892c8cf0e5229376d1bcd9440ff3b6b89405b2ba73f702d63','--expected-source','a66ecef50848cc11160fc0c5a1f188355731ff98','--execution',str(OBS/'execution.json'),'--source-file',str(R/'runtime/owned-source/generate_episode_learning.rs'),'--source-sha256','30ac874b799a41cbfc57893944ac82cf7ac0c4a2681d2d95747da08017b2c769']
 if report['finite_episode_positive']:
  q=R/'qualifications/actual9-0001-attempt1';roots += [q,R/'observations/actual9-0001-attempt1']
  argv += ['--qualification',str(q),'--qualification-report-sha256',h(q/'report.json'),'--qualification-manifest-sha256',h(q/'manifest.json')]
 verification=[]
 for root in roots:
  z=subprocess.run([str(VERIFIER),'verify-report',str(root)],capture_output=True,text=True,check=True);verification.append({'root':str(root),'manifest_sha256':h(root/'manifest.json'),'stdout':z.stdout,'exit_code':z.returncode})
 (AUDIT/'normative-verification.json').write_text(json.dumps(verification,indent=2)+'\n')
 with (AUDIT/'result.json').open('x') as out,(AUDIT/'stderr.log').open('x') as err,(AUDIT/'telemetry.jsonl').open('x') as tele:
  p=subprocess.Popen(argv,stdout=out,stderr=err,preexec_fn=lambda:os.sched_setaffinity(0,{0,1}))
  (AUDIT/'launch.json').write_text(json.dumps({'argv':argv,'pid':p.pid,'started_utc':started,'reader_sha256':h(READER),'scope':'saved arithmetic only, no model/backward/proposals'},indent=2)+'\n')
  while True:
   alive=p.poll() is None;row={'pid':p.pid,'alive':alive,'seconds':time.monotonic()-start}
   try:
    status=Path(f'/proc/{p.pid}/status').read_text();row.update({k:int(next(x.split()[1] for x in status.splitlines() if x.startswith(k+':'))) for k in ['VmRSS','VmHWM']})
   except (FileNotFoundError,StopIteration):pass
   tele.write(json.dumps(row)+'\n');tele.flush()
   if not alive:break
   time.sleep(10)
  rc=p.wait()
 (AUDIT/'execution.json').write_text(json.dumps({'exit_code':rc,'started_utc':started,'seconds':time.monotonic()-start,'children_max_rss_kib':resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss},indent=2)+'\n')
 if rc!=0:raise RuntimeError('independent saved audit failed; retain diagnostic root')
 need(read(AUDIT/'result.json')['status']=='PASS','audit returned no PASS')
finally:
 subprocess.run([str(SEAL),'seal',str(AUDIT)],check=True)
 subprocess.run([str(VERIFIER),'verify-report',str(AUDIT)],check=True)
print(json.dumps({'audit':str(AUDIT),'exit_code':rc,'seconds':time.monotonic()-start}),flush=True)
