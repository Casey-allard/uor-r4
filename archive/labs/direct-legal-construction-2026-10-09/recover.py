#!/usr/bin/env python3
"""Restore only verified retained archive members; no download, model or derivative work."""
from pathlib import Path
import ast, hashlib, json, shutil, tarfile
W=Path(__file__).resolve().parents[3]
retained=W/'local/restore/codex-protected-discrete-feedback-20261009/recovery'
root=W/'local/recovery';root.mkdir(exist_ok=False)
specs=next(ast.literal_eval(n.value) for n in ast.parse((retained/'fetch.py').read_text()).body if isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='specs' for t in n.targets))
specs.append(('parent','codex/native-prediction/503d64b39-source48-generate64/frozen-checkpoint-and-panel.tar.gz','b1192489b94d06811948fe7e8ac13e3482050cf4',22149739,'767dd285447118dc2c5ce7d0c68554ccaf16dcdec0d894e2f21f5dbf27d278b4','sol-prototype/restore/parent-unpacked',''))
receipts=[]
for label,key,rev,size,digest,base,strip in specs:
 p=retained/'downloads'/label/key
 assert p.stat().st_size==size and hashlib.sha256(p.read_bytes()).hexdigest()==digest
 dest=root/'inputs'/base;dest.mkdir(parents=True,exist_ok=False);files=0;expanded=0
 with tarfile.open(p) as t:
  for m in t:
   if strip and not m.name.startswith(strip):continue
   rel=m.name[len(strip):]
   if label!='parent' and not(rel.startswith('runs/') or rel in ['saved-episode-authority.json','capture-supervise.py']):continue
   q=Path(rel);assert not q.is_absolute() and '..' not in q.parts
   if m.isdir():continue
   assert m.isfile()
   assert shutil.disk_usage(W).free>(30<<30)+(128<<20)+m.size
   target=dest/q;target.parent.mkdir(parents=True,exist_ok=True)
   with target.open('xb') as out:shutil.copyfileobj(t.extractfile(m),out)
   files+=1;expanded+=m.size
 receipts.append(dict(label=label,sha256=digest,bytes=size,revision=rev,files=files,expanded_bytes=expanded))
 print(json.dumps(receipts[-1]),flush=True)
(root/'receipts.json').write_text(json.dumps(receipts,indent=2)+'\n')
