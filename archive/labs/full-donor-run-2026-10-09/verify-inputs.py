#!/usr/bin/env python3
"""Verify saved complete-file authorities and prepare the one prospective config; no model."""
from pathlib import Path
import hashlib,json,subprocess,time
R=Path('/workspace/uor-r4/codex/sol-full-donor-20261009')
S=Path('/root/codex/full-donor-src')
OLD=Path('/workspace/uor-r4/codex/sol-coupled-episode-learning/configs/coupled-episode-0001-attempt2.json')
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
need(h(OLD)=='4c33cf1bb35f5bc984f47a224d2c07ef1ba56393c7b2732ea48439baf56c2497','historical fresh coupled config identity')
need(h(VERIFIER)=='d90411116c702dc4149fc055706f34c276c1ead29c5b0ec95600c30cc976326d','normative verifier identity')
a=read(OLD);need(list(a['coupled_episode_learning'])==['original_inputs'],'original config was not fresh')
reference=read(S/'docs/labs/bounded-pair-run-2026-10-09/input-verification.json')
pair=read(S/'docs/labs/bounded-pair-run-2026-10-09/pair-config.json')
need(a['coupled_episode_learning']['original_inputs']==pair['generate_episode_learning']['original_inputs'],'fixed objective differs')
receipt={'roots':[], 'original_config_sha256':h(OLD),'verifier_sha256':h(VERIFIER),'learning_policy_change':'donor_credit: full_pool_utility only; out is a fresh path'}
roots=[(x['root'],x['report_sha256'],x['manifest_sha256']) for x in reference['roots']]
pin=read(S/'docs/labs/coupled-export-recovery-2026-10-09/implementation-packet.json')['retained_gradient_restart']['configuration']['coupled_episode_learning']['retained_gradient']
roots.append((pin['retained_failed_root'],pin['expected_report_sha256'],pin['expected_manifest_sha256']))
legacy=read(S/'docs/labs/coupled-export-recovery-2026-10-09/model-rust-verification.json')
roots.append((legacy['checks'][0]['argv'][-1],legacy['report_sha256'],legacy['manifest_sha256']))
for root,report,manifest in roots:
 p=Path(root);need(h(p/'report.json')==report and h(p/'manifest.json')==manifest,'pinned root '+root)
 subprocess.run([str(VERIFIER),'verify-report',root],check=True)
 m=read(p/'manifest.json');receipt['roots'].append({'root':root,'report_sha256':report,'manifest_sha256':manifest,'files':len(m['files']),'bytes':sum(x['bytes'] for x in m['files']),'complete_set_verified':True})
 print('verified',root,flush=True)
for key in ['training_inputs','training_labels','development_inputs','development_labels','parent_config']:
 p=Path(a[key]);expected=reference[key];need(str(p)==expected['file'] and h(p)==expected['sha256'] and p.stat().st_size==expected['bytes'],'original file '+key);receipt[key]=expected
for key in ['checkpoint','categorical','saved_fit']:need(a[key]==pair[key] and Path(a[key]).is_dir(),'original parent '+key)
e=a['coupled_episode_learning']['original_inputs']['episode'];need(h(Path(e['typed_authority']))==e['expected_typed_authority_sha256'],'typed episode authority')
a['coupled_episode_learning']['donor_credit']='full_pool_utility';a['out']=str(R/'runs/full-donor-0001-attempt1')
need(not Path(a['out']).exists(),'exclusive fresh model root')
config=R/'configs/full-donor-0001-attempt1.json';write(config,a)
receipt.update(status='PASS',seconds=time.monotonic()-start,config=str(config),config_sha256=h(config),source='4f7eee35b250b6d5bf9e250fc0ea00d356998695',scope='fourteen original sealed roots plus historical gradient and completed negative roots; no backward or model execution')
write(R/'evidence/input-verification.json',receipt);print(json.dumps({'status':'PASS','roots':len(roots),'config_sha256':h(config),'seconds':receipt['seconds']}),flush=True)
