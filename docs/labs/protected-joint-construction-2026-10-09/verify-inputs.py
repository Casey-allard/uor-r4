#!/usr/bin/env python3
"""Verify saved complete-file authorities and prepare the one prospective config; no model."""
from pathlib import Path
import hashlib,json,subprocess,time
R=Path('/workspace/uor-r4/codex/sol-protected-joint-20261009')
SOURCE=json.loads((R/'source.json').read_text())['commit']
S=Path(json.loads((R/'source.json').read_text())['build_path'])
OLD=S/'docs/labs/full-donor-run-2026-10-09/model-config.json'
VERIFIER=Path('/workspace/uor-r4/codex/sol-intermediate-word-boundary/publications/runtime-76e10b53f7-attempt2/native-reached-prefix-attribution')
def h(p):
 d=hashlib.sha256()
 with Path(p).open('rb') as f:
  for b in iter(lambda:f.read(1<<20),b''):d.update(b)
 return d.hexdigest()
def read(p):return json.loads(Path(p).read_text())
def need(v,m):
 if not v:raise ValueError(m)
def write(p,v):
 with p.open('x') as f:json.dump(v,f,indent=2);f.write('\n')
start=time.monotonic()
need(h(OLD)=='f778b5affdb17e9da1c29ce517f280357e9333082618c34eee06a0015b7f6692','historical fresh coupled config identity')
need(h(VERIFIER)=='d90411116c702dc4149fc055706f34c276c1ead29c5b0ec95600c30cc976326d','normative verifier identity')
a=read(OLD);need(set(a['coupled_episode_learning'])=={'original_inputs','donor_credit'},'reference config was not fresh')
reference=read(S/'docs/labs/bounded-pair-run-2026-10-09/input-verification.json')
pair=read(S/'docs/labs/bounded-pair-run-2026-10-09/pair-config.json')
need(a['coupled_episode_learning']['original_inputs']==pair['generate_episode_learning']['original_inputs'],'fixed objective differs')
receipt={'roots':[], 'original_config_sha256':h(OLD),'verifier_sha256':h(VERIFIER),'learning_policy_change':'vs #2069 only prefix_transaction: protected_joint_vector and fresh output path/report cap512MiB; all other config values identical'}
roots=[(x['root'],x['report_sha256'],x['manifest_sha256']) for x in reference['roots']]
baseline=read(S/'docs/labs/full-donor-run-2026-10-09/model-config.json')['out']
roots.append((baseline,'56faa224da29b5da1db6fe82e21c4d19b6474be1316fef7fcb4716013cdd01a8','9c19b82392b9e8f2263257992bf321ef5ed939e3b6e7788b783aa21328e45726'))
for root,report,manifest in roots:
 p=Path(root);need(h(p/'report.json')==report and h(p/'manifest.json')==manifest,'pinned root '+root)
 subprocess.run([str(VERIFIER),'verify-report',root],check=True)
 m=read(p/'manifest.json');receipt['roots'].append({'root':root,'report_sha256':report,'manifest_sha256':manifest,'files':len(m['files']),'bytes':sum(x['bytes'] for x in m['files']),'complete_set_verified':True})
 print('verified',root,flush=True)
for key in ['training_inputs','training_labels','development_inputs','development_labels','parent_config']:
 p=Path(a[key]);expected=reference[key];need(str(p)==expected['file'] and h(p)==expected['sha256'] and p.stat().st_size==expected['bytes'],'original file '+key);receipt[key]=expected
for key in ['checkpoint','categorical','saved_fit']:need(a[key]==pair[key] and Path(a[key]).is_dir(),'original parent '+key)
e=a['coupled_episode_learning']['original_inputs']['episode'];need(h(Path(e['typed_authority']))==e['expected_typed_authority_sha256'],'typed episode authority')
a['coupled_episode_learning']['donor_credit']='full_pool_utility';a['coupled_episode_learning']['prefix_transaction']='protected_joint_vector';a['out']=str(R/'runs/protected-joint-0001-attempt3');a['maximum_report_bytes']=512<<20
need(not Path(a['out']).exists(),'exclusive fresh model root')
reference_config=read(S/'docs/labs/full-donor-run-2026-10-09/model-config.json')
comparison=json.loads(json.dumps(a));comparison['out']=reference_config['out'];comparison['maximum_report_bytes']=reference_config['maximum_report_bytes'];comparison['coupled_episode_learning'].pop('prefix_transaction')
need(comparison==reference_config,'unexpected config delta beyond joint policy/out/report cap')
config=R/'configs/protected-joint-0001-attempt3.json';write(config,a)
receipt.update(status='PASS',seconds=time.monotonic()-start,config=str(config),config_sha256=h(config),source=SOURCE,scope='fourteen original sealed roots plus corrected donor comparison root; no backward or model execution')
write(R/'evidence/input-verification.json',receipt);print(json.dumps({'status':'PASS','roots':len(roots),'config_sha256':h(config),'seconds':receipt['seconds']}),flush=True)
