#!/usr/bin/env python3
"""Preserve the checked executable and immutable launch inputs before fitting."""
from pathlib import Path
import hashlib,json,subprocess,shutil,time
R=Path('/workspace/uor-r4/codex/sol-bounded-pair-20261009');S=Path('/root/codex/bounded-pair-src')
SOURCE='a66ecef50848cc11160fc0c5a1f188355731ff98'
BINARY='f37b39134e865f6892c8cf0e5229376d1bcd9440ff3b6b89405b2ba73f702d63'
CONFIG='7c15dd4956770c0f008644f8ac4b99d60b6b2bf085847377d0fadbfa41c45740'
def h(p):
 q=hashlib.sha256()
 with Path(p).open('rb') as f:
  for b in iter(lambda:f.read(1<<20),b''):q.update(b)
 return q.hexdigest()
def need(v,m):
 if not v:raise ValueError(m)
def git(*a):return subprocess.check_output(['git',*a],cwd=S,text=True).strip()
start=time.monotonic();need(git('rev-parse','HEAD')==SOURCE and not git('status','--porcelain'),'checked source differs')
need(h(R/'runtime/geometric-frozen-map-fit')==BINARY,'binary differs');need(h(R/'configs/pair-0001-attempt1.json')==CONFIG,'config differs')
checks=R/'evidence/source-bound-build'
need((checks/'source.commit').read_text().strip()==SOURCE,'build source receipt')
for name,count in [('generate-tests.log',15),('prefix-tests.log',16)]:need(f'test result: ok. {count} passed; 0 failed;' in (checks/name).read_text(),'focused test result')
need('Exit status: 0' in (checks/'build.log').read_text(),'build result')
files={}
def copy(src,name):
 dst=R/'runtime'/name;need(not dst.exists(),'runtime file exists '+name);dst.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(src,dst);files[name]={'sha256':h(dst),'bytes':dst.stat().st_size}
for name in ['build.log','generate-tests.log','prefix-tests.log','source.commit','binary.sha256','source-files.sha256']:copy(checks/name,'checks/'+name)
copy(R/'configs/pair-0001-attempt1.json','pair-config.json');copy(R/'evidence/input-verification.json','input-verification.json')
for name in ['generate_episode_learning.rs','prefix_fragment_learning.rs']:
 copy(S/'crates/uor-r4-training/examples/geometric_frozen_map_fit'/name,'owned-source/'+name)
copy(R/'build-runtime-bound.sh','build-runtime.sh')
copy(Path('/workspace/uor-r4/codex/sol-sequence-progress-causal/capture-supervise.py'),'capture-supervise.py')
files['geometric-frozen-map-fit']={'sha256':BINARY,'bytes':(R/'runtime/geometric-frozen-map-fit').stat().st_size}
identity={'source_commit':SOURCE,'source_tree':git('rev-parse','HEAD^{tree}'),'clean_build_checkout':True,'binary_sha256':BINARY,'config_sha256':CONFIG,'family':'pair','maximum_coordinates':960,'cuda_compute_cap':120,'focused_tests':31,'input_roots_verified':14,'source_persistence':'Public protected main contains complete source at pinned commit; runtime stores both mechanism source files.','actual_model_fit':'NOT_RUN','files':files}
(R/'runtime/runtime-identity.json').write_text(json.dumps(identity,indent=2)+'\n')
from huggingface_hub import HfApi,hf_hub_download
prefix='codex/sol-bounded-pair-20261009/runtime-a66ecef50-attempt1';repo='caseyallard/uor-r4-store'
need(not (R/'runtime-upload.json').exists(),'upload receipt exists')
commit=HfApi().upload_folder(repo_id=repo,repo_type='dataset',folder_path=str(R/'runtime'),path_in_repo=prefix,commit_message='Preserve checked bounded pair runtime and original-input identities before model execution')
fresh=R/'runtime-redownload';need(not fresh.exists(),'fresh verification directory exists')
for p in (R/'runtime').rglob('*'):
 if not p.is_file():continue
 rel=str(p.relative_to(R/'runtime'))
 q=Path(hf_hub_download(repo,prefix+'/'+rel,repo_type='dataset',revision=commit.oid,local_dir=fresh,force_download=True))
 need(h(p)==h(q) and p.stat().st_size==q.stat().st_size,'fresh runtime roundtrip '+rel)
receipt={'status':'PASS','revision':commit.oid,'prefix':prefix,'all_fresh_download_verified':True,'source_commit':SOURCE,'binary_sha256':BINARY,'config_sha256':CONFIG,'payload_bytes':sum(p.stat().st_size for p in (R/'runtime').rglob('*') if p.is_file()),'seconds':time.monotonic()-start}
(R/'runtime-upload.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt),flush=True)
