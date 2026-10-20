#!/usr/bin/env python3
"""Restore one immutable source snapshot into an exclusively created directory."""
import argparse,hashlib,json,os,pathlib,subprocess
p=argparse.ArgumentParser();p.add_argument('snapshot',help='Snapshot name or zero-based index');p.add_argument('destination');a=p.parse_args()
root=pathlib.Path(__file__).resolve().parent
manifest=json.loads((root/'manifest.json').read_text());snapshots=manifest['snapshots']
matches=[s for s in snapshots if a.snapshot in (s['name'],str(s['index']))]
if len(matches)!=1:raise SystemExit('Unknown or ambiguous snapshot')
end=matches[0]['index'];dest=pathlib.Path(a.destination).resolve();dest.mkdir(parents=False,exist_ok=False)
for s in snapshots[:end+1]:
 patch=root/s['patch'];raw=patch.read_bytes()
 if hashlib.sha256(raw).hexdigest()!=s['patch_sha256']:raise SystemExit('Patch hash mismatch')
 if raw:subprocess.run(['git','apply','--binary','--whitespace=nowarn',str(patch)],cwd=dest,env={**{k:v for k,v in os.environ.items() if not k.startswith('GIT_')},'GIT_CEILING_DIRECTORIES':str(dest.parent)},check=True)
 for d in sorted(dest.rglob('*'),key=lambda x:len(x.parts),reverse=True):
  if d.is_dir() and not any(d.iterdir()):d.rmdir()
 for d in s['explicit_directories']:
  q=dest/d['path'];q.mkdir(parents=True,exist_ok=True);q.chmod(d['mode'])
 expected={f['path']:f for f in s['files']};actual={str(q.relative_to(dest)):q for q in dest.rglob('*') if q.is_file()}
 if set(expected)!=set(actual):raise SystemExit('Restored file-set mismatch')
 for path,f in expected.items():
  q=actual[path];b=q.read_bytes()
  if len(b)!=f['bytes'] or hashlib.sha256(b).hexdigest()!=f['sha256']:raise SystemExit('Restored file hash mismatch: '+path)
  q.chmod(f['mode'])
 print(json.dumps({'snapshot':s['name'],'files':len(actual),'status':'PASS'}))
print('Restored '+matches[0]['name']+' to '+str(dest))
