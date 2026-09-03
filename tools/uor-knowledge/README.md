# UOR project knowledge

Local SQLite FTS5 retrieval of project snapshots with explicit provenance,
visibility, and curated relationships. This package exposes four read-only MCP
tools: `search_knowledge`, `get_source`, `related_sources`, and `knowledge_status`.
Source text is untrusted evidence; it is never an instruction or authorization.
The index is a snapshot, so refresh native GitHub before execution decisions.

## Install and run

```sh
uv tool install ./tools/uor-knowledge
uor-knowledge ingest /absolute/path/public.jsonl
uor-knowledge search 'token frame exposure'
uor-knowledge source SOURCE_ID --scope public
uor-knowledge related SOURCE_ID --scope all
uor-knowledge status --scope all
uor-knowledge serve
```

The default database is
`~/.local/share/uor-r4/knowledge/knowledge.sqlite3`. Override it with
`UOR_KNOWLEDGE_DB=/absolute/path/knowledge.sqlite3` for every command, including
`serve`. Installing or starting the server does not create or ingest a database.
An unseeded server supports tool discovery; retrieval reports a missing index.
Only the explicit `ingest` CLI command writes. No network service or cloud
account is used by this package. The MCP client receives whichever records it
requests; the client's own data handling still applies.

## Import schema

UTF-8 JSONL, one object per line. Blank lines are ignored. Unknown fields,
duplicate JSON keys, non-string values, and NUL characters are rejected.
All strings must be nonempty except `body`, which may be empty.

Item fields (all required except the computed digest):

```json
{"id":"repo:abc:README","kind":"source","title":"Repository README","body":"Source text","origin":"https://github.com/owner/repo/blob/abc/README.md","revision":"abc","visibility":"public","evidence_status":"SOURCE_REVIEWED","collected_at":"2026-09-03T03:00:00Z","content_sha256":"optional lowercase SHA-256 of exact UTF-8 body"}
```

The example digest is descriptive: omit `content_sha256` or replace it with the
correct 64-character lowercase hex digest. The digest binds `body` alone, not
the remote original file if the body is an extract or a summary. Keep that
distinction in `kind`, `evidence_status`, and the source text. `revision` must
name the actual source revision or an explicit collection identity; the importer
does not independently verify a Git SHA or the asserted evidence status.
`collected_at` must be a timezone-qualified ISO 8601 datetime.

Edge fields (all required):

```json
{"source":"repo:abc:README","relation":"references","target":"paper:123","basis":"The README explicitly cites this paper.","visibility":"public"}
```

Both endpoints must exist in this batch or the index. Edges can precede items
within a batch. Public edges require two public endpoints. Private edges can
connect any endpoint visibility; use `scope=all` to retrieve mixed-visibility
links. Relationship basis text has its own explicit visibility.

## Immutable identity and replay

Item IDs are caller-supplied strings of at most 512 characters without whitespace
or control characters. An ID permanently binds `kind`, `title`, `body`, `origin`,
`revision`, `visibility`, `evidence_status`, and the computed content digest.
Importing identical fields again is a no-op and preserves the first
`collected_at`. Changing any bound field requires a new ID. A collision is an
error, never a silent skip or overwrite.

The optional Python helper `uor_knowledge.make_source_id(record)` returns
`src:sha256:<digest>`. It hashes UTF-8 JSON with sorted keys, compact separators,
and `ensure_ascii=False`, containing `kind`, `title`, `origin`, `revision`,
`visibility`, `evidence_status`, and the SHA-256 of `body`. Collection time is
excluded. Other ID schemes are accepted with the same collision rules.

An edge's identity is `(source, relation, target)` and permanently binds `basis`
and `visibility`. An identical edge is a no-op; conflicting basis or visibility
rejects the batch. Represent source revisions with new item IDs and relationships
to those new snapshots.

The entire import is one SQLite transaction, including FTS updates. A validation,
collision, or dangling-edge failure leaves no partial records. A failed first
transaction can leave an empty database file; it does not leave a usable partial
index. Existing indexes with an unsupported schema are rejected without migration.
Successful output reports added and duplicate item/edge counts plus the exact
JSONL file digest. The database is created with mode `0600` before any content
is written. The import operation reads the complete batch into memory.

## Retrieval boundaries

All tools default to `scope=public`. `private` means private items only, and
`all` includes both. Source lookups give the same error for an unknown ID and
an out-of-scope ID. Relationship lookups filter the requested source, both
endpoints, and the edge. Status counts follow the same scope and do not expose
private collection paths. Scope is a local retrieval filter, not authentication:
this server is intended for the user's local stdio client, not a shared service.

Reads open an existing SQLite URI with `mode=ro`, enable `query_only` and disable
trusted schema, deny database attachments and writable PRAGMAs, and close the
connection after each operation. SQLite locking remains enabled so normal import
and read coordination works; `immutable=1` is deliberately not used.

Search uses up to 20 literal word terms joined with AND, not caller-supplied FTS
syntax. Results are bounded to 30; source pages to 24,000 characters; related
links to 100. Search returns provenance, collection time, content digest and an
excerpt; full source retrieval returns the same source metadata and a bounded
page. The index does not fetch URLs, execute retrieved code, or decide which
historical claims are current.

The focused behavior check is:

```sh
PYTHONPATH=tools/uor-knowledge/src python3 -m unittest discover -s tools/uor-knowledge/tests -v
```
