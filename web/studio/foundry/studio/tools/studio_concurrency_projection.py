"""Project a literal workflow boundary; this is not a Prism compiler or scheduler."""
import argparse
import json
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[2]
START = '# BEGIN FoundrySpecification.studioConcurrencyYaml\n'
END = '# END FoundrySpecification.studioConcurrencyYaml\n'

def project(model, workflow):
    values = re.findall(r'name: studioConcurrencyYaml,\s*parameters: \[\],\s*result: String,\s*body: ("(?:[^"\\]|\\.)*")\s*}', model)
    if len(values) != 1:
        raise ValueError('expected exactly one canonical Studio concurrency literal')
    block = json.loads(values[0])
    if not isinstance(block, str) or not block.startswith('concurrency:\n') or not block.endswith('\n'):
        raise ValueError('invalid canonical concurrency block')
    if workflow.count(START) != 1 or workflow.count(END) != 1:
        raise ValueError('expected exactly one Studio projection boundary')
    before, after = workflow.split(START)
    _, tail = after.split(END)
    if re.search(r"^concurrency:", before + tail, re.MULTILINE):
        raise ValueError("concurrency exists outside its canonical boundary")
    return before + START + block + END + tail

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    target = ROOT / '.github/workflows/studio.yml'
    actual = target.read_text()
    expected = project((ROOT / 'app/src/FoundrySpecification.lex.tex').read_text(), actual)
    if args.check:
        if actual != expected:
            raise SystemExit('Studio concurrency differs from canonical source')
        print('Studio concurrency matches canonical source')
    else:
        target.write_text(expected)
