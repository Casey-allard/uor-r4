# Static runtime source audit — #1096

**2026-09-03 — `SOURCE_INSPECTED`; correction candidate pending the sole readiness attempt.**
Inspection began at `origin/main` `df2c4cb8ef47e35b6d4083d4b8da135c7676fc19`
in the isolated `r4-runtime-readiness` worktree. This record separates a verified
allowlist omission from the unverified cause of the original launch denial.
The auditor inspected source, retained receipts, symlink/configuration metadata
and Mach-O dependency metadata. No Python runtime, sandbox, model, corpus,
comparison, proof tool or build was executed by this audit.

## Preserved stop and launcher

The original [#1094 stop](r4_text_clause_adapter_1094_evidence/prepare-stopped.json)
records `UNAVAILABLE_REFERENCE_REPLAY`, an `execvp` failure with `Operation not
permitted`, and no Python readiness event. Its SHA256 remains
`87bd3082ce9b4da5e5227a3b82f6515773cf5f113de689ff6251a2a45340fad5`.
The [original profile](r4_text_clause_adapter_1094_evidence/worker.sb) has SHA256
`914b72856e0822c981b1295b8e96048fa43f35b8c1dedeea58f51321a781a72d`.
That single error does not identify the denied path or operation.

The original `campaign.arm_process` starts `/usr/bin/sandbox-exec -f worker.sb`,
then the venv interpreter with `-m r4_softmax_trainer.text_clause_adapter.worker`,
the binding path, the selected arm and `--readiness-only`. It sets the candidate
source `PYTHONPATH`, disables bytecode writes, sets the declared CPU thread
environment, uses working directory `/`, and supplies a denied sentinel path.
The actual interpreter chain, inspected using `ls` and `readlink`, is:

```text
/Users/casey.allard/.codex/worktrees/r4-language-r4/uor-r4/tools/r4-softmax-trainer/.venv/bin/python
  -> /Users/casey.allard/.local/share/uv/python/cpython-3.12-macos-aarch64-none/bin/python3.12

/Users/casey.allard/.local/share/uv/python/cpython-3.12-macos-aarch64-none
  -> /Users/casey.allard/.local/share/uv/python/cpython-3.12.14-macos-aarch64-none
```

The final executable is therefore
`/Users/casey.allard/.local/share/uv/python/cpython-3.12.14-macos-aarch64-none/bin/python3.12`.
The venv's `pyvenv.cfg` names the intermediate `cpython-3.12-macos-aarch64-none/bin`
as `home`, and sets `include-system-site-packages = false`.

At the inspected baseline, `contract.sandbox_profile` permits the venv tree and
`python.resolve().parent.parent`, the final runtime tree. It omits the intermediate
uv alias. This is a source/metadata finding; attributing the original `execvp`
failure to that omission is a hypothesis.

## Operations supported by the inspected sources

The Python 3.12 [path-initialization documentation](https://docs.python.org/3.12/library/sys_path_init.html)
describes executable-based standard-library discovery, symbolic-link resolution,
`PYTHONPATH`, and subsequent `site` processing. [PEP 405](https://peps.python.org/pep-0405/)
describes detecting `pyvenv.cfg` near the venv executable, using its `home` for the
base installation, and retaining venv-specific site packages. These references
support startup semantics only; neither diagnoses this machine's sandbox denial.

The worker's binding verification calls `Path.resolve()` on every bound source
path and its executing package. The installed CPython `pathlib.py` delegates to
`posixpath.realpath`; `_joinrealpath` calls `os.lstat` on each path component
(line 468) and `os.readlink` on symbolic links (line 491). With non-strict
resolution, an `OSError` from `lstat` can treat a component as non-symbolic.
Literal metadata access to ancestors of already allowed paths is therefore
supported by actual worker operations. This does not establish which ancestor
operation, if any, caused the pre-Python failure.

Apple's installed `/usr/share/sandbox/kcm.sb` separates metadata access for code
signature checks from data access; `/usr/share/sandbox/webdav_agent.sb` also uses
a literal metadata allowance. The installed `sandbox-exec(1)` manual describes
applying a profile before executing the command. These local primary sources
support the distinction between metadata and content access, without providing
a trace of this failure or a general sandbox-security guarantee.

`otool -L` inspected the final Python executable and these files under the venv's
`lib/python3.12/site-packages/torch/`:

- `_C.cpython-312-darwin.so`
- `lib/libtorch_python.dylib`
- `lib/libtorch_cpu.dylib`

Python's listed dependencies are under `/System` and `/usr/lib`. The inspected
Torch objects list system dependencies and `@rpath` libraries present in the
already allowed venv, including `libc10`, `libomp`, `libtorch`, and `libshm`.
These load-command observations support no additional home-directory runtime
tree. They are not an executed dynamic-loader trace or a complete audit of every
possible optional import.

The installed `_editable_impl_uor_r4_softmax_trainer.pth` adds the old
`/Users/casey.allard/.codex/worktrees/r4-language-r4/uor-r4/tools/r4-softmax-trainer/src`
path. Preserve its denial: the explicit candidate `PYTHONPATH` precedes ordinary
site path additions, and the worker verifies that its executing source is in the
bound closure. No permission for that old editable source follows from this audit.

## Small primary-file identities

`BASE` below means
`/Users/casey.allard/.local/share/uv/python/cpython-3.12.14-macos-aarch64-none`;
`VENV` means
`/Users/casey.allard/.codex/worktrees/r4-language-r4/uor-r4/tools/r4-softmax-trainer/.venv`.
These are local originals inspected without executing them.

| Original file | SHA256 |
|---|---|
| `BASE/lib/python3.12/posixpath.py` | `03825681086638649a43480954f7f6a16b4da3bd41ece956864ae96e4f795cd9` |
| `BASE/lib/python3.12/pathlib.py` | `10ba48ae8063cfe7589436041d0d4628c05f427c7709cf8e75e486f8930cc32c` |
| `VENV/pyvenv.cfg` | `00191a444d479d3efa08bd18d0519f75ff38b76b9d7cd59fc370599cee6b9fa9` |
| `VENV/lib/python3.12/site-packages/_editable_impl_uor_r4_softmax_trainer.pth` | `53a9e8fd048f31caf3d6dcc02194bbed6e4240e0aa3a5f3a1361d50ce39030df` |
| `VENV/lib/python3.12/site-packages/_virtualenv.pth` | `69ac3d8f27e679c81b94ab30b3b56e9cd138219b1ba94a1fa3606d5a76a1433d` |
| `/usr/share/sandbox/kcm.sb` | `4b0db524ca9e7cb731c7a164ba5cf42a8c7997eec984723033004d1a495b3aaa` |
| `/usr/share/sandbox/webdav_agent.sb` | `6fcf379ccc5a1d1962abb12a596eb795e857229cbca5860450967fab1af24c47` |

## Candidate and evidence boundary

The proposed correction retains the existing venv/final-runtime/source trees
and exact bound files. It adds literal reads of intermediate runtime symlinks
and metadata-only exceptions for exact ancestors of permitted paths. Directory
enumeration/content access through those ancestors, broad home/uv allowlists,
old editable source, corpus/reference data and historical reports remain outside
the candidate. Its actual effect is pending independent review and the sole
bounded readiness attempt.

The existing worker's `--readiness-only` branch verifies source/asset bytes,
tests isolation and configures Torch, then skips model loading and the stdin
batch loop. A separate parent deadline is needed for #1096's 60-second cap;
the original worker timer is 120 seconds. Calling #1094 `prepare()` would also
read authoring corpus and create preparation receipts, so it is outside this
isolated readiness decision. Success requires the actual ready/done receipts,
fixed runtime and binding identities, denied harmless probes, null model states,
zero model loads/forwards/updates, resource compliance and exit zero.

The accepted [#1085 NEMESIS/W33/UOR source audit](integration/clause-segmentation-1085-sources.md)
is reused as background guidance for typed admission, raw-byte identity, ordered
provenance and distinct artifact/schema identities. Its pinned external originals
were not downloaded or reread for this runtime audit. Those sources establish no
Python startup, sandbox, parser or model result. Mathematical proof, measured
readiness and the present cause hypothesis remain distinct. The original #1094
stop, #1079 weak-control finding and #1082 descriptive evidence are preserved.
Readiness success alone cannot qualify raw-text behavior or authorize withheld
comparison.
