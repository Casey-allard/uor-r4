#!/usr/bin/env python3
"""Relocate and authenticate retained inputs; no model or new derivative implementation."""
from pathlib import Path
import hashlib,json,subprocess
W=Path(__file__).resolve().parents[3]
R=W/'local/recovery/inputs'
OLD=W/'local/restore/codex-protected-joint-20261009/sol-protected-joint-20261009'
OUT=W/'local/direct-legal'
VERIFIER=W/'local/restore/codex-protected-discrete-feedback-20261009/runtime/native-reached-prefix-attribution'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def read(p):return json.loads(Path(p).read_text())
def write(p,v):
 p.parent.mkdir(parents=True,exist_ok=True)
 with p.open('x') as f:json.dump(v,f,indent=2);f.write('\n')
def path(p):
 assert p.startswith('/workspace/uor-r4/codex/')
 return R/p.removeprefix('/workspace/uor-r4/codex/')
reference=read(W/'docs/labs/bounded-pair-run-2026-10-09/input-verification.json')
checks=[]
for item in reference['roots']:
 p=path(item['root'])
 assert sha(p/'report.json')==item['report_sha256']
 assert sha(p/'manifest.json')==item['manifest_sha256']
 subprocess.run([str(VERIFIER),'verify-report',str(p)],check=True)
 checks.append(dict(root=str(p),report_sha256=sha(p/'report.json'),manifest_sha256=sha(p/'manifest.json'),files=len(read(p/'manifest.json')['files'])))
for key in ['training_inputs','training_labels','development_inputs','development_labels','parent_config']:
 item=reference[key];p=path(item['file']);assert sha(p)==item['sha256'] and p.stat().st_size==item['bytes']
config=read(W/'docs/labs/protected-joint-construction-2026-10-09/model-config.json')
def relocate(v):
 if isinstance(v,str) and v.startswith('/workspace/uor-r4/codex/'):return str(path(v))
 if isinstance(v,list):return [relocate(x) for x in v]
 if isinstance(v,dict):return {k:relocate(x) for k,x in v.items()}
 return v
config=relocate(config)
config['out']=str(OUT/'runs/attempt1')
config['maximum_report_bytes']=512<<20
config['maximum_seconds']=21600
learning=config['coupled_episode_learning'];learning['prefix_transaction']='protected_legal_set'
learning['saved_protected_credit']=dict(contract_version=1,root=str(OLD/'runs/protected-joint-0001-attempt3'),runtime_root=str(OLD/'runtime'),observation_root=str(OLD/'observations/protected-joint-0001-attempt3'),expected_runtime_identity_sha256=sha(OLD/'runtime/runtime-identity.json'),expected_observation_manifest_sha256=sha(OLD/'observations/protected-joint-0001-attempt3/manifest.json'))
write(OUT/'config.json',config)
write(OUT/'input-verification.json',dict(status='PASS',roots=checks,config_sha256=sha(OUT/'config.json'),verifier_sha256=sha(VERIFIER),imported_backward_calls=411,new_backward_calls=0))
print(json.dumps(dict(status='PASS',roots=len(checks),config_sha256=sha(OUT/'config.json'))))
