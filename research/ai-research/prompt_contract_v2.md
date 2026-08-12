SESSION MODE: FRESH SONNET
Assume no prior conversation context.
Learn only from this prompt and the repo/files.  PROJECT / REPO CONTEXT

You are working in a local research repo for a prime-transport / geometric routing project.

Project purpose:
- study whether routing / inference can collapse from expensive learned multi-candidate evaluation into cheaper geometric execution laws
- study angular + radial state representations, harmonic structure, transport dynamics, delayed injection, and signal preservation
- use bounded experiments with explicit artifacts and honest reporting
- prioritize raw measured results over interpretation

Repo conventions:
- experiment scripts live under: tools/prime_transport/
- research writeups live under: docs/research/
- experiment CSV/JSON outputs live under: results/prime_transport_recursive_system/

Working style:
- each task must create the exact requested deliverables
- prefer bounded probes over broad redesigns
- do not silently widen scope
- do not rewrite conclusions unless explicitly asked
- raw numeric results are more important than prose summaries
- if a conclusion is confounded or inconclusive, say so plainly

Trust rules:
- trust raw CSV/JSON numbers more than markdown summaries
- do not claim a hypothesis is falsified unless the test was fair and uncontaminated
- do not optimize or redesign when the task is only measurement/extraction
- do not continue autonomously beyond the requested task

EXECUTION RULES

- Do not continue autonomously after the requested task is complete.
- Do not rewrite markdown to “improve” conclusions unless explicitly asked.
- Do not widen the scope beyond the stated objective.
- If the result is inconclusive, say “INCONCLUSIVE” plainly.
- Prefer direct extraction from code/CSV over speculative interpretation.  CANONICAL SOURCE CONTROL (MANDATORY)

Every task must explicitly define its canonical data sources.

Definitions:
- CANONICAL: files that define the current ground truth for reasoning
- CONTRAST: files used only for comparison or failure analysis

Rules:
- The model must ONLY treat CANONICAL files as the source of truth.
- CONTRAST files may not be used to define the system, only to explain deviations.

If multiple CSVs exist:
- Do NOT select a canonical source implicitly.
- Do NOT default to the most recent or most familiar file.
- Do NOT merge results across files unless explicitly instructed.

If canon is ambiguous:
- STOP and return: "CANON AMBIGUOUS"

If the model uses a non-canonical file as primary:
- The result is INVALID.

Required behavior:
- Explicitly list CANONICAL and CONTRAST sources before analysis.
- Ground all reasoning in CANONICAL sources first.
- Only then reference CONTRAST sources.

Failure to follow this invalidates the task.  GEOMETRY-NATIVE EXECUTION CONTRACT (MANDATORY)

- Do NOT define tasks in terms of mod, modulus, or k-harmonic labels.
- These may be used ONLY as post-hoc descriptors, never as primary task definitions.

- Tasks must be defined in geometry-native terms:
  - phase partitions
  - cyclic state structure
  - transition behavior
  - symmetry or invariance properties

- Harmonic structure must be treated as:
  - emergent from the representation
  - not pre-specified as "k=2", "k=3", etc.

- The system must not rely on modular arithmetic as a control primitive.

If mod/k appear in the implementation:
- they must be derived or incidental, not required for correctness.  
