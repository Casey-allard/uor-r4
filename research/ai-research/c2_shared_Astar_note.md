# Shared canonical A_* test for the C^2 quotient

For W = 2310, 30030, 510510:
1. build the C^2 quotient from low-order torus harmonics
2. fit scale-specific transport matrices A_r
3. fit one shared canonical transport matrix A_* on pooled quotient data
4. compare the transport error of A_* vs each scale-specific A_r

Interpretation:
- if A_* performs close to A_r on each scale, then the quotient has a reusable global transport law
- if A_* is much worse, the quotient remains scale-specific
