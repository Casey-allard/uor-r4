from pathlib import Path
import subprocess,hashlib,json
r=Path('/Users/casey.allard/uor-r4-investigations/causal-20260924T000007');deps=Path('/Users/casey.allard/uor-r4-kimi/target/release/deps')
command=['/Users/casey.allard/.cargo/bin/rustc','--edition=2021','-C','opt-level=2',str(r/'verify_retain.rs'),'-L','dependency='+str(deps),'-o',str(r/'verify-retain')]
libs={}
for crate in ['uor_r4_core','serde_json','sha2']:
 p=max(deps.glob('lib'+crate+'-*.rlib'),key=lambda p:p.stat().st_mtime);command+=['--extern',crate+'='+str(p)];libs[crate]={'path':str(p),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
with (r/'verifier-build.log').open('w') as f:subprocess.run(command,stdout=f,stderr=subprocess.STDOUT,check=True,timeout=90)
with (r/'verifier-run.log').open('w') as f:subprocess.run([str(r/'verify-retain'),str(r)],stdout=f,stderr=subprocess.STDOUT,check=True,timeout=60)
(r/'verifier-build-identity.json').write_text(json.dumps({'command':command,'libraries':libs,'source_sha256':hashlib.sha256((r/'verify_retain.rs').read_bytes()).hexdigest()},indent=2));print((r/'verifier-run.log').read_text())
