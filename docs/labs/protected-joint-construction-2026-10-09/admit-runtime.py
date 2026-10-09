"""Bind the built runtime, pinned config and helpers before any model invocation."""
from pathlib import Path
import hashlib,json,shutil
R=Path('/workspace/uor-r4/codex/sol-protected-joint-20261009')
def h(p):
 d=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1<<20),b''):d.update(b)
 return d.hexdigest()
def read(p):return json.loads(p.read_text())
def write(p,v):
 with p.open('x') as f:json.dump(v,f,indent=2);f.write('\n')
# The pinned historical supervisor seals via this legacy executable path.
# Resolve and verify it before launching a process, including failure paths.
SEALER_SHA='369374dfd08ba8f5b0c5a2a9ff80aab25977614d381fc81dc979991db38c481d'
sealer=Path('/workspace/uor-r4/codex/sol-prefix-vector-20261009/runtime/native_historical_version')
assert h(sealer)==SEALER_SHA
for dest in [Path('/root/codex/prototype-target/release/examples/native_historical_version'),R/'runtime/native_historical_version']:
 if not dest.exists():
  dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(sealer,dest)
 assert h(dest)==SEALER_SHA
s=read(R/'source.json');b=R/'runtime/geometric-frozen-map-fit';c=R/'configs/protected-joint-0001-attempt3.json'
checks=read(R/'evidence/source-bound-build/checks.json')
assert len(checks)==2 and all(x['exit_code']==0 for x in checks)
assert (R/'evidence/source-bound-build/source.commit').read_text().strip()==s['commit']
assert (R/'evidence/source-bound-build/binary.sha256').read_text().strip()==h(b)
assert read(R/'evidence/input-verification.json')['config_sha256']==h(c)
files={}
for name in ['prepare-actual9.py','run-protected-joint.py']:
 p=R/name;files[name]={'sha256':h(p),'bytes':p.stat().st_size};shutil.copy2(p,R/'runtime'/name)
identity={'source_commit':s['commit'],'source_snapshot_verified':True,'supervisor_sealer_sha256':SEALER_SHA,'binary_sha256':h(b),'config_sha256':h(c),'donor_credit':'full_pool_utility','coordinates':1920,'prefix_transaction':'protected_joint_vector','active_families':['prefix.coefficients','generate.unary'],'files':files}
write(R/'runtime/runtime-identity.json',identity)
write(R/'runtime-preservation.json',{'source_commit':s['commit'],'status':'PASS','canonical_network_volume':True,'binary_sha256':h(b),'config_sha256':h(c),'root':str(R),'scope':'source snapshot checked against source.json; built runtime/config/helpers preserved on canonical EUR-NO-1 network volume before execution; complete outcome requires iCloud preservation after use'})
print(json.dumps(identity))
