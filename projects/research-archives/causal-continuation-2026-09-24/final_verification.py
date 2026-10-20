from pathlib import Path
import json,hashlib,subprocess,re,datetime,os
r=Path('/Users/casey.allard/uor-r4-investigations/causal-20260924T000007');w=Path('/Users/casey.allard/uor-r4-worktrees/observer-transport-20260923');owner=Path('/Users/casey.allard/uor-r4')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
lib=Path(re.search(r'Running unittests src/lib.rs \(([^)]+)\)',(r/'core-final-tests.log').read_text()).group(1))
runner=Path(re.search(r'Running unittests src/bin/ordinary-lexical.rs \(([^)]+)\)',(r/'runner-final-tests.log').read_text()).group(1))
checks=[]
for executable,flt,expected in [(lib,'native_geometric::learner::transferable_lexical',20),(lib,'native_geometric::learner::relative_action_learning',2),(lib,'native_geometric::learner::hamilton_transport',6),(lib,'native_geometric::learner::lexical_residual',7),(runner,None,25),(r/'allocation-census',None,1)]:
 cmd=[str(executable)]+([flt]if flt else[]);run=subprocess.run(cmd,capture_output=True,text=True,timeout=60);assert run.returncode==0,run.stdout+run.stderr
 result=[x for x in run.stdout.splitlines()if x.startswith('test result:')];assert len(result)==1 and f'{expected} passed; 0 failed' in result[0],result
 checks.append({'binary':str(executable),'binary_sha256':sha(executable),'filter':flt,'result':result[0]});print(result[0],flush=True)
for cmd in [['git','diff','--check'],['/Users/casey.allard/.cargo/bin/cargo','fmt','--all','--check'],['python3','scripts/check_claim_wording.py']]:
 run=subprocess.run(cmd,cwd=w,capture_output=True,text=True,timeout=60);assert run.returncode==0,run.stdout+run.stderr
before=json.loads((r/'owner-extra-hashes.json').read_text());changed=[p for p,d in before.items()if not(owner/p).is_file()or sha(owner/p)!=d]
unexpected=[p for p in changed if not p.startswith('.omo/')];assert not unexpected,unexpected
owner_diff=subprocess.check_output(['git','diff','--binary'],cwd=owner);assert owner_diff==(r/'owner-diff-before.patch').read_bytes()
core='crates/uor-r4-core/src/native_geometric/learner/transferable_lexical.rs';harness='crates/uor-r4-core/src/bin/support/causal_continuation.rs'
for path in [core,harness]:
 base=subprocess.check_output(['git','show','4af04cf56f213df40b78c73a7c51ed5274517857:'+path],cwd=w);assert base==(w/path).read_bytes()
record={'recorded_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'focused_tests':checks,'total_passed':61,'owner_extra_files_hashed':len(before),'owner_changed_files':changed,'owner_tracked_diff_unchanged':True,'experiment_core_and_harness_identical_to_frozen_commit':True,'format_diff_claim_checks_passed':True,'physical_free_bytes':os.statvfs(owner).f_bavail*os.statvfs(owner).f_frsize,'scope':'Retained release test executables rerun, with pinned binary identities. No full-repository suite or whole-application qualification claim.'}
(r/'final-verification.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:v for k,v in record.items()if k!='focused_tests'},indent=2))
