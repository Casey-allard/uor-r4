Here is the translation of the paper into English, with the mathematics formatted in LaTeX.

***

**Math-Net.Ru**
**All-Russian mathematical portal**

V. V. Kapustin, Schrödinger Operator with Morse Potential and Zeros of the Riemann Zeta Function, *Mat. Zametki*, 2022, Volume 111, Issue 2, 304–307

https://www.mathnet.ru/eng/mzm13363

***

**Schrödinger Operators with Morse Potentials and Zeros of the Riemann Zeta Function**

**V. V. Kapustin**

**Keywords:** Riemann xi-function, de Branges space, one-dimensional perturbations of self-adjoint operators.

**DOI:** https://doi.org/10.4213/mzm13363

In article [1], Schrödinger operators with Morse potentials on the right half-line were studied. In some cases, their spectra model the set of non-trivial zeros of the Riemann zeta-function $\zeta$ in the sense of estimates for the number of zeros depending on the bound for the imaginary part. In this note, it will be shown that an *exact* coincidence of the operator's spectrum with the set of zeros of the zeta-function (after a simple transformation of the complex plane) can be achieved using a one-dimensional perturbation of the operator.

Fix a real number $\nu > 0$ such that $\zeta$ has a simple root at the point $1/2 + i\nu$. Consider the set
$$ Z = \left\{ \lambda : \text{Re } \lambda > 0, \; \zeta\left(\frac{1}{2} + i\lambda\right) = 0 \right\} \setminus \{\nu\}. $$
It is well known that if $\lambda \in Z$, then $|\text{Im } \lambda| < 1/2$, and the famous Riemann hypothesis regarding the zeros of the zeta-function is equivalent to the statement that $Z$ is a subset of the real line.

**Theorem 1.** *Let $Q$ be the Schrödinger operator $Qu = -u'' + qu$ with the Morse potential $q(x) = (1/4)e^{2x} + ke^x$ on the half-line $(x_0, +\infty)$, where $x_0 = \log(4\pi)$, and $k = 1/2$ or $k = -1/2$. For any self-adjoint boundary condition at the point $x_0$, we obtain a self-adjoint operator $Q$ with a discrete spectrum; assume that the spectrum of operator $Q$ does not contain the point $0$. Then there exists a one-dimensional perturbation of the bounded self-adjoint operator $Q^{-1}$, the spectrum of which coincides with the set $\{4/\lambda^2 : \lambda \in Z\}$.*

In the exceptional case where $0$ is an eigenvalue, the result holds in terms of a perturbation of the resolvent.

**Proof.** The proof is based on the results of article [2].
Let us denote by $\mathscr{E}$ the entire function $\mathscr{E}(z) = 2K_{(1-iz)/2}(2\pi)$, where $K$ is the modified Bessel function defined in the standard way. In article [2], the de Branges space $\mathscr{H}_{\mathscr{E}}$ with the structure function $\mathscr{E}$ was considered. For $\text{Im } z > 0$, we have $|\mathscr{E}(z)| < |\mathscr{E}(\bar{z})|$, and the space $\mathscr{H}_{\mathscr{E}}$ is defined as the set of all entire functions $F$ such that the functions $F(z)/\mathscr{E}(z)$ and $F(\bar{z})/\mathscr{E}(z)$ belong to the Hardy space $H^2$ in the upper half-plane $\{\text{Im } z > 0\}$. The norms of these functions in $H^2$ coincide and define the norm of the function $F$ in $\mathscr{H}_{\mathscr{E}}$.

Let us define the entire function
$$ \phi(z) = \frac{\nu^2}{\xi(1/2)} \cdot \frac{\xi(1/2 - iz)}{\nu^2 - z^2}, $$
where $\xi$ is the Riemann xi-function,
$$ \xi(s) = \frac{1}{2} s(s-1) \pi^{-s/2} \Gamma\left(\frac{s}{2}\right) \zeta(s). $$

***

*(Page 3)*

In [2], it is proven that the map $T$, defined by the equality
$$ (TF)(z) = \frac{1}{z}(F - F(0)\phi), $$
is a bounded operator in $\mathscr{H}_{\mathscr{E}}$. The function $\phi$ is even, and the set of its zeros is the union of the sets $Z$ and $-Z = \{-\lambda : \lambda \in Z\}$. The spectrum of $T$ coincides with the set $\{1/\lambda : \phi(\lambda) = 0\}$; the corresponding eigenfunctions have the form $\phi/(z - \lambda)$.

