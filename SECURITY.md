# Security policy

UOR-R4 is a pre-alpha research project. There are no supported releases and no
security-maintained version; the only branch is `main`, and it is not a production service.

## Reporting a vulnerability

Report privately through GitHub Security Advisories:
<https://github.com/UOR-Foundation/uor-r4/security/advisories/new>

Do not open a public issue for a suspected vulnerability. The maintainer is the repository
owner, Casey Allard ([@Casey-allard](https://github.com/Casey-allard)). The
[organization-level policy](https://github.com/UOR-Foundation/.github/blob/main/SECURITY.md)
also applies.

We aim to acknowledge a report within 72 hours and to say within seven days whether it is in
scope and what the next step is. This is a volunteer research project: there is no bounty
programme and no paid response commitment.

## What is in scope

- The Rust crates in this repository and the scripts under `scripts/`.
- The WebAssembly serving path (`serving-wasm`) and its audit scripts.
- Committed data and configuration.

## What is not in scope

- Third-party providers, GPU pods or model hosts used by the labs.
- Historical, inactive evidence preserved under `docs/`: a defect there is a documentation
  issue, not a vulnerability.
- Anything that requires a credential the project does not issue.

## Secrets

Never commit credentials, API keys, tokens, passwords or absolute home paths. The project's
own rule is in [CONTRIBUTING.md](CONTRIBUTING.md) and
[docs/contributing/HARD-RULES.md](docs/contributing/HARD-RULES.md). If a secret does land in
a commit, report it privately as above rather than opening a PR that pastes it into the
conversation.
