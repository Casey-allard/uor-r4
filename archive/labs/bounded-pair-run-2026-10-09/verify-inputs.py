#!/usr/bin/env python3
"""Verify retained authorities and prepare the prospective pair config; no fitting."""
import hashlib,json,pathlib,subprocess,time
R=pathlib.Path('/workspace/uor-r4/codex/sol-bounded-pair-20261009')
def h(p):
 q=hashlib.sha256()
 with pathlib.Path(p).open('rb') as f:
  for b in iter(lambda:f.read(1<<20),b''):q.update(b)
 return q.hexdigest()
def require(v,s):
 if not v:raise ValueError(s)
old=R/'evidence/original-generate-config.json'
require(h(old)=='f504d1eee00abc8517e19cbb2c2e2e067e5529820d549ff3fb0c71e9fdea4120','original config identity')
a=json.loads(old.read_text());g=a['generate_episode_learning'];o=g['original_inputs'];e=o['episode']
verifier=pathlib.Path('/workspace/uor-r4/codex/sol-intermediate-word-boundary/publications/runtime-76e10b53f7-attempt2/native-reached-prefix-attribution')
require(h(verifier)=='d90411116c702dc4149fc055706f34c276c1ead29c5b0ec95600c30cc976326d','normative verifier identity')
roots={}
def add(root,report,seal):
 tup=(report,seal)
 require(root not in roots or roots[root]==tup,'conflicting root pins')
 roots[root]=tup
add(g['retained_episode_root'],g['expected_report_sha256'],g['expected_manifest_sha256'])
add(o['retained_intermediate_root'],'c9b9fe10b6fbb4332cf919a5df7ba31403ad3d51ac6f877d94daeabd99672bee','de90ba0ba2809ed37ca868ef2bcf94b6ceb8b176a60b86ae88801876dafbd0e5')
add(o['retained_probe_root'],'1f7a51fe58e56862f6e8cd269225445bf9d42354d3cfe7eaf96a5910707568c9','c43bbbe81eeea18338b2ea541c50f8f01d24dcfbe9a32662b73e2d2ebc03b332')
add(e['retained_supplement_root'],e['expected_supplement_report_sha256'],e['expected_supplement_manifest_sha256'])
for c in [e['retained_projection']]+[p['capture'] for p in e['phases']]:add(c['root'],c['expected_report_sha256'],c['expected_manifest_sha256'])
require(h(e['typed_authority'])==e['expected_typed_authority_sha256'],'typed authority')
receipt={'roots':[], 'old_config_sha256':h(old),'verifier_sha256':h(verifier)}
start=time.monotonic()
for root,(report,seal) in roots.items():
 p=pathlib.Path(root)
 require(h(p/'report.json')==report,'report '+root)
 require(h(p/'manifest.json')==seal,'manifest '+root)
 run=subprocess.run([str(verifier),'verify-report',root],capture_output=True,text=True,check=True)
 m=json.loads((p/'manifest.json').read_text())
 receipt['roots'].append({'root':root,'report_sha256':report,'manifest_sha256':seal,'files':len(m['files']),'bytes':sum(v['bytes'] for v in m['files']),'complete_set_verified':True,'verification_stdout':run.stdout.strip()})
 print('verified',root,flush=True)
retained=json.loads((pathlib.Path(g['retained_episode_root'])/'config.json').read_text())
require(retained['prefix_fragment_learning']==o,'typed original inputs/config path equality')
for key in ['training_inputs','training_labels','development_inputs','development_labels','parent_config']:
 p=pathlib.Path(a[key]);require(p.is_file(),key);receipt[key]={'file':str(p),'sha256':h(p),'bytes':p.stat().st_size}
for key in ['checkpoint','categorical','saved_fit']:require(pathlib.Path(a[key]).is_dir(),key)
a['out']=str(R/'runs/pair-0001-attempt1')
a['generate_episode_learning']['family']='pair';a['generate_episode_learning']['maximum_coordinates']=960
require(not pathlib.Path(a['out']).exists(),'fresh run root')
out=R/'configs/pair-0001-attempt1.json'
with out.open('x') as f:json.dump(a,f,indent=2);f.write('\n')
receipt.update(status='PASS',seconds=time.monotonic()-start,config_sha256=h(out),config=str(out),scope='retained root complete-set integrity and configuration admission; no model execution')
with (R/'evidence/input-verification.json').open('x') as f:json.dump(receipt,f,indent=2);f.write('\n')
print(json.dumps({'status':'PASS','roots':len(roots),'seconds':receipt['seconds'],'config_sha256':h(out)}),flush=True)
