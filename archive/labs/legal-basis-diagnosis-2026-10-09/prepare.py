#!/usr/bin/env python3
"""Authenticate retained bytes and package f32 bit patterns; no model arithmetic."""
from pathlib import Path
import hashlib, json, struct, sys
import blake3
w=Path(__file__).resolve().parents[3]
root=w/'local/restore/codex-protected-joint-20261009/sol-protected-joint-20261009/runs/protected-joint-0001-attempt3'
out=w/'local/input';out.mkdir(exist_ok=False)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text())
pins=read(w/'docs/labs/protected-joint-construction-2026-10-09/result.json')
for name in ['report','manifest']:
 assert sha(root/f'{name}.json')==pins[f'{name}_sha256']
manifest=read(root/'manifest.json');paths=set()
for f in manifest['files']:
 rel=Path(f['path']);assert not rel.is_absolute() and '..' not in rel.parts and str(rel) not in paths
 paths.add(str(rel));p=root/rel
 assert not p.is_symlink() and p.stat().st_size==f['bytes']
 assert blake3.blake3(p.read_bytes()).hexdigest()==f['blake3']
assert {str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()}==paths|{'manifest.json'}
grad=read(root/'coupled-gradient-receipt.json');margin=read(root/'protected-margin-receipt.json')
assert sha(root/'protected-margin-receipt.json')==grad['protected_margin_receipt_sha256']
assert len(margin['terms'])==380
checks=[]
def bits(f,n):
 p=root/f['file'];assert sha(p)==f['sha256'];b=p.read_bytes();assert len(b)==4*n
 checks.append({'file':f['file'],'sha256':f['sha256'],'bytes':len(b)})
 return list(struct.unpack('<'+'I'*n,b))
lookup={(x['family'],x['kind']):x for x in grad['files']}
master=[];gradient=[]
for family in ['prefix.coefficients','generate.unary']:
 master+=bits(lookup[family,'initial-master'],960)
 gradient+=bits(lookup[family,'gradient'],960)
j=[]
for i,t in enumerate(margin['terms']):
 assert t['guard_index']==i and t['shape']==[1920] and t['status']=='PRESENT'
 j.append(bits(t,1920))
provenance={'archive':'icloud:UOR-R4/results/codex/codex-protected-joint-20261009.tar','archive_md5':'467c102adc510a6352951596b969f1be','root':str(root.relative_to(w/'local/restore')),'report_sha256':pins['report_sha256'],'manifest_sha256':pins['manifest_sha256'],'manifest_files_verified':len(paths),'raw_files':checks,'scope':'Original development input245 objective and380 guards from #2101, no new derivatives','normalization':'Rust harness uses current protected_joint_vector::unit_rows arithmetic'}
v={'schema':'uor-r4.legal-basis-input/1','master_bits':master,'gradient_bits':gradient,'jacobian_bits':j,'provenance':provenance}
p=out/'input.json';p.write_text(json.dumps(v,separators=(',',':'))+'\n')
receipt={'status':'PASS','input_sha256':sha(p),'input_bytes':p.stat().st_size,'manifest_files_verified':len(paths),'raw_files_verified':len(checks),'provenance':provenance}
(out/'verification.json').write_text(json.dumps(receipt,indent=2)+'\n')
(out/'guard-map.json').write_text(json.dumps(margin['terms'],indent=2)+'\n')
print(json.dumps({k:v for k,v in receipt.items() if k!='provenance'}))
