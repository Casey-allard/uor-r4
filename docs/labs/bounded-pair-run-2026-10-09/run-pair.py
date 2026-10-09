#!/usr/bin/env python3
"""Run the admitted experiment and its immediate positive-only qualification."""
from pathlib import Path
import hashlib,json,subprocess,time,datetime,shutil
R=Path('/workspace/uor-r4/codex/sol-bounded-pair-20261009')
BINARY=R/'runtime/geometric-frozen-map-fit'
BINARY_SHA='f37b39134e865f6892c8cf0e5229376d1bcd9440ff3b6b89405b2ba73f702d63'
CONFIG=R/'configs/pair-0001-attempt1.json'
CONFIG_SHA='7c15dd4956770c0f008644f8ac4b99d60b6b2bf085847377d0fadbfa41c45740'
SUPERVISOR=Path('/workspace/uor-r4/codex/sol-sequence-progress-causal/capture-supervise.py')
FACTORY=R/'prepare-pair-actual9.py'
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_bytes())
def need(v,m):
 if not v:raise ValueError(m)
need(h(BINARY)==BINARY_SHA and h(CONFIG)==CONFIG_SHA,'runtime/config differs')
need(h(SUPERVISOR)=='393532acefc51e80d52952f3a1a367785523fade6a755b16439ffc902d836115','immutable supervisor differs')
need(h(FACTORY)=='27078b90849e9af53a27d40fd4173c0a89b09e130b8445616ee41793a9a18fda','reviewed qualification factory differs')
up=read(R/'runtime-upload.json');need(up['status']=='PASS' and up['all_fresh_download_verified'] and up['binary_sha256']==BINARY_SHA and up['config_sha256']==CONFIG_SHA,'durable runtime preservation incomplete')
need(read(R/'evidence/input-verification.json')['status']=='PASS','input verification missing')
need(shutil.disk_usage(R).free>(16<<30)+(128<<20),'storage admission')
need(not (R/'runs/pair-0001-attempt1').exists() and not (R/'observations/pair-0001-attempt1').exists(),'fresh run/observation root')
started=datetime.datetime.now(datetime.timezone.utc).isoformat();start=time.monotonic()
with (R/'execution-admission.json').open('x') as f:json.dump({'source_commit':'a66ecef50848cc11160fc0c5a1f188355731ff98','binary_sha256':BINARY_SHA,'config_sha256':CONFIG_SHA,'runtime_revision':up['revision'],'factory_sha256':h(FACTORY),'supervisor_sha256':h(SUPERVISOR),'started_utc':started,'family':'pair','maximum_coordinates':960,'physical_backward_calls':31,'cpu_threads':2,'model_process_ram_projection_bytes':4<<30,'whole_task_storage_cap_bytes':16<<30,'model_report_cap_bytes':512<<20,'actual_model_fit':'ADMITTED_NOT_YET_EXECUTED'},f,indent=2)
argv=['python3',str(SUPERVISOR),str(BINARY),str(CONFIG),BINARY_SHA,CONFIG_SHA,'cuda-learning',str(R/'observations/pair-0001-attempt1')]
subprocess.run(argv,check=True)
subprocess.run(['python3',str(FACTORY)],check=True)
decision=read(R/'evidence/actual9-config-decision.json')
if decision['status']=='CONFIG_PREPARED_NOT_EVALUATED':
 cfg=Path(decision['config']);need(h(cfg)==decision['config_sha256'],'qualification config changed')
 subprocess.run(['python3',str(SUPERVISOR),str(BINARY),str(cfg),BINARY_SHA,decision['config_sha256'],'cpu-artifact',str(R/'observations/actual9-0001-attempt1')],check=True)
else:need(decision['status']=='NOT_RUN_CONSTRUCTION_NEGATIVE','unknown construction decision')
result={'status':'MODEL_AND_CONDITIONAL_QUALIFICATION_FINISHED','started_utc':started,'seconds':time.monotonic()-start,'construction_report_sha256':h(R/'runs/pair-0001-attempt1/report.json'),'construction_manifest_sha256':h(R/'runs/pair-0001-attempt1/manifest.json'),'actual9':decision['status'],'independent_saved_audit':'NOT_RUN'}
with (R/'execution-result.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
print(json.dumps(result),flush=True)