The square of the operator $T$ acts according to the formula
$$ (T^2 F)(z) = \frac{F - (F(0) + zF'(0))\phi}{z^2}. $$
If $F$ is an even function, then
$$ (T^2 F)(z) = \frac{F - F(0)\phi}{z^2}, $$
and if $F$ is odd, then
$$ (T^2 F)(z) = \frac{F - F'(0)z\phi}{z^2}. $$
Consequently, the operator $T^2$ decomposes into a direct sum of its parts on the even and odd subspaces of $\mathscr{H}_{\mathscr{E}}$. For each of these parts, the spectrum is the set $\{1/\lambda^2 : \lambda \in Z\}$, and the eigenfunctions have the form $\phi/(z^2 - \lambda^2)$ and $z\phi/(z^2 - \lambda^2)$, respectively.

Let us utilize the results of article [2] related to the canonical system considered therein. For $t < 0$, define
$$ A(t, z) = \sqrt{\frac{-t}{2\pi}} e^{-t} (K_s(-t) + K_{s-1}(-t)), \quad B(t, z) = -i\sqrt{\frac{-t}{2\pi}} e^t \cdot (K_s(-t) - K_{s-1}(-t)), $$
where $K_s$ is the modified Bessel function, and $s = s(z) = (1 - iz)/2$. The operators $V_\pm$,
$$ (V_+ f_+)(z) = \frac{1}{\sqrt{\pi}} \int_{-\infty}^{-2\pi} f_+(t) A(t, z) w_+(t) \, dt, $$
$$ (V_- f_-)(z) = \frac{1}{\sqrt{\pi}} \int_{-\infty}^{-2\pi} f_-(t) B(t, z) w_-(t) \, dt $$
isometrically map the weighted spaces $L^2((-\infty, -2\pi), w_+)$ and $L^2((-\infty, -2\pi), w_-)$ with
$$ w_+(t) = \frac{e^{2t}}{-2t}, \quad w_-(t) = \frac{e^{-2t}}{-2t} $$
onto the even and odd subspaces of $\mathscr{H}_{\mathscr{E}}$, respectively; cf. formula (4) in [2]. To construct operators in these weighted $L^2$-spaces with the spectrum $\{1/\lambda^2 : \lambda \in Z\}$, we apply a unitary transplantation from the de Branges space using the operators $V_\pm$.

Consider the Sturm–Liouville operators
$$ \mathscr{S}_+ f_+ = -\frac{1}{w_+} \cdot \frac{d(w_+^{-1} \dot{f}_+)}{dt}, \quad \mathscr{S}_- f_- = -\frac{1}{w_-} \cdot \frac{d(w_-^{-1} \dot{f}_-)}{dt} $$
in the weighted spaces $L^2((-\infty, -2\pi), w_+)$ and $L^2((-\infty, -2\pi), w_-)$, respectively. For each of them, if we impose a self-adjoint boundary condition at the point $-2\pi$, we obtain a self-adjoint operator. The goal is to show that if $0$ is not a point of the spectrum, then a one-dimensional perturbation of the inverse operator can have the spectrum $\{1/\lambda^2 : \lambda \in Z\}$.

Integrating by parts and using the relations
$$ \dot{B} = -zw_+ A, \quad \dot{A} = zw_- B, $$

***

*(Page 4)*

one can calculate the expressions for $V_\pm \mathscr{S}_\pm f_\pm$. We have
$$ (V_+ \mathscr{S}_+ f_+)(z) = z^2 \cdot (V_+ f_+)(z) + \frac{1}{\sqrt{\pi}} \left( f_+(-2\pi) \cdot zB(-2\pi, z) - \frac{\dot{f}_+(-2\pi)}{w_-(-2\pi)} A(-2\pi, z) \right). $$
In particular,
$$ F(0) = -\frac{1}{\sqrt{\pi}} \frac{\dot{f}_+(-2\pi)}{w_-(-2\pi)}, $$
and for $F = V_+ \mathscr{S}_+ f_+$, we obtain the formula
$$ (V_+ f_+)(z) = \frac{F(z) - F(0)\psi}{z^2} $$
where
$$ \psi(z) = A(-2\pi, z) - \frac{f_+(-2\pi)}{\dot{f}_+(-2\pi)} \cdot w_-(-2\pi) \cdot zB(-2\pi, z), $$
provided $\dot{f}_+(-2\pi) \neq 0$. If we impose the self-adjoint boundary condition $f_+(-2\pi) = h \cdot \dot{f}_+(-2\pi)$ for real $h$, we obtain a formula in which $\psi$ is a real linear combination of the even functions $A(-2\pi, z)$ and $zB(-2\pi, z)$. (The self-adjoint boundary condition $\dot{f}_+(-2\pi) = 0$ is exceptional, since in that case constant functions belong to the kernel of the operator.)

Thus, for any real $h$, the inverse operator to the Sturm–Liouville operator with the self-adjoint boundary condition $f_+(-2\pi) = h \cdot \dot{f}_+(-2\pi)$ is unitarily equivalent to the operator $F \mapsto (F(z) - F(0)\psi)/z^2$ in the even subspace of the space $\mathscr{H}_{\mathscr{E}}$. It is clear that the restriction of the operator $T^2$ to this subspace, obtained by replacing $\phi$ with $\psi$, is its one-dimensional perturbation; it possesses the required spectrum.

Note that the formula for $\psi$ gives a description of the spectrum of the unperturbed operator: $\lambda^{-2}$ belongs to the spectrum if and only if $\psi(\lambda) = 0$. In particular, for $h=0$ we have $\psi(z) = A(-2\pi, z)$, and the set of zeros of $\psi$ has the form
$$ \{ \lambda : K_{(1-i\lambda)/2}(2\pi) + K_{(1+i\lambda)/2}(2\pi) = 0 \}. $$
Analogously, if $F = V_- \mathscr{S}_- f_-$, then
$$ F(z) = z^2 \cdot (V_- f_-)(z) + \frac{1}{\sqrt{\pi}} \left( f_-(-2\pi) \cdot zA(-2\pi, z) + \frac{\dot{f}_-(-2\pi)}{w_+(-2\pi)} B(-2\pi, z) \right); $$
for a self-adjoint boundary condition at $-2\pi$, one can write $V_- f_-$ in the form
$$ (V_- f_-)(z) = \frac{F - F'(0)z\psi}{z^2}, $$
where $\psi$ is a linear combination of the functions $A$ and $(1/z)B$. Replacing $\psi$ with $\phi$, we obtain a one-dimensional perturbation with the spectrum $\{1/\lambda^2 : \lambda \in Z\}$.

The reduction of an arbitrary Sturm–Liouville operator to a Schrödinger operator is carried out by the Liouville transformation. We use its modified version corresponding to the change of variable $t = -(1/2)e^x$. By direct calculation, it is verified that the operators
$$ f_+ \mapsto \frac{1}{\sqrt{2}} e^{-(1/2)e^x} f_+ \left( -\frac{e^x}{2} \right), \quad f_- \mapsto \frac{1}{\sqrt{2}} e^{(1/2)e^x} f_- \left( -\frac{e^x}{2} \right) $$
isometrically map the spaces $L^2((-\infty, -2\pi), w_\pm)$ onto the unweighted $L^2$-space on the interval $(\log(4\pi), +\infty)$, and the unitary copies of the Sturm–Liouville operators with a self-adjoint boundary condition at $-2\pi$ are the operators $4Q$, where $Q$ is the operator referred to in the theorem. The preceding reasoning shows that the operator with spectrum $\{1/\lambda^2 : \lambda \in Z\}$ is obtained by a one-dimensional perturbation of the operator $(4Q)^{-1}$. The assertion of the theorem follows immediately from this.

The author is very grateful to R. V. Romanov for valuable discussions.

***

*(Page 5)*

**REFERENCES**

[1] J. C. Lagarias, *Commun. Number Theory Phys.*, **3**:2 (2009), 323–361.
[2] V. V. Kapustin, *Algebra i Analiz*, **33**:4 (2021), 107–124. [St. Petersburg Math. J., **33**:4 (2022), 569–581].

**V. V. Kapustin**
St. Petersburg Department of Steklov Mathematical Institute of the Russian Academy of Sciences
E-mail: kapustin@pdmi.ras.ru

Received: 24.07.2021
Accepted for publication: 05.08.2021
