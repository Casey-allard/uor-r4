"""Recover immutable saved input packages; does not execute model code."""
from pathlib import Path
from huggingface_hub import HfApi,hf_hub_download
from concurrent.futures import ThreadPoolExecutor
import hashlib,json,tarfile
R=Path('/workspace/uor-r4/codex/sol-prefix-vector-20261009/recovery')
repo='caseyallard/uor-r4-store';api=HfApi();revision=api.repo_info(repo,repo_type='dataset').sha
specs=[('intermediate','codex/sol-readout-intermediate-candidate/intermediate-0001-4232ad3287','intermediate-0001-4232ad3287.tar.gz'),('prefix-credit','codex/sol-prefix-context-credit/prefix-context-credit-0001-92215de1ef','prefix-context-credit-0001-92215de1ef.tar.gz'),('episode','codex/sol-episode-progression-learning/complete-attempt1','complete-evidence.tar.gz'),('joint','codex/sol-joint-fragment-learning/complete-attempt1','evidence.tar.gz'),('supplement','codex/sol-prefix-trajectory-learning/supplement-results-attempt1','supplement-results.tar.gz'),('sequence','codex/sol-sequence-progress-causal/complete-attempt1','evidence.tar.gz'),('legacy','codex/sol-coupled-episode-learning/outcome-export-attempt3','outcome.tar.gz'),('baseline','codex/sol-full-donor-20261009/outcome-full-donor-0001-attempt1','outcome.tar.gz')]
def download(spec):
 name,prefix,file=spec
 p=Path(hf_hub_download(repo,prefix+'/'+file,repo_type='dataset',revision=revision,local_dir=R/name))
 with tarfile.open(p) as t:names=t.getnames()
 x={'label':name,'revision':revision,'prefix':prefix,'path':str(p),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'first_members':names[:15],'interesting_members':[n for n in names if n.endswith(('capture-supervise.py','saved-episode-authority.json')) or 'conditional-position-08-attempt1/manifest.json' in n]}
 print(json.dumps(x),flush=True);return x
receipts=list(ThreadPoolExecutor(4).map(download,specs))
(R/'download-inventory.json').write_text(json.dumps(receipts,indent=2)+'\n')
