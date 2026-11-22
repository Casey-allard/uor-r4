"""Project the literal browser-mode catalog; this is not a LexLean compiler."""
import argparse
import json
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[2]
MODEL = ROOT / 'app/src/FoundrySpecification.lex.tex'
TARGET = ROOT / 'studio/shell/chat-mode-catalog.mjs'

def project(source):
    matches = re.findall(r'name: browserChatModeCatalog,\s*parameters: \[\],\s*result: String,\s*body: ("(?:[^"\\]|\\.)*")\s*}', source)
    if len(matches) != 1:
        raise ValueError('exactly one canonical mode catalog is required')
    payload = json.loads(json.loads(matches[0]))
    return '// Generated from FoundrySpecification.browserChatModeCatalog. Do not edit.\nexport const chatModeCatalog = ' + json.dumps(payload, ensure_ascii=False, indent=2) + ';\n'

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    expected = project(MODEL.read_text())
    if args.check:
        if not TARGET.exists() or TARGET.read_text() != expected:
            raise SystemExit('browser mode catalog differs from canonical source')
        print('canonical browser mode catalog matches')
    else:
        TARGET.write_text(expected)
