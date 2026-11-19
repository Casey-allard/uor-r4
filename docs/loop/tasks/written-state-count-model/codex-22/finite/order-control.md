# Same-multiset finite clause proof

Use the established frozen D11 paired parent, tokenizer and role state for both seeds. The four user messages below are frozen before projection. The first and fourth select Rosalind. The second and third select Benedict. Each prompt starts with the same two facts.

1. Facts. Rosalind owns glass jars. Benedict owns wooden brushes. please keep rather than drop Rosalind; please drop rather than keep Benedict.
2. Facts. Rosalind owns glass jars. Benedict owns wooden brushes. please drop rather than keep Rosalind; please keep rather than drop Benedict.
3. Facts. Rosalind owns glass jars. Benedict owns wooden brushes. please keep rather than drop Benedict; please drop rather than keep Rosalind.
4. Facts. Rosalind owns glass jars. Benedict owns wooden brushes. please drop rather than keep Benedict; please keep rather than drop Rosalind.

Use literal-role protocol 2, preserve original token positions, and use only each candidate's own half-open clause as evidence. Exclude the semicolon and terminal period. Validate original names and separate assertion support. Validate that opposed clauses contain the same raw D11 root multiset outside the mention before comparing the vectors. Compare original typed Identity and Positional 32-integer vectors on each same-person polarity flip and clause-order-only flip. Preserve equality, distances, all coordinates and all input pins regardless of outcome. No wording substitution, fit or prediction.
