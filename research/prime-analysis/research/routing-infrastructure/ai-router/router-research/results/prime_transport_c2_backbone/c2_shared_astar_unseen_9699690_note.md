# Shared C^2 transport law on unseen W=9699690

Setup:
- train wheels: 2310, 30030, 510510
- test wheel: 9699690
- exact torus modes with kb,kf in [-3,3], excluding (0,0)
- deterministic gauge fixing applied to the top-2 complex SVD basis

Test comparison:
- err_test = 0.278267864521
- err_star = 0.281905374440
- err_star / err_test = 1.013071972668

Eigenvalues:
- A_test: (0.936623750126586-0.028572996369631398j), (0.9833365762038013+0.004837307236634641j)
- A_*: (0.9391973066188745-0.039723900660880274j), (0.9772077983103942+0.0080937223723965j)

- conjugation-aligned distance(A_*, A_test) = 0.013078988998

Conclusion: A_* generalizes acceptably at the unseen scale.
