"""Project only the project's literal discovery catalog payload; never a LexLean compiler."""
import argparse
import json
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[2]
MODEL = ROOT / 'app/src/FoundrySpecification.lex.tex'
TARGET = ROOT / 'studio/shell/discovery-catalog.mjs'

def project(source):
    matches = re.findall(r'name: discoveryCatalogJson,\s*parameters: \[\],\s*result: String,\s*body: ("(?:[^"\\]|\\.)*")\s*}', source)
    if len(matches) != 1:
        raise ValueError('discovery catalog must have exactly one literal payload')
    payload = json.loads(json.loads(matches[0]))
    return '// Generated from FoundrySpecification.discoveryCatalogJson. Do not edit.\nexport const discoveryCatalog = '+json.dumps(payload,ensure_ascii=False,indent=2)+';\n'

if __name__ == '__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');args=parser.parse_args()
    expected=project(MODEL.read_text())
    if args.check:
        if not TARGET.exists() or TARGET.read_text()!=expected:
            raise SystemExit('command discovery catalog projection differs from canonical source')
        print('canonical command discovery catalog projection matches')
    else:
        TARGET.write_text(expected)
