"""Compile pinned Rust CUDA learner and focused checks; no model execution."""
from pathlib import Path
import os,subprocess,time,resource,json,hashlib,shutil
R=Path('/workspace/uor-r4/codex/sol-protected-joint-20261009')
SOURCE=json.loads((R/'source.json').read_text())['commit']
S=Path(json.loads((R/'source.json').read_text())['build_path'])
T=Path('/root/build/target-sm120')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
identity=json.loads((R/'source.json').read_text())
for rel,digest in identity['rust_files'].items():
 assert sha(S/rel)==digest,rel
e=dict(os.environ);e.update(CARGO_TARGET_DIR=str(T),UOR_BUILD_SOURCE_COMMIT=SOURCE,CARGO_INCREMENTAL='0',CUDA_COMPUTE_CAP='120',CARGO_BUILD_JOBS='2',RAYON_NUM_THREADS='2',OMP_NUM_THREADS='2',MKL_NUM_THREADS='2',OPENBLAS_NUM_THREADS='2')
e['PATH']='/root/.cargo/bin:/usr/local/cuda-12.8/bin:'+e['PATH'];e['LD_LIBRARY_PATH']='/usr/local/cuda-12.8/lib64:'+e.get('LD_LIBRARY_PATH','')
E=R/'evidence/source-bound-build';E.mkdir()
checks=[]
for name,args in [('build',['build','--release','--locked','-p','uor-r4-training','--features','cuda','--example','geometric-frozen-map-fit']),('coupled-tests',['test','--release','--locked','-p','uor-r4-training','--features','cuda','--example','geometric-frozen-map-fit','coupled_episode_learning','--','--test-threads=2'])]:
 start=time.monotonic()
 with (E/(name+'.log')).open('x') as f:r=subprocess.run(['cargo',*args],cwd=S,env=e,stdout=f,stderr=subprocess.STDOUT)
 receipt={'stage':name,'argv':['cargo',*args],'exit_code':r.returncode,'seconds':time.monotonic()-start,'child_max_rss_bytes':resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss*1024}
 checks.append(receipt);(E/'checks.json').write_text(json.dumps(checks,indent=2)+'\n');print(json.dumps(receipt),flush=True)
 if r.returncode:raise SystemExit(r.returncode)
assert 'test result: ok.' in (E/'coupled-tests.log').read_text() and '0 failed;' in (E/'coupled-tests.log').read_text()
shutil.copy2(T/'release/examples/geometric-frozen-map-fit',R/'runtime/geometric-frozen-map-fit')
(E/'binary.sha256').write_text(sha(R/'runtime/geometric-frozen-map-fit')+'\n');(E/'source.commit').write_text(SOURCE+'\n')
print('BUILD_AND_FOCUSED_TESTS_PASS',flush=True)
