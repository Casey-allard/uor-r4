# `just vv` is the normative release gate (SPEC.md section 20.2).
default: vv

# The whole gate. Expected to FAIL at step 9 while WGG-GO-1 is outstanding.
vv: root-check firewall manifest-check releasepath build required claims axioms
    @python3 Tools/gate.py

bootstrap:
    @lean --version && lake --version

build:
    lake build

test:
    lake build

prove:
    lake build

claims:
    @python3 -c "import json;d=json.load(open('model/claims.json'));print(f'claims: {len(d[\"claims\"])}');[print(f'  {c[\"id\"]:<8} {c[\"level\"]:<13} {c[\"status\"]}') for c in d['claims']]"

axioms:
    @python3 Tools/axioms.py

artifact-check:
    @test -f artifacts/wasm-gemm-gnaf.wasm || (echo "artifact absent: gated on WS-001/LB-001" && exit 1)

emit:
    @echo "emit: gated on WS-001 (mechanized Core 3.0 semantics)" && exit 1

mutation:
    @echo "mutation: gated on the checkers it would falsify" && exit 1

reproduce:
    @echo "reproduce: gated on emit" && exit 1

docs:
    @python3 Tools/gen_conformance.py

# Regenerate the root import module from the layer tree.
root:
    @python3 Tools/root.py

# Fail if the root import is stale or any module belongs to no SPEC 5 layer.
root-check:
    @python3 Tools/root.py --check

# SPEC 10.1: the competitor universe must not import the artifact or a conclusion.
firewall:
    @python3 Tools/firewall.py

# SPEC 4/5: regenerate the ordered acyclic identity manifest.
manifest:
    @python3 Tools/manifest.py

manifest-check:
    @python3 Tools/manifest.py --check

# SPEC 19/6.3: no noncomputable definition on the release path.
releasepath:
    @python3 Tools/releasepath.py

# SPEC 15: required declarations, checked against the compiled environment.
required:
    @python3 Tools/required.py --list

# SPEC 1: the frozen WGG-GO-1 schema binding is definitional (Iff.rfl).
schema:
    @lake build WasmGemmGnaf.Conformance.Schema && echo 'schema binding holds'
