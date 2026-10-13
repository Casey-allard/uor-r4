# Response to Referee Report (Amir Rahnama)

Dear Referee,

Thank you for your detailed and insightful "Theory-Only" report. We have carefully reviewed your feedback, particularly regarding the "finite-to-infinite Pick step" and the "Schur/Herglotz pinch" condition. We agree with your assessment that the previous draft's reliance on a simple tail bound to prove infinite positivity was insufficient.

We have updated **Paper 1 (`paper1_farfield.tex`)** to address these points directly. Below is a summary of the changes (marked in blue in the revised manuscript):

### 1. Finite-to-Infinite Pick Step (The "Hard Blocker")

**Referee Critique:** The implication "finite Pick gap + tail bound $\Rightarrow$ infinite positivity" was not rigorously certified. The sketched operator-norm argument lacked the necessary block-bound quantification for a Schur complement proof.

**Our Resolution:**
We have **removed the reliance** on the infinite Pick extension for the main theorem.
*   **Hybrid Strategy:** The proof of Theorem 1 now relies primarily on the **Rectangle Certification** (rigorous ball arithmetic on $[0.6, 0.7] \times [-20, 20]$) combined with standard **Analytic Asymptotics** for $|t| > 20$. This path is fully unconditional and does not require infinite operator theory.
*   **Pick as Corroboration:** We retained the finite Pick certificate (at $\sigma_0=0.599, 0.6, 0.7$) but explicitly downgraded its role to a **"local robustness check"** (Proposition 10). We added text clarifying that while the finite gap is strong evidence, the rigorous global proof rests on the Hybrid Strategy.
*   **Removed Flawed Lemma:** We removed the problematic "Tail-to-infinite stability" lemma and the claim that the tail bound proves infinite positivity.

### 2. Schur/Herglotz Pinch (The "Minor Fix")

**Referee Critique:** The pinch argument requires explicitly excluding the degenerate case $\Theta \equiv 1$.

**Our Resolution:**
*   We updated **Corollary 5 (`cor:no-poles`)** and its proof to explicitly invoke the **Maximum Modulus Principle dichotomy**: if $\Theta(s_0)=1$, then $\Theta \equiv 1$.
*   We added a concrete exclusion: on the relevant half-planes, we know $\Theta(s) \to 1/3$ as $\Re s \to +\infty$ (from the normalization of $\mathcal J$), which rules out the constant case $\Theta \equiv 1$.

We believe these changes fully resolve the theoretical gaps identified in your report while preserving the unconditional strength of the result via the alternative (Hybrid) rigorous path.

Sincerely,
Jonathan Washburn
