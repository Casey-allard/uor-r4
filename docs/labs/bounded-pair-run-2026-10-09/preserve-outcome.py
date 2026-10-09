#!/usr/bin/env python3
"""Package terminal evidence, verify every archive member, and cold-store it."""
from pathlib import Path
import hashlib,json,tarfile,time,subprocess,shutil
R=Path('/workspace/uor-r4/codex/sol-bounded-pair-20261009')
V=Path('/workspace/uor-r4/codex/sol-intermediate-word-boundary/publications/runtime-76e10b53f7-attempt2/native-reached-prefix-attribution')
def h(p):
 q=hashlib.sha256()
 with Path(p).open('rb') as f:
  for b in iter(lambda:f.read(1<<20),b''):q.update(b)
 return q.hexdigest()
def need(v,m):
 if not v:raise ValueError(m)
def read(p):return json.loads(p.read_bytes())
start=time.monotonic();need(h(V)=='d90411116c702dc4149fc055706f34c276c1ead29c5b0ec95600c30cc976326d','verifier')
roots=[R/'runs/pair-0001-attempt1',R/'observations/pair-0001-attempt1',R/'audits/pair-0001-attempt1']
need(read(roots[0]/'report.json')['status']=='COMPLETED','completed model');need(read(roots[2]/'result.json')['status']=='PASS' and read(roots[2]/'execution.json')['exit_code']==0,'completed independent audit')
if read(roots[0]/'report.json')['finite_episode_positive']:roots += [R/'qualifications/actual9-0001-attempt1',R/'observations/actual9-0001-attempt1']
for root in roots:subprocess.run([str(V),'verify-report',str(root)],check=True,capture_output=True)
leaves=['runs','observations','audits','configs','runtime','evidence','runtime-upload.json','execution-admission.json','execution-result.json','build-runtime.sh','build-runtime-bound.sh','verify-inputs.py','preserve-runtime.py','prepare-pair-actual9.py','run-pair.py','pair-saved-audit.py','run-audit.py','preserve-outcome.py']
if (R/'qualifications').exists():leaves.append('qualifications')
files={}
for leaf in leaves:
 p=R/leaf;need(p.exists(),'missing publication component '+leaf)
 for q in sorted(p.rglob('*')) if p.is_dir() else [p]:
  if q.is_file():files[str(q.relative_to(R))]={'sha256':h(q),'bytes':q.stat().st_size}
need(sum(x['bytes'] for x in files.values())<1<<30,'retained package cap')
P=R/'publication-pair-0001-attempt1';P.mkdir()
a=P/'outcome.tar.gz'
with tarfile.open(a,'w:gz',compresslevel=6) as tar:
 for name in sorted(files):tar.add(R/name,arcname=name,recursive=False)
seen=set()
with tarfile.open(a,'r:gz') as tar:
 for m in tar:
  need(m.isfile() and m.name in files and m.name not in seen,'archive exact member');seen.add(m.name)
  f=tar.extractfile(m);q=hashlib.sha256()
  for b in iter(lambda:f.read(1<<20),b''):q.update(b)
  need(q.hexdigest()==files[m.name]['sha256'] and m.size==files[m.name]['bytes'],'archive member digest')
need(seen==set(files),'archive full inventory')
identity={'schema':'uor-r4.bounded-pair-outcome/1','files':files,'files_count':len(files),'uncompressed_bytes':sum(v['bytes'] for v in files.values()),'archive_sha256':h(a),'archive_bytes':a.stat().st_size,'complete_member_inventory_verified':True,'source_commit':'a66ecef50848cc11160fc0c5a1f188355731ff98','run_report_sha256':h(roots[0]/'report.json'),'run_manifest_sha256':h(roots[0]/'manifest.json'),'audit_result_sha256':h(roots[2]/'result.json'),'audit_manifest_sha256':h(roots[2]/'manifest.json'),'selected_model':False}
(P/'identity.json').write_text(json.dumps(identity,indent=2)+'\n')
from huggingface_hub import HfApi,hf_hub_download
repo='caseyallard/uor-r4-store';prefix='codex/sol-bounded-pair-20261009/outcome-pair-0001-attempt1'
commit=HfApi().upload_folder(repo_id=repo,repo_type='dataset',folder_path=str(P),path_in_repo=prefix,commit_message='Preserve bounded pair negative, full checkpoint, independent audit and runtime')
fresh=R/'outcome-redownload';need(not fresh.exists(),'fresh remote verification directory')
for p in P.iterdir():
 q=Path(hf_hub_download(repo,prefix+'/'+p.name,repo_type='dataset',revision=commit.oid,local_dir=fresh,force_download=True))
 need(h(q)==h(p) and q.stat().st_size==p.stat().st_size,'cold roundtrip '+p.name)
receipt={'status':'PASS','revision':commit.oid,'prefix':prefix,'all_fresh_download_verified':True,'full_archive_inventory_verified':True,'files':{p.name:{'sha256':h(p),'bytes':p.stat().st_size} for p in P.iterdir()},'uncompressed_bytes':identity['uncompressed_bytes'],'archived_files':len(files),'seconds':time.monotonic()-start,'scope':'Terminal negative retained; no whole-answer promotion. Source on main, runtime/outcome private store; iCloud copy next.'}
(R/'outcome-upload.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt),flush=True)
