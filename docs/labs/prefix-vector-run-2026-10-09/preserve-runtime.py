"""Preserve checked native runtime, config and launch helpers before model execution."""
from pathlib import Path
from huggingface_hub import HfApi,hf_hub_download
import json,hashlib,shutil,subprocess,time
R=Path('/workspace/uor-r4/codex/sol-prefix-vector-20261009');SOURCE='cc079744f4824e91a6ad739e2e7cf45c8d374527';S=Path('/root/build/src-'+SOURCE)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text())
start=time.monotonic();checks=R/'evidence/source-bound-build';runtime=R/'runtime'
assert not (R/'runtime-upload.json').exists()
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=S,text=True).strip()==SOURCE
assert not subprocess.check_output(['git','status','--porcelain'],cwd=S,text=True).strip()
assert all(c['exit_code']==0 for c in read(checks/'checks.json'))
assert 'test result: ok. 25 passed; 0 failed;' in (checks/'coupled-tests.log').read_text()
config=R/'configs/prefix-vector-0001-attempt1.json';config_sha=sha(config)
assert read(R/'evidence/input-verification.json')['config_sha256']==config_sha
assert sha(runtime/'geometric-frozen-map-fit')==(checks/'binary.sha256').read_text().strip()
old=read(Path('/workspace/uor-r4/codex/sol-full-donor-20261009/runtime/runtime-identity.json'))
assert sha(runtime/'native_historical_version')==old['files']['native_historical_version']['sha256']
for p in checks.iterdir():shutil.copy2(p,runtime/p.name)
for name in ['build-runtime.py','recover-inputs.py','restore-layout.py','verify-inputs.py','prepare-prefix-vector-actual9.py','run-prefix-vector.py','preserve-runtime.py']:
 shutil.copy2(R/name,runtime/name)
shutil.copy2(config,runtime/'prefix-vector-config.json');shutil.copy2(R/'evidence/input-verification.json',runtime/'input-verification.json')
source=runtime/'owned-source';source.mkdir()
for name in ['coupled_episode_learning.rs','gradient_vector_prefix.rs','context_cue_coadapt.rs','generate_episode_learning.rs']:
 shutil.copy2(S/'crates/uor-r4-training/examples/geometric_frozen_map_fit'/name,source/name)
for path in ['/workspace/uor-r4/codex/sol-sequence-progress-causal/capture-supervise.py','/workspace/uor-r4/codex/sol-prefix-margin-boundary/claim-report','/workspace/uor-r4/codex/sol-intermediate-word-boundary/publications/runtime-76e10b53f7-attempt2/native-reached-prefix-attribution']:
 shutil.copy2(path,runtime/Path(path).name)
files={str(p.relative_to(runtime)):{'sha256':sha(p),'bytes':p.stat().st_size} for p in runtime.rglob('*') if p.is_file()}
identity={'source_commit':SOURCE,'source_tree':subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=S,text=True).strip(),'clean_build_checkout':True,'binary_sha256':sha(runtime/'geometric-frozen-map-fit'),'config_sha256':config_sha,'donor_credit':'full_pool_utility','prefix_transaction':'gradient_vector_prefix','active_families':['prefix.coefficients','generate.unary'],'coordinates':1920,'constructor_coordinates':960,'vector_radii':[1,2,4,7],'cuda_compute_cap':120,'focused_tests':25,'input_roots_verified':15,'actual_model_fit':'NOT_RUN','sealer_origin':'restored verified #2069 runtime','files':files}
(runtime/'runtime-identity.json').write_text(json.dumps(identity,indent=2)+'\n')
repo='caseyallard/uor-r4-store';prefix='codex/sol-prefix-vector-20261009/runtime-cc079744-attempt1'
commit=HfApi().upload_folder(repo_id=repo,repo_type='dataset',folder_path=str(runtime),path_in_repo=prefix,commit_message='Preserve verified coordinated Prefix CUDA runtime and original-parent config before execution')
fresh=R/'runtime-redownload'
for p in runtime.rglob('*'):
 if not p.is_file():continue
 q=Path(hf_hub_download(repo,prefix+'/'+str(p.relative_to(runtime)),repo_type='dataset',revision=commit.oid,local_dir=fresh,force_download=True))
 assert sha(p)==sha(q) and p.stat().st_size==q.stat().st_size
receipt={'status':'PASS','revision':commit.oid,'prefix':prefix,'all_fresh_download_verified':True,'source_commit':SOURCE,'binary_sha256':identity['binary_sha256'],'config_sha256':config_sha,'payload_bytes':sum(p.stat().st_size for p in runtime.rglob('*') if p.is_file()),'seconds':time.monotonic()-start}
(R/'runtime-upload.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt),flush=True)
