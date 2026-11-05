# CPU hierarchical copy-key depth screen

This lab record contains three sealed attempts:

- [`attempt-1.md`](attempt-1.md): the public-DEV command used normalized panel
  filenames and stopped before any target reading.
- [`attempt-2.md`](attempt-2.md): token and mask names were correct but the
  manifest still used the normalized panel name; it stopped before any target
  reading.
- [`attempt-3.md`](attempt-3.md): the registered run completed on two newly
  generated panels.

The successful evidence is in
[`evidence-attempt-3/`](evidence-attempt-3/). Its raw manifest, preserved as
`RAW-SHA256SUMS`, has digest
`bec19988e1b161973de1037f444ffc1516093f2bd33f54df4a35ee52a09c3a22`.

The full sealed attempts remain in the local CPU pipeline. Generated corpora
are omitted from Git under CHARTER §3.11. Each published evidence directory
therefore carries `RAW-SHA256SUMS`, byte-identical to the sealed attempt, and
`PUBLISHED-SHA256SUMS`, covering every committed evidence file. Panel
manifests preserve both corpus hashes and generator seeds.
