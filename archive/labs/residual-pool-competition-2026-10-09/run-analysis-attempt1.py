#!/usr/bin/env python3
"""Run one saved-journal reader inside native claimed/sealed report boundaries."""
from pathlib import Path
import argparse,datetime,hashlib,json,os,resource,subprocess,time

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def need(v,m):
 if not v:raise ValueError(m)
def write(p,v):
 with p.open('x') as f:json.dump(v,f,indent=2,allow_nan=False);f.write('\n')
def main():
 a=argparse.ArgumentParser();a.add_argument('--reader-sha256',required=True);args=a.parse_args()
 repo=Path(__file__).resolve().parents[3];local=repo/'local';reader=Path(__file__).with_name('analyze.py')
 tool=local/'bin/report-tool'
 need(sha(tool)=='79e8370646462717a5fd94f6e868383baca90ed491fc583646bb93d187cea0de','normative tool identity')
 need(sha(repo/'crates/uor-r4-integer/src/report_output.rs')=='cda30f9ba4ef8ff3e245ae352cd28932f9a24ab78598f683607e6996598fa810','normative source identity')
 need(sha(reader)==args.reader_sha256,'reviewed reader bytes')
 output=local/'competition-attempt1';obs=local/'observation-attempt1'
 need(not output.exists() and not obs.exists(),'exclusive fresh roots')
 for p in [output,obs]:subprocess.run([str(tool),'claim',str(p)],check=True)
 start=time.monotonic();started=datetime.datetime.now(datetime.timezone.utc).isoformat();rc=None;error=None
 try:
  evidence=local/'recovered/evidence';identity=local/'recovered/codex-full-donor-outcome-20261009/publication-full-donor-0001-attempt1/identity.json'
  checked=[]
  for rel in ['runs/full-donor-0001-attempt1','observations/full-donor-0001-attempt1','audits/full-donor-0001-attempt1']:
   p=evidence/rel;r=subprocess.run([str(tool),'verify',str(p)],capture_output=True,text=True,check=True);checked.append({'root':rel,'manifest_sha256':sha(p/'manifest.json'),'exit_code':r.returncode})
  write(obs/'input-verification.json',checked)
  argv=['python3',str(reader),'--run',str(evidence/'runs/full-donor-0001-attempt1'),'--identity',str(identity),'--evidence',str(evidence),'--output',str(output)]
  write(obs/'launch.json',{'argv':argv,'reader_sha256':args.reader_sha256,'started_utc':started,'scope':'Saved pool competition and feature incidence only; no model forward, encoder, proposal or backward','cpu_workers':2})
  with (obs/'stdout.log').open('x') as stdout,(obs/'stderr.log').open('x') as stderr:
   r=subprocess.run(argv,stdout=stdout,stderr=stderr,env={**os.environ,'OMP_NUM_THREADS':'2','OPENBLAS_NUM_THREADS':'2'});rc=r.returncode
  need(rc==0,'saved reader failed; inspect preserved stderr')
  need(json.loads((output/'report.json').read_text())['status']=='PASS_SAVED_COMPETITION','reader result status')
 except Exception as exc:
  error=str(exc);raise
 finally:
  write(obs/'execution.json',{'status':'PASS' if rc==0 and error is None else 'FAILED','exit_code':rc,'error':error,'started_utc':started,'seconds':time.monotonic()-start,'children_max_rss_bytes_macos':resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss,'reader_sha256':args.reader_sha256})
  for p in [output,obs]:
   subprocess.run([str(tool),'seal',str(p)],check=True);subprocess.run([str(tool),'verify',str(p)],check=True)
 print(json.dumps({'status':'PASS','output':str(output),'observation':str(obs),'seconds':time.monotonic()-start}))
if __name__=='__main__':main()
