"""Project only the project's literal registry payload; never a LexLean compiler."""
import argparse
import json
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[2]
MODEL = ROOT / 'app/src/CommandRegistry.lex.tex'
TARGET = ROOT / 'studio/shell/command-registry-data.mjs'

def project(source):
    matches = re.findall(r'name: commandRegistryJson,\s*parameters: \[\],\s*result: String,\s*body: ("(?:[^"\\]|\\.)*")\s*}', source)
    if len(matches) != 1:
        raise ValueError('registry must have exactly one literal payload')
    payload = json.loads(json.loads(matches[0]))
    return '// Generated from CommandRegistry.commandRegistryJson. Do not edit.\nexport const commandRegistryData = '+json.dumps(payload,ensure_ascii=False,indent=2)+';\n'

if __name__ == '__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');args=parser.parse_args()
    expected=project(MODEL.read_text())
    if args.check:
        if not TARGET.exists() or TARGET.read_text()!=expected:
            raise SystemExit('command registry projection differs from canonical source')
        print('canonical command registry projection matches')
    else:
        TARGET.write_text(expected)
