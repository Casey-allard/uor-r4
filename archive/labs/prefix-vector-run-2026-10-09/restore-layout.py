"""Restore only required saved roots, with exclusive file writes and path validation."""
from pathlib import Path,PurePosixPath
import json,tarfile,shutil,hashlib
R=Path('/workspace/uor-r4/codex/sol-prefix-vector-20261009/recovery');base=Path('/workspace/uor-r4/codex')
layout={'intermediate':('sol-readout-intermediate-candidate',''),'prefix-credit':('sol-prefix-context-credit',''),'episode':('sol-episode-progression-learning','sol-episode-progression-learning/'),'joint':('sol-joint-fragment-learning',''),'supplement':('sol-prefix-trajectory-learning',''),'sequence':('sol-sequence-progress-causal',''),'baseline':('sol-full-donor-20261009','')}
receipt=[]
for x in json.loads((R/'download-inventory.json').read_text()):
 if x['label'] not in layout:continue
 root,strip=layout[x['label']]; dest=base/root;assert not dest.exists();dest.mkdir()
 files=0;size=0
 with tarfile.open(x['path']) as t:
  for m in t:
   if strip and not m.name.startswith(strip):continue
   rel=m.name[len(strip):]
   if not rel:continue
   # All scientific roots are complete; unrelated old launch/audit outputs are omitted.
   keep=rel.startswith('runs/') or rel=='saved-episode-authority.json' or rel=='capture-supervise.py'
   if x['label']=='baseline':keep=keep or rel.startswith('runtime/')
   if not keep:continue
   q=PurePosixPath(rel);assert not q.is_absolute() and '..' not in q.parts
   if m.isdir():continue
   assert m.isfile(),m.name
   p=dest/rel;p.parent.mkdir(parents=True,exist_ok=True)
   with p.open('xb') as out:shutil.copyfileobj(t.extractfile(m),out)
   p.chmod(m.mode&0o777);files+=1;size+=m.size
 receipt.append({'package':x,'destination':str(dest),'files':files,'bytes':size})
 print(root,files,size,flush=True)
parent=base/'sol-prototype/restore/parent-unpacked'
for rel in ['native-prediction-control/recomposition-503d64b39-attempt1','native-geometric-generate/prose-panel-dev-2']:
 shutil.copytree(base/rel,parent/rel)
(R/'restore-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
