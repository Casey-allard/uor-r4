# ORDER: m4-lawful-likelihood-bpb, deepseek-8 (a lawful likelihood for BPB scoring: the design unit)

Lab: deepseek. Item: [m4-lawful-likelihood-bpb](../../../../roadmap/items/m4-lawful-likelihood-bpb.md) (#2861). Base: `a8ff5690e`.

## Goal

Record the design note the item requires before any code: a geometric construction from the model's Q24 integer potentials to an exact Q48 probability that contains none of the banned normalized-exponential forms and no dense matmul, with testable lawfulness properties, the finite host controls, and the open choices the PM and FIT must settle. **Design-only unit:** the item stays `ready`; no code, no model or GPU work, no BPB number.

## Cancel probe (run first)

Class **instrument**. At base `a8ff5690e`: no lawful likelihood exists (`docs/research/open-questions.md`: "A lawful likelihood is still unbuilt"); the historical integer read normalizes through a Q31 exp table (`crates/uor-r4-integer/src/geometric_read.rs`), a normalized-exponential form §3.2 bans; `sampling.rs` requires masses summing exactly to `1<<48` and nothing lawful produces them; CHARTER §4.8 says BPB returns once a lawful likelihood exists. The item is ready, scoped and unclaimed. The task stands.

## Files in scope

`docs/labs/m4-lawful-likelihood-bpb/**`, `crates/uor-r4-discrete/**`, `crates/uor-r4-integer/**`, `docs/mechanisms/lut-serving.md`, `docs/model/serving-and-export.md`, `docs/research/findings.md`, `docs/research/open-questions.md`, and this task folder. This unit writes only `docs/labs/m4-lawful-likelihood-bpb/README.md` and the task folder.

## Done when

- The design note is posted on #2861 and recorded in `docs/labs/m4-lawful-likelihood-bpb/README.md`.
- It states the geometric construction (the potential-ratio lawful likelihood), why no §3.2 banned form and no matmul appears, what "lawful" means as testable properties, how bits are computed from the masses, and the finite host controls before any GPU work.
- It names the open choices the PM/FIT settle: normalization family, calibration, the held-out stream's name and hash, division in the scoring path, and the crate boundary.
- It is honest about the unit: no code, no model or GPU run, no BPB number; the item remains ready for the implementation unit.
- `loop gates --full` is GREEN, and `loop report --check` and `loop pr-check --local` pass.

## Stop and ask if

- The design needs a bar, status, constraint or spend change (an owner card), or the held-out stream must be named now (that is the pre-registration/FIT step).
- GPU work becomes implied (it is not in this unit).
- A file outside the scope must change.