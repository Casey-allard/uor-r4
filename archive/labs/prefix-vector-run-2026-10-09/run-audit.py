"""Normatively verify and seal one reviewed saved-data comparison, after qualification."""
from pathlib import Path
import json,hashlib,subprocess,time,resource
R=Path('/workspace/uor-r4/codex/sol-prefix-vector-20261009')
V=Path('/workspace/uor-r4/codex/sol-intermediate-word-boundary/publications/runtime-76e10b53f7-attempt2/native-reached-prefix-attribution')
CLAIM=Path('/workspace/uor-r4/codex/sol-prefix-margin-boundary/claim-report')
READER=R/'compare-vector.py';A=R/'audits/prefix-vector-0001-attempt1';M=R/'runs/prefix-vector-0001-attempt1'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text())
assert sha(READER)=='f536e8b5b4612ce52edfbea0e6749b42feb89fdb09f34ca04d5fa406bd4d5720'
assert sha(CLAIM)=='394ad004859ae8d9baaf04510f230fb725bbc2f7caabe3c7abe26b1cf70ea208'
assert sha(V)=='d90411116c702dc4149fc055706f34c276c1ead29c5b0ec95600c30cc976326d'
assert not A.exists();subprocess.run([str(CLAIM),str(A)],check=True)
start=time.monotonic();rc=None;error=None
try:
 result=read(R/'execution-result.json');assert result['status']=='MODEL_AND_CONDITIONAL_QUALIFICATION_FINISHED'
 roots=[M,R/'observations/prefix-vector-0001-attempt1',Path('/workspace/uor-r4/codex/sol-full-donor-20261009/runs/full-donor-0001-attempt1')]
 if read(M/'report.json')['final_gate']['passed']:
  assert result['actual9']=='EXECUTED_SEPARATE_TYPED_REPORT'
  roots += [R/'qualifications/actual9-0001-attempt1',R/'observations/actual9-0001-attempt1']
 else:assert result['actual9']=='NOT_RUN_CONSTRUCTION_NEGATIVE'
 checks=[]
 for root in roots:
  c=subprocess.run([str(V),'verify-report',str(root)],capture_output=True,text=True);checks.append({'root':str(root),'exit_code':c.returncode,'stdout':c.stdout,'stderr':c.stderr});assert c.returncode==0
 # Reader requires only attempt.json on entry; retain verification outside its claimed output until completion.
 config=R/'configs/prefix-vector-0001-attempt1.json'
 args=['python3',str(READER),'--config',str(config),'--config-sha256',sha(config),'--candidate-report-sha256',sha(M/'report.json'),'--candidate-manifest-sha256',sha(M/'manifest.json'),'--output',str(A)]
 process=subprocess.run(args,capture_output=True,text=True);rc=process.returncode
 (A/'stdout.log').write_text(process.stdout);(A/'stderr.log').write_text(process.stderr)
 (A/'normative-verification.json').write_text(json.dumps(checks,indent=2)+'\n')
 assert rc==0,process.stderr
 assert read(A/'report.json')['status']=='PASS_SAVED_COMPARISON'
except BaseException as e:
 error=str(e);raise
finally:
 (A/'supervised-execution.json').write_text(json.dumps({'exit_code':rc,'error':error,'seconds':time.monotonic()-start,'child_max_rss_bytes':resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss*1024,'reader_sha256':sha(READER)},indent=2)+'\n')
 subprocess.run([str(R/'runtime/native_historical_version'),'seal',str(A)],check=True)
 subprocess.run([str(V),'verify-report',str(A)],check=True)
print('PASS_SAVED_COMPARISON',flush=True)
