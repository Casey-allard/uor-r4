Here is the full translation of the paper, with mathematical expressions formatted in LaTeX.

***

**Algebra i Analiz**
**Tom 33 (2021), No 4**

**Dedicated to the memory of Vladimir Marikhin**

# THE SET OF ZEROS OF THE RIEMANN ZETA FUNCTION AS THE POINT SPECTRUM OF AN OPERATOR

**© V. V. KAPUSTIN**

> One of the possible ways one could attempt to prove the Riemann hypothesis consists of constructing a self-adjoint operator whose spectrum coincides with the set $\{z : |\text{Im}\, z| < \frac{1}{2}, \zeta(\frac{1}{2} - iz) = 0\}$. In this paper, a one-dimensional perturbation of a self-adjoint operator associated with a certain canonical system is constructed, for which a completely analogous property holds.

The main object of the paper is a canonical system on the interval $(-\infty, -2\pi)$ with the Hamiltonian
$$ H(t) = \begin{pmatrix} \frac{e^{2t}}{-2t} & 0 \\ 0 & \frac{e^{-2t}}{-2t} \end{pmatrix}. \quad (1) $$

A canonical system is the equation
$$ J \dot{f}(t) = z H(t) f(t) $$
on an interval of the real line, where $f = \begin{pmatrix} f_+ \\ f_- \end{pmatrix}$ is a function of $t$, $z$ is the spectral parameter, $J = \begin{pmatrix} 0 & -1 \\ 1 & 0 \end{pmatrix}$, and $H$ is the Hamiltonian, representing a locally summable non-negative matrix-valued $(2 \times 2)$-function. In connection with the theory of canonical systems, the reader is referred to the book [1]. In the standard case, the Hamiltonian is summable near the left end of the interval; however, generally speaking, this may not be so; in our case, the Hamiltonian is non-summable near $-\infty$.

*Key words:* Riemann xi-function, de Branges space, canonical system.
*The work was partially supported by the RFBR grant No 19-01-00565a.*

---
**Page 108**

The Hilbert space of the canonical system is a weighted space of pairs of functions on the considered interval, for which the Hamiltonian acts as the weight (the definition looks more complicated in the general situation where the Hamiltonian has so-called indivisible intervals). The operator of the canonical system $\mathcal{L}$ is defined by the formula
$$ (\mathcal{L}f)(t) = H(t)^{-1} J \dot{f}(t) $$
and is formally symmetric.

The Hamiltonian (1) on the interval $(-\infty, -2\pi)$ is a special case of a diagonal Hamiltonian of the form
$$ H(t) = \begin{pmatrix} h_+(t)^2 & 0 \\ 0 & h_-(t)^2 \end{pmatrix} \quad (2) $$
on an interval $(-\infty, a)$, where the positive functions $h_+, h_-$ are such that
(p1) $h_+^2 \in L^1(-\infty, a)$;
(p2) $h_-^2 \notin L^1(-\infty, a)$ and $h_-^2 \in L^1(a-1, a)$ (that is, summability holds near the right end of the interval and fails near the left).

The right end of the interval is regular, and a boundary condition is imposed there. The standard condition $f_-(a) = 0$ yields an operator with a non-trivial kernel containing the element
$$ e = \begin{pmatrix} 1 \\ 0 \end{pmatrix} $$
of the canonical system space. For $\nu \in \mathbb{C}$, we define the operator $L_\nu$ as the restriction of the mapping $\mathcal{L}$ to the domain of definition given by the boundary condition
$$ f_+(a) = \nu f_-(a). $$
At the left end $-\infty$, we have the limit point case. Consequently, the operators $L_\nu$ for real $\nu$ are self-adjoint. For canonical systems with Hamiltonians of the form (2) possessing properties (p1), (p2), we obtain the following proposition.

**Proposition 1.** *For any complex number $\nu$, $L_\nu$ is an (unbounded) closed operator in the canonical system space. If $\nu$ is a real number, then $L_\nu$ is a self-adjoint operator.*

Condition
(p3)
$$ \lim_{x \to -\infty} \left( \int_{-\infty}^x h_+(t)^2 \, dt \times \int_{x}^a h_-(t)^2 \, dt \right) = 0 $$

---
**Page 109**

is equivalent to $H$ being a structural Hamiltonian for a chain of de Branges spaces. An equivalent condition for the string operator, corresponding to the case of diagonal Hamiltonians, was obtained in work [2]; in article [3], this result was extended to the general case, and the criterion is presented in the form given above. For real $\nu$, the self-adjoint operators $L_\nu$ have discrete spectra. Property (p3) is obviously satisfied in our case for the Hamiltonian (1).

We have $\mathcal{L}e = 0$, and 0 is an isolated point of the spectrum of the restriction of the operator $\mathcal{L}$ to the class of functions defined by the boundary condition
$$ f_-(-2\pi) = 0. $$
For the considered operator, it follows from this that for any complex number $\nu$, the kernel of the operator $L_\nu$ is trivial and its spectrum does not contain the point 0. This allows us to define bounded operators $T_\nu$ as the inverses of $L_\nu$:
$$ T_\nu = L_\nu^{-1}. $$
In the main theorem, we will only need the case $\nu = 0$.
Let us fix a complex number $\alpha$ with real part $\frac{1}{2}$, at which the Riemann zeta function has a simple root: $\zeta(\alpha) = 0$.

**Main Theorem.** *There exists a real-valued function $\gamma$ on $(-\infty, -2\pi)$, for which $\begin{pmatrix} 0 \\ \gamma \end{pmatrix}$ is an element of the canonical system space on $(-\infty, -2\pi)$ with Hamiltonian (1), and such that the operator*
$$ T = T_0 + \left( *, \begin{pmatrix} 1 \\ 0 \end{pmatrix} \right) \begin{pmatrix} 0 \\ \gamma \end{pmatrix} $$
*has a purely point spectrum coinciding with the set*
$$ \left\{ \frac{1}{z} : \frac{1}{2} - iz \in \{ s : \text{Re}\, s \in (0, 1), \zeta(s)=0 \} \setminus \{ \alpha, \bar{\alpha} \} \right\}. $$

It is clear that the Riemann hypothesis is equivalent to the statement that the spectrum of the operator $T$ lies on the real line.
One can verify that
$$ T_\nu = T_0 - \nu \left( *, \begin{pmatrix} 1 \\ 0 \end{pmatrix} \right) \begin{pmatrix} 1 \\ 0 \end{pmatrix}, \quad (3) $$
see Sec. 6.3. Consequently, $T$ is a one-dimensional perturbation of the self-adjoint operator $T_\nu$ for any real $\nu$.
Clarification of the form of the function $\gamma$ is proposed in subsequent works.

---
**Page 110**

R. Romanov drew the author's attention to the connection of the canonical system with Hamiltonian (1) with article [4], in which Schrödinger operators with Morse potentials are considered. In addition, the author is grateful to R. Romanov for valuable discussions related to canonical systems and Bessel functions, as well as to all colleagues who showed interest in this work.

### §1. Chain of de Branges spaces

In this section, we will consider canonical systems of general form on an interval of the type $(-\infty, a)$, whose Hamiltonians have the form (2) and possess properties (p1)–(p3).

Let an entire function $\mathcal{E}$ belong to the Hermite-Biehler class, that is, $|\mathcal{E}(\bar{z})| < |\mathcal{E}(z)|$ for all $z$ from the upper half-plane. The de Branges space $\mathcal{H}_{\mathcal{E}}$ with structural function $\mathcal{E}$ consists of all entire functions $F$ for which the functions $\frac{F}{\mathcal{E}}$ and $\frac{F^\#}{\mathcal{E}}$ belong to the Hardy class $H^2$ in the upper half-plane, where the notation
$$ F^\#(z) = \overline{F(\bar{z})} $$
is used. The norms of the functions $\frac{F}{\mathcal{E}}$ and $\frac{F^\#}{\mathcal{E}}$ in $H^2$ coincide, and they define the norm of the function $F$ in the space $\mathcal{H}_{\mathcal{E}}$.

Any de Branges space possesses a maximal chain of de Branges subspaces. The structure of a de Branges space, corresponding to the ordering of its de Branges subspaces by inclusion, is expressed in terms of a canonical system, see Proposition 3 below. The structural function of the chain of spaces associated with a canonical system is the function $E(t, z)$, defined for $t \in (-\infty, a]$. For each $t$, this is an entire function of $z$ belonging to the Hermite-Biehler class, and thus for each $t$, we have a corresponding de Branges space; the function $E(a, z)$ corresponds to the initial de Branges space. Besides this, for
$$ A(t, z) = \frac{1}{2} \left( E(t, z) + E^\#(t, z) \right), \quad B(t, z) = \frac{1}{2i} \left( E(t, z) - E^\#(t, z) \right), $$
where $E^\#(t, z) = \overline{E(t, \bar{z})}$, the vector-function $\begin{pmatrix} A \\ B \end{pmatrix}$ is a solution to the canonical system. It is clear that
$$ E = A + iB. $$
Let us assume also that
(p4) $E(t, z) \to 1$ (which is equivalent to $A(t, z) \to 1$ and $B(t, z) \to 0$) as $t \to -\infty$ for each $z$.

---
**Page 111**

From the equation of the canonical system, it follows that for $z=0$ its solution is a constant function of $t$. Therefore, from condition (p4), we obtain that $A(t, 0) = 1, B(t, 0) = 0$ (or, otherwise, $E(t, 0) = 1$) for all $t$.
It would be interesting to find out how general property (p4) is for canonical systems whose Hamiltonians satisfy conditions (p1)–(p3).

**Proposition 2.** *Suppose that $\begin{pmatrix} A \\ B \end{pmatrix}$ is such a solution of the canonical system that for the function $E = A + iB$, property (p4) holds. Then $E$ belongs to the Hermite-Biehler class as a function of $z$ for each $t$.*

The proof coincides with the proof of Proposition 4 of work [1], one only needs to replace the left end of the interval with $-\infty$. For the convenience of the reader, a sketch of the proof will be given in Sec. 6.1.

With each element $f = \begin{pmatrix} f_+ \\ f_- \end{pmatrix}$ of the canonical system space, an entire function $Vf$ is associated,
$$ (Vf)(z) = \frac{1}{\sqrt{\pi}} \int_{-\infty}^a \left( f_+(t) A(t, z) h_+^2(t) + f_-(t) B(t, z) h_-^2(t) \right) \, dt. \quad (4) $$
This is an element of the de Branges space corresponding to the canonical system, whose structural function is
$$ \mathcal{E}(z) = E(a, z) = A(a, z) + iB(a, z). $$
In the following proposition, it is asserted that the general fact of the theory of de Branges spaces for canonical systems with a regular left end of the interval (when the Hamiltonian is summable near the left end) is true in our case as well.

**Proposition 3.** *The operator $V$ isometrically maps the canonical system space onto $\mathcal{H}_{\mathcal{E}}$.*

A sketch of the proof will be given in Sec. 6.2.

### §2. One-dimensional perturbations

In Sec. 6.3, some auxiliary formulas will be established. In particular, it will be proven that for $F \in \mathcal{H}_{\mathcal{E}}$ we have
$$ (V T_0 V^* F)(z) = \frac{1}{z} \left( F(z) - F(0) \frac{\mathcal{E}(z) + \mathcal{E}^\#(z)}{2} \right), \quad (5) $$

---
**Page 112**

where $\mathcal{E}^\#(z) = \overline{\mathcal{E}(\bar{z})}$, which is a direct analogue of the same formula for the case when the left end of the interval is regular.

**Lemma 4.** *Take an element $\omega$ of the canonical system space and consider the operator*
$$ T = T_0 + (*, e) \, \omega, $$
*where $e = \begin{pmatrix} 1 \\ 0 \end{pmatrix}$. Then $T$ is unitarily equivalent to the operator in $\mathcal{H}_{\mathcal{E}}$ acting by the formula*
$$ F \mapsto \frac{F - F(0)\varphi}{z} $$
*where*
$$ \varphi = \frac{\mathcal{E} + \mathcal{E}^\#}{2} - \sqrt{\pi} z V \omega. $$

**Proof.** From the definition of the function $Vf$, we have
$$ (Vf)(0) = \frac{1}{\sqrt{\pi}} \int_{-\infty}^a f_+(t) h_+^2(t) \, dt = \frac{1}{\sqrt{\pi}} (f, e). \quad (6) $$
By formula (6) for $F = Vf \in \mathcal{H}_{\mathcal{E}}$, we obtain
$$ (V^* F, e) = (f, e) = \sqrt{\pi} (Vf)(0) = \sqrt{\pi} F(0). $$
Now using formula (5), we can write
$$ \begin{aligned} V T V^* F &= V T_0 V^* F + (V^* F, e) V \omega \\ &= \frac{1}{z} \left( F - F(0) \frac{\mathcal{E} + \mathcal{E}^\#}{2} \right) + \sqrt{\pi} F(0) V \omega = \frac{F - F(0)\varphi}{z} \end{aligned} $$
with the required function $\varphi$. $\square$

We have $E(t, 0) = 1$ for any $t$; in particular, $\mathcal{E}(0) = E(a, 0) = 1$. Thus,
$$ \varphi(0) = 1, \quad \frac{1}{z} \left( \frac{\mathcal{E} + \mathcal{E}^\#}{2} - \varphi \right) = \sqrt{\pi} V \omega \in \mathcal{H}_{\mathcal{E}}. $$
The operator $T$ from the main theorem can be written in the form
$$ T = T_0 + (*, e) \, \omega, $$
as in Lemma 4 with $\omega = \begin{pmatrix} 0 \\ \gamma \end{pmatrix}$.

**Lemma 5.** *For a complex number $\lambda \neq 0$, the point $\frac{1}{\lambda}$ belongs to the spectrum of the operator $T = T_0 + (\cdot, e) \omega$ if and only if $\varphi(\lambda) = 0$; at such points, the subspace of eigenvectors has dimension 1.*

---
**Page 113**

*In particular, for $\omega = 0$, it is seen from the lemma that the spectrum of the operator $T_0$ is the set $\{ \frac{1}{\lambda} : \mathcal{E}(\lambda) + \mathcal{E}^\#(\lambda) = 0 \}$; in other words, the spectrum of the operator $L_0$ coincides with the set of zeros of the function $\mathcal{E} + \mathcal{E}^\#$.*

**Proof.** By Lemma 4, the operator $T$ is unitarily equivalent to the operator $F \mapsto \frac{F - F(0)\varphi}{z}$ in $\mathcal{H}_{\mathcal{E}}$.
For $\lambda \neq 0$, if $\varphi(\lambda) = 0$, then
$$ \frac{\varphi}{z - \lambda} = \frac{1}{z - \lambda} \left( \frac{\mathcal{E} + \mathcal{E}^\#}{2} - \sqrt{\pi} z V \omega \right) = \frac{1}{z - \lambda} \left( \frac{\mathcal{E} + \mathcal{E}^\#}{2} - \sqrt{\pi} \lambda V \omega \right) - \sqrt{\pi} V \omega $$
is an element of the space $\mathcal{H}_{\mathcal{E}}$, wherein
$$ \frac{\varphi}{z - \lambda} \to \frac{\frac{\varphi}{z-\lambda} + \frac{1}{\lambda}\varphi}{z} = \frac{1}{\lambda} \frac{\varphi}{z - \lambda}; $$
consequently, $\frac{1}{\lambda}$ is an eigenvalue. Conversely, from the equation for eigenvectors, it is obtained that the eigenfunction can only have such a form, and then $\varphi(\lambda) = 0$.
If $\varphi(\lambda) \neq 0$, then the resolvent acts as $F \mapsto \frac{z F(z) - \frac{\lambda F(\lambda)}{\varphi(\lambda)} \cdot \varphi(z)}{1 - \frac{z}{\lambda}}$. $\square$

Jordan blocks exist at points where the zeros of $\varphi$ are multiple. The point 0 belongs to the spectrum when the space is infinite-dimensional, and is an eigenvalue if and only if $\varphi \in \mathcal{H}_{\mathcal{E}}$.

### §3. Structural function of the chain of de Branges spaces

The Hamiltonian $H$, defined by formula (1) on the interval $(-\infty, -2\pi)$, is a special case of a diagonal Hamiltonian on $(-\infty, a)$ with $a = -2\pi$, having the form (2) for
$$ h_+(t) = \frac{e^t}{\sqrt{-2t}}, \quad h_-(t) = \frac{e^{-t}}{\sqrt{-2t}} $$
and possessing properties (p1)–(p3).
One can rewrite the canonical system in the form
$$ \dot{f}_+(t) = z h_-^2(t) f_-(t), \quad \dot{f}_-(t) = -z h_+^2(t) f_+(t). $$
The space of the canonical system has the form
$$ \left\{ \begin{pmatrix} f_+ \\ f_- \end{pmatrix} : f_+ h_+, f_- h_- \in L^2(-\infty, a) \right\}. $$
The differential operator of the canonical system acts according to the formula
$$ \mathcal{L} \begin{pmatrix} f_+ \\ f_- \end{pmatrix} = \begin{pmatrix} h_+^{-2} & 0 \\ 0 & h_-^{-2} \end{pmatrix} \begin{pmatrix} 0 & -1 \\ 1 & 0 \end{pmatrix} \begin{pmatrix} \dot{f}_+ \\ \dot{f}_- \end{pmatrix} = \begin{pmatrix} -h_+^{-2} \dot{f}_- \\ h_-^{-2} \dot{f}_+ \end{pmatrix}. $$

---
**Page 114**

For $t < 0$, set
$$ A(t, z) = \sqrt{\frac{-t}{2\pi}} e^{-t} \left( K_s(-t) + K_{s-1}(-t) \right), $$
$$ B(t, z) = -i \sqrt{\frac{-t}{2\pi}} e^{t} \cdot \left( K_s(-t) - K_{s-1}(-t) \right), $$
where $K_s$ is the modified Bessel function, and the notation
$$ s = s(z) = \frac{1 - iz}{2} $$
is used.
Here and in what follows, if $s$ appears in expressions defining a function of $z$, this formula should be applied.
The definition and properties of the Bessel function can be found, for example, in Chapter 7 of the book [5]. Only a few relations for them will be required here, and appropriate references will be given for each.
We have $K_s = K_{-s}$, see [5, formula (7.2.14)]. For real values of the argument $t$, we have
$$ K_s^\#(t) = \overline{K_{\frac{1-i\bar{z}}{2}}(t)} = K_{\frac{1+iz}{2}}(t) = K_{1-s}(t) = K_{s-1}(t). $$
For $s = \frac{1}{2}$, we have $K_{1/2}(-t) = \sqrt{\frac{\pi}{-2t}} e^t$, see [5, formula (7.2.42)]; consequently,
$$ A(t, 0) = \sqrt{\frac{-t}{2\pi}} e^{-t} \cdot 2 K_{1/2}(-t) = 1, \quad B(t, 0) = 0. $$
Let us define
$$ \begin{aligned} E(t, z) &= A(t, z) + iB(t, z) \\ &= \sqrt{\frac{-t}{2\pi}} e^{-t} \left( K_s(-t) + K_{s-1}(-t) \right) + \sqrt{\frac{-t}{2\pi}} e^t \cdot \left( K_s(-t) - K_{s-1}(-t) \right) \\ &= \sqrt{\frac{-t}{2\pi}} \left( (e^{-t} + e^t) K_s(-t) + (e^{-t} - e^t) K_{s-1}(-t) \right). \end{aligned} $$

**Theorem 6.** *The function $E$ is the structural function of the chain of de Branges spaces for the canonical system with the diagonal Hamiltonian*
$$ \begin{pmatrix} \frac{e^{2t}}{-2t} & 0 \\ 0 & \frac{e^{-2t}}{-2t} \end{pmatrix} $$
*for $t < 0$.*

---
**Page 115**

**Proof.** The theorem contains the assertion that the vector-function $\begin{pmatrix} A \\ B \end{pmatrix}$ satisfies the equation of the canonical system:
$$ \dot{A}(t, z) = z \frac{e^{-2t}}{-2t} B(t, z), \quad \dot{B}(t, z) = -z \frac{e^{2t}}{-2t} A(t, z). $$
From relations (7.11.25–26) of book [5], we obtain
$$ \dot{K}_s(-t) = -K_{s-1}(-t) + \frac{s}{t} K_s(-t). $$
Since
$$ \sqrt{2\pi} A(t, z) = \sqrt{-t} e^{-t} \cdot \left( K_s(-t) + K_{s-1}(-t) \right), $$
$$ \sqrt{2\pi} B(t, z) = -i \sqrt{-t} e^t \cdot \left( K_s(-t) - K_{s-1}(-t) \right), $$
we have
$$ \begin{aligned} \sqrt{2\pi} \cdot \dot{A}(t, z) &= \left( \frac{-1}{2\sqrt{-t}} - \sqrt{-t} \right) e^{-t} \cdot \left( K_s(-t) + K_{s-1}(-t) \right) \\ &\quad - \sqrt{-t} e^{-t} \cdot \left( -K_{s-1}(-t) + \frac{s}{t} K_s(-t) - K_{s}(-t) + \frac{1-s}{t} K_{s-1}(-t) \right) \\ &= \frac{e^{-t}}{\sqrt{-t}} \times \left\{ \left( \frac{-1}{2} + t \right) \cdot \left( K_s(-t) + K_{s-1}(-t) \right) \right. \\ &\quad \left. + t \cdot \left( \left( \frac{s}{t} - 1 \right) K_s(-t) + \left( \frac{1-s}{t} - 1 \right) K_{s-1}(-t) \right) \right\} \\ &= \frac{e^{-t}}{\sqrt{-t}} \times \left( s - \frac{1}{2} \right) \cdot \left( K_s(-t) - K_{s-1}(-t) \right) \\ &= \frac{e^{-t}}{\sqrt{-t}} \cdot \frac{-iz}{2} \cdot \frac{\sqrt{2\pi} B(t, z)}{-i \sqrt{-t} e^t} \\ &= \sqrt{2\pi} \cdot z \frac{e^{-2t}}{-2t} B(t, z), \end{aligned} $$
as required. Similarly,
$$ \begin{aligned} \sqrt{2\pi} \cdot \dot{B}(t, z) &= -i \left( \frac{-1}{2\sqrt{-t}} + \sqrt{-t} \right) e^t \cdot \left( K_s(-t) - K_{s-1}(-t) \right) \\ &\quad -i \sqrt{-t} e^t \cdot \left( -K_{s-1}(-t) + \frac{s}{t} K_s(-t) + K_s(-t) - \frac{1-s}{t} K_{s-1}(-t) \right) \end{aligned} $$

---
**Page 116**

$$ \begin{aligned} &= \frac{-ie^t}{\sqrt{-t}} \times \left\{ \left( \frac{-1}{2} - t \right) \cdot \left( K_s(-t) - K_{s-1}(-t) \right) \right. \\ &\quad \left. + t \cdot \left( \left( \frac{s}{t} + 1 \right) K_s(-t) - \left( 1 + \frac{1-s}{t} \right) K_{s-1}(-t) \right) \right\} \\ &= \frac{-ie^t}{\sqrt{-t}} \times \left( s - \frac{1}{2} \right) \cdot \left( K_s(-t) + K_{s-1}(-t) \right) \\ &= \frac{-ie^t}{\sqrt{-t}} \cdot \frac{-iz}{2} \cdot \frac{\sqrt{2\pi} A(t, z)}{\sqrt{-t} e^{-t}} \\ &= -\sqrt{2\pi} \cdot z \frac{e^{2t}}{-2t} A(t, z). \end{aligned} $$
Furthermore, for any $s$, we have $\frac{\sqrt{-t}}{e^t} K_s(-t) \to \sqrt{\frac{\pi}{2}}$ as $t \to -\infty$, see [5], formula (7.4.4) with $M=1$; consequently, $A(t, z) \to 1$. Besides this, $B(t, z) \to 0$ exponentially. It remains to apply Proposition 2. $\square$

### §4. Model de Branges space

For each $t < 0$, the function $E(t, z)$ belongs to the Hermite-Biehler class, so that $\frac{E^\#(t, z)}{E(t, z)}$ is an inner function. We have
$$ \begin{aligned} \frac{E^\#(t, z)}{E(t, z)} &= \frac{\sqrt{\frac{-t}{2\pi}} e^{-t} (K_{s-1}(-t) + K_s(-t)) + \sqrt{\frac{-t}{2\pi}} e^t (K_{s-1}(-t) - K_s(-t))}{\sqrt{\frac{-t}{2\pi}} e^{-t} (K_s(-t) + K_{s-1}(-t)) + \sqrt{\frac{-t}{2\pi}} e^t (K_s(-t) - K_{s-1}(-t))} \\ &= \frac{\left( \frac{K_{s-1}(-t)}{K_s(-t)} + 1 \right) + e^{2t} \cdot \left( \frac{K_{s-1}(-t)}{K_s(-t)} - 1 \right)}{\left( \frac{K_{s-1}(-t)}{K_s(-t)} + 1 \right) - e^{2t} \cdot \left( \frac{K_{s-1}(-t)}{K_s(-t)} - 1 \right)} = \frac{\frac{K_{s-1}(-t)}{K_s(-t)} + \frac{1-e^{2t}}{1+e^{2t}}}{1 + \frac{1-e^{2t}}{1+e^{2t}} \frac{K_{s-1}(-t)}{K_s(-t)}}. \end{aligned} $$
Since $|\frac{1-e^{2t}}{1+e^{2t}}| < 1$, the function $\frac{K_{s-1}(-t)}{K_s(-t)}$ is also an inner function (the formula means that the function $\frac{E^\#(t, z)}{E(t, z)}$ is a so-called Frostman shift of the function $\frac{K_{s-1}(-t)}{K_s(-t)}$). In particular, it is immediately seen from this that $K_s(-t)$ is also a Hermite-Biehler function (as a function of $z$).
At $t = -2\pi$, we obtain
$$ \begin{aligned} \mathcal{E}(z) &= E(-2\pi, z) \\ &= e^{2\pi} \left( K_s(2\pi) + K_{s-1}(2\pi) \right) + e^{-2\pi} \left( K_s(2\pi) - K_{s-1}(2\pi) \right) \\ &= (e^{2\pi} + e^{-2\pi}) \left( K_s(2\pi) + \beta K_{s-1}(2\pi) \right), \quad \text{where } \beta = \frac{1 - e^{-4\pi}}{1 + e^{-4\pi}}. \quad (7) \end{aligned} $$

---
**Page 117**

It is well known that the structural function of a de Branges space is not defined uniquely. The set of all structural functions of a de Branges space with structural function $\mathcal{E}$ is parameterized by points of the unit disk, namely, one can write $\mathcal{H}_{\mathcal{E}} = \mathcal{H}_{\mathcal{E}_\beta}$, where
$$ \mathcal{E}_\beta = \frac{1}{(1 - |\beta|^2)^{1/2}} (\mathcal{E} - \bar{\beta} \mathcal{E}^\#), \quad |\beta| < 1. $$
The coefficient $\frac{1}{(1 - |\beta|^2)^{1/2}}$ is needed for equality of norms in the spaces. In our case, we obtain
$$ \begin{aligned} \mathcal{E}_\beta(z) &= \frac{e^{2\pi} + e^{-2\pi}}{(1 - |\beta|^2)^{1/2}} \left( K_s(2\pi) + \beta K_{s-1}(2\pi) - \bar{\beta} K_{s-1}(2\pi) - |\beta|^2 K_s(2\pi) \right) \\ &= (e^{2\pi} + e^{-2\pi}) (1 - |\beta|^2)^{1/2} K_s(2\pi) = 2 K_s(2\pi). \end{aligned} $$
Thus, the following fact is established.

**Proposition 7.** *The de Branges spaces with structural functions $\mathcal{E}(z) = E(-2\pi, z)$ from formula (7) and $2 K_s(2\pi)$ coincide.*

### §5. Proof of the main theorem

The Riemann xi-function is defined by the formula
$$ \xi(s) = \frac{1}{2} s(s-1) \pi^{-s/2} \Gamma\left(\frac{s}{2}\right) \zeta(s). $$
Set
$$ \varphi(z) = \frac{|\text{Im}\, \alpha|^2}{\xi(\frac{1}{2})} \cdot \frac{\xi(2s - \frac{1}{2})}{(2s - \frac{1}{2} - \alpha)(2s - \frac{1}{2} - \bar{\alpha})}. $$
Then $\varphi(0) = 1$, and the set of zeros of the function $\varphi$ coincides with the set from the formulation of the main theorem. Indeed, the function $\varphi$ vanishes at zero if and only if
$$ 2s - \frac{1}{2} = 2 \cdot \frac{1-iz}{2} - \frac{1}{2} = \frac{1}{2} - iz $$
is a zero of the zeta function from the vertical strip, in which the real part lies in the interval $(0, 1)$, excluding the points $\alpha$ and $\bar{\alpha}$.
The proof will be based on Lemma 5. Let us define
$$ \Phi = \frac{1}{\sqrt{\pi} z} \left( \frac{\mathcal{E} + \mathcal{E}^\#}{2} - \varphi \right). $$

---
**Page 118**

It is required to verify that $\Phi \in \mathcal{H}_{\mathcal{E}}$, or, more precisely, that $V \begin{pmatrix} 0 \\ \gamma \end{pmatrix} = \Phi$ for some real-valued function $\gamma$ such that $\begin{pmatrix} 0 \\ \gamma \end{pmatrix}$ is an element of the canonical system space.
The fact that $\frac{\Phi}{\mathcal{E}} \in H^2$ is a consequence of the following lemma, which allows obtaining the main theorem without researching the concrete element of the canonical system space associated with the zeta function.
The following proposition was communicated to the author by R. Romanov; its proof will be presented in Sec. 6.4.

**Lemma 8.** *The function $\frac{\pi^{-s} \Gamma(s)}{K_s(2\pi)}$ is bounded in the half-plane $\{\text{Re}\, s > \frac{1}{2}\}$.*

We have
$$ \begin{aligned} \frac{\Phi(z)}{\mathcal{E}(z)} &= \frac{1}{\mathcal{E}(z) \cdot \sqrt{\pi} z} \left( \frac{\mathcal{E}(z) + \mathcal{E}^\#(z)}{2} \right. \\ &\quad \left. - \frac{|\text{Im}\, \alpha|^2}{\xi(\frac{1}{2})} \cdot \frac{\frac{1}{2}(2s - \frac{1}{2})(2s - \frac{3}{2}) \pi^{\frac{1}{4} - s} \Gamma(s - \frac{1}{4}) \zeta(2s - \frac{1}{2})}{(2s - \frac{1}{2} - \alpha)(2s - \frac{1}{2} - \bar{\alpha})} \right). \end{aligned} $$
The function in the parentheses vanishes at zero. The function $\frac{\mathcal{E}^\#(z)}{\mathcal{E}(z)}$ is inner; consequently, the term
$$ \frac{1}{\mathcal{E}(z) \cdot \sqrt{\pi} z} \cdot \frac{\mathcal{E}(z) + \mathcal{E}^\#(z)}{2} = \frac{1}{\sqrt{\pi} z} \cdot \frac{1 + \frac{\mathcal{E}^\#(z)}{\mathcal{E}(z)}}{2} $$
will belong to $H^2$ after removing the singularity at zero (for example, by subtracting $\frac{1}{\sqrt{\pi} z}$). It remains to estimate
$$ \begin{aligned} &\frac{|\text{Im}\, \alpha|^2 \cdot \frac{1}{2} (2s - \frac{1}{2}) (2s - \frac{3}{2}) \pi^{\frac{1}{4} - s} \Gamma(s - \frac{1}{4}) \zeta(2s - \frac{1}{2})}{(e^{2\pi} + e^{-2\pi}) (K_s(2\pi) + \beta K_{s-1}(2\pi)) \cdot \sqrt{\pi} z \cdot \xi(\frac{1}{2}) \cdot (2s - \frac{1}{2} - \alpha) (2s - \frac{1}{2} - \bar{\alpha})} \\ &= \frac{|\text{Im}\, \alpha|^2 \pi^{-1/4}}{2(e^{2\pi} + e^{-2\pi})\xi(\frac{1}{2})} \times \frac{(2s - \frac{1}{2}) (2s - \frac{3}{2}) \cdot \zeta(2s - \frac{1}{2})}{z \cdot (2s - \frac{1}{2} - \alpha) (2s - \frac{1}{2} - \bar{\alpha})} \\ &\qquad \times \frac{\pi^{-s} \Gamma(s)}{K_s(2\pi)} \times \frac{\Gamma(s - \frac{1}{4})}{\Gamma(s)} \times \frac{1}{1 + \beta \frac{K_{s-1}(2\pi)}{K_s(2\pi)}}. \end{aligned} $$
The factor containing the zeta function will belong to $H^2$ after removing the singularity at zero. The factor $\frac{\pi^{-s} \Gamma(s)}{K_s(2\pi)}$ is bounded by the lemma.

---
**Page 119**

The ratio $\frac{\Gamma(s - \frac{1}{4})}{\Gamma(s)}$ behaves like $s^{-1/4}$. The last factor is bounded, since $|\beta| < 1$ and the function $\frac{K_{s-1}(2\pi)}{K_s(2\pi)}$ is inner. Thus, the entire expression defines a function from $H^2$.
It is proven that $\frac{\Phi(z)}{\mathcal{E}(z)} \in H^2$. Since $\xi(1 - s) = \xi(s)$, we obtain $\varphi^\# = \varphi$ and $\Phi^\# = \Phi$. This immediately implies the property $\Phi \in \mathcal{H}_{\mathcal{E}}$.
Being an element of the de Branges space, $\Phi$ admits a representation in the form of the integral transformation (4) of some element of the canonical system space. Since $\Phi$ is an odd function, the corresponding element of the canonical system space has the form $\begin{pmatrix} 0 \\ \gamma \end{pmatrix}$. Since $\Phi$ takes real values on the real line, $\gamma$ is a real-valued function.
The theorem is proved. $\square$

### §6. Proofs

This section is devoted to proofs of some facts of the general theory of canonical systems, adapted to the case described by properties (p1)–(p4), as well as Lemma 8.

**1. Proof of Proposition 2.** From the canonical system for any $t$, the well-known important relation is easily obtained
$$ \frac{d}{dt} \left( \overline{A(t, z_1)} B(t, z_2) - \overline{B(t, z_1)} A(t, z_2) \right) \quad (8) $$
$$ = (\bar{z}_1 - z_2) \left( h_+^2(t) \overline{A(t, z_1)} A(t, z_2) + h_-^2(t) \overline{B(t, z_1)} B(t, z_2) \right). $$
In particular,
$$ i \frac{d}{dt} \left( \overline{A(t, z)} B(t, z) - \overline{B(t, z)} A(t, z) \right) = 2 \text{Im}\, z \cdot \left( h_+^2(t) |A(t, z)|^2 + h_-^2(t) |B(t, z)|^2 \right) $$
for $\text{Im}\, z > 0$. The expression on the right-hand side is non-negative. Integrating the left-hand side from $-\infty$ to $a$ and taking into account the assumption that $A(t, z) \to 1$ and $B(t, z) \to 0$ as $t \to -\infty$, we obtain
$$ i \left( \overline{A(a, z)} B(a, z) - \overline{B(a, z)} A(a, z) \right) > 0. $$
Thus, the expression
$$ |\mathcal{E}(z)|^2 - |\mathcal{E}^\#(z)|^2 = |A(a, z) + iB(a, z)|^2 - |A(a, z) - iB(a, z)|^2 $$
$$ = 2i \left( \overline{A(a, z)} B(a, z) - \overline{B(a, z)} A(a, z) \right) $$

---
**Page 120**

is positive when $z$ is in the upper half-plane. This means that the function $\mathcal{E}(z) = E(a, z)$ belongs to the Hermite-Biehler class, as required. The proof for $t < a$ is completely analogous. $\square$

**2. Proof of Proposition 3.** The expression for the values of the function $Vf$ from the de Branges space at point $z$ can be viewed as a scalar product, namely,
$$ (Vf)(z) = (f, k_z), \quad k_z(t) = \frac{1}{\sqrt{\pi}} \begin{pmatrix} \overline{A(t, z)} \\ \overline{B(t, z)} \end{pmatrix}. $$
Using formula (8), we have
$$ (Vk_\lambda)(z) = (k_\lambda, k_z) = \frac{1}{\pi} \int_{-\infty}^a \left( \overline{A(t, \lambda)} A(t, z) h_+^2(t) + \overline{B(t, \lambda)} B(t, z) h_-^2(t) \right) \, dt $$
$$ = \frac{1}{\pi} \frac{1}{\bar{\lambda} - z} \int_{-\infty}^a \left( \frac{d}{dt} \left( \overline{A(t, \lambda)} B(t, z) - \overline{B(t, \lambda)} A(t, z) \right) \right) \, dt $$
$$ = \frac{1}{\pi} \frac{\overline{A(a, \lambda)} B(a, z) - \overline{B(a, \lambda)} A(a, z)}{\bar{\lambda} - z} = \frac{1}{2\pi i} \frac{\overline{\mathcal{E}(\lambda)} \mathcal{E}(z) - \overline{\mathcal{E}^\#(\lambda)} \mathcal{E}^\#(z)}{\bar{\lambda} - z}. $$
It is well known that the functions $Vk_z$ are reproducing kernels of the de Branges space $\mathcal{H}_{\mathcal{E}}$; this means that for $F \in \mathcal{H}_{\mathcal{E}}$ the values of $F$ can be written as scalar products
$$ F(z) = (F, Vk_z)_{\mathcal{H}_{\mathcal{E}}}. \quad (9) $$
Consequently,
$$ (Vk_\lambda, Vk_z)_{\mathcal{H}_{\mathcal{E}}} = (Vk_\lambda)(z) = (k_\lambda, k_z). $$
Thus, the mapping $V$ preserves scalar products, and, consequently, acts as an isometry on the closed linear span of all elements $k_z$. The functions $Vk_z$ form a complete set in $\mathcal{H}_{\mathcal{E}}$, since by formula (9), the function orthogonal to all of them is identically zero. It remains to show that the closed linear span of all elements $k_z$ coincides with the entire canonical system space. If $\nu$ is a real number, the self-adjoint operator $L_\nu$ has a purely point spectrum; this is the key fact of paper [3]. The vectors $k_z$ satisfy the relation $\mathcal{L}k_z = \bar{z}k_z$; in particular, such vectors form a system of all eigenvectors of the operator $L_\nu$ when the number $\nu$ is real and $z$ runs through the spectrum of the operator $L_\nu$. Thus, for any real $\nu$,

---
**Page 121**

elements of the form $k_z$, belonging to the domain of definition of the operator $L_\nu$, form a complete orthogonal system in the canonical system space. The required statement is proved. $\square$

**3. Some auxiliary formulas.** Applying the mapping $V$ to the vector $\mathcal{L}f = \begin{pmatrix} -h_+^{-2} \dot{f}_- \\ h_-^{-2} \dot{f}_+ \end{pmatrix}$, we obtain the function
$$ (V\mathcal{L}f)(z) $$
$$ = \frac{1}{\sqrt{\pi}} \int_{-\infty}^a \left( (-h_+^{-2}(t) \dot{f}_-(t)) A(t, z) h_+^2(t) + (h_-^{-2}(t) \dot{f}_+(t)) B(t, z) h_-^2(t) \right) \, dt $$
$$ = \frac{1}{\sqrt{\pi}} \int_{-\infty}^a \left( -\dot{f}_-(t) A(t, z) + \dot{f}_+(t) B(t, z) \right) \, dt $$
$$ = \frac{1}{\sqrt{\pi}} \left( -f_-(a) A(a, z) + f_+(a) B(a, z) + \int_{-\infty}^a \left( f_-(t) \dot{A}(t, z) - f_+(t) \dot{B}(t, z) \right) \, dt \right). $$
Since
$$ \frac{1}{\sqrt{\pi}} \int_{-\infty}^a \left( f_-(t) \dot{A}(t, z) - f_+(t) \dot{B}(t, z) \right) \, dt $$
$$ = \frac{1}{\sqrt{\pi}} \int_{-\infty}^a \left( f_-(t) z h_-^2(t) B(t, z) + z f_+(t) h_+^2(t) A(t, z) \right) \, dt = z \cdot (Vf)(z), $$
from the formula for $\mathcal{E}$ we finally obtain
$$ (V\mathcal{L}f)(z) = z \cdot (Vf)(z) - \frac{1}{\sqrt{\pi}} \left[ f_-(a) \frac{\mathcal{E}(z) + \mathcal{E}^\#(z)}{2} - f_+(a) \frac{\mathcal{E}(z) - \mathcal{E}^\#(z)}{2i} \right]. $$
In particular, if the function $f$ belongs to the domain of the operator $L_0$, then $f_+(a) = 0$ and
$$ (V L_0 f)(z) = z \cdot (Vf)(z) - \frac{1}{\sqrt{\pi}} f_-(a) \frac{\mathcal{E}(z) + \mathcal{E}^\#(z)}{2}. \quad (10) $$
From the canonical system equation, the relation
$$ \frac{d}{dt} (T_0 f)_- = -h_+^2 f_+ $$
follows.
Consequently,

---
**Page 122**

$$ (T_0 f)_-(a) = - \int_{-\infty}^a h_+^2(t) f_+(t) \, dt = -(f, e), \quad (11) $$
where, as usual, $e = \begin{pmatrix} 1 \\ 0 \end{pmatrix}$. For a complex number $\nu$, since $\mathcal{L} T_\nu f = f = \mathcal{L} T_0 f$, the difference $T_\nu f - T_0 f$ belongs to the kernel of the mapping $\mathcal{L}$. The kernel is one-dimensional and contains the vector $e$. By the definition of the domain of the operator $L_\nu = T_\nu^{-1}$, the relation $(T_\nu f)_+(a) = \nu (T_\nu f)_-(a)$ must hold, whence $T_\nu f = T_0 f + \nu (T_0 f)_-(a) e$. Taking into account relation (11), we obtain formula (3).

Take $F \in \mathcal{H}_{\mathcal{E}}$, and let $F = Vf$, where $f$ is an element of the canonical system space. By formulas (6) and (11), we have
$$ F(0) = \frac{1}{\sqrt{\pi}} (f, e) = -\frac{1}{\sqrt{\pi}} (T_0 f)_-(a). $$
The function $T_0 f$ belongs to the domain of $L_0$, and by formula (10) for $T_0 f$ instead of $f$, we obtain
$$ \begin{aligned} F(z) &= (Vf)(z) = (V L_0 T_0 f)(z) = z \cdot (V T_0 f)(z) - \frac{1}{\sqrt{\pi}} (T_0 f)_-(a) \frac{\mathcal{E}(z) + \mathcal{E}^\#(z)}{2} \\ &= z \cdot (V T_0 V^* F)(z) + F(0) \cdot \frac{\mathcal{E}(z) + \mathcal{E}^\#(z)}{2}, \end{aligned} $$
which is equivalent to formula (5).

**4. Proof of Lemma 8** (communicated to the author by R. Romanov). All zeros of the function $K_s(2\pi)$ (in variable $s$) lie on the imaginary axis, see the statement containing formula (17) in article [6]. Roughly speaking, it will be shown that the function $\frac{K_s(2\pi)}{\pi^{-s} \Gamma(s)}$ tends to $\frac{1}{2}$ as $s$ tends to infinity in the closed domain $\{\text{Re}\, s \geqslant \frac{1}{2}\}$. (In fact, it will be shown somewhat more loosely than the stated assertion, which can be strengthened to the given formulation.) Consequently, for the function from the lemma, it will be proven that it is bounded and boundedly invertible in the region under consideration.
Rewrite the function $K_s$ via the modified Bessel function $I_s$:
$$ K_s(2\pi) = \frac{\pi}{2 \sin \pi s} (I_{-s}(2\pi) - I_s(2\pi)) = \frac{1}{2} \Gamma(s) \Gamma(1-s) (I_{-s}(2\pi) - I_s(2\pi)). $$
Consequently,
$$ \frac{K_s(2\pi)}{\pi^{-s} \Gamma(s)} = \frac{1}{2} \pi^s \Gamma(1-s) (I_{-s}(2\pi) - I_s(2\pi)). $$

---
**Page 123**

We have
$$ I_s(2\pi) = \frac{\pi^s}{\Gamma(1+s)} \left( 1 + O\left(\frac{1}{s}\right) \right) \quad (12) $$
as $s \to \infty$ in the sector where the argument of $s$ belongs to $[-\pi + \delta, \pi - \delta]$ for an arbitrarily small $\delta$; see formula (7.01) in [7]. By this formula for $-s$, we can also write
$$ I_{-s}(2\pi) = \frac{\pi^{-s}}{\Gamma(1-s)} \left( 1 + O\left(\frac{1}{s}\right) \right), $$
and both formulas hold when the argument of $s$ belongs to $[-\pi + \delta, -\delta] \cup [\delta, \pi - \delta]$.
We work with values of $s$ for which $\text{Re}\, s \geqslant \frac{1}{2}$, and, thus, we are interested in values of the argument of $s$ in the interval $(-\frac{\pi}{2}, \frac{\pi}{2})$. First of all, take $s$ with argument from $(-\frac{\pi}{2}, -\delta] \cup [\delta, \frac{\pi}{2})$. For this case, we consider separately the terms containing $I_{-s}(2\pi)$ and $I_s(2\pi)$. For the first of them, we have:
$$ \lim \pi^s \Gamma(1-s) I_{-s}(2\pi) = \lim \pi^s \Gamma(1-s) \frac{\pi^{-s}}{\Gamma(1-s)} = 1. $$
To prove the required convergence, we show that the other term $\pi^s \Gamma(1-s) I_s(2\pi)$ tends to zero. According to formula (12), it behaves like $\pi^{2s} \frac{\Gamma(1-s)}{\Gamma(1+s)}$. Stirling's formula gives the equivalent expression $\frac{(\pi e)^{2s}}{(1-s^2)^s}$. For the logarithm of the modulus of this quantity, we have:
$$ \log \left| \frac{(\pi e)^{2s}}{(1-s^2)^s} \right| = 2 \text{Re}\, s \cdot \log(\pi e) - \text{Re} (s \cdot \log(1-s^2)) $$
$$ = 2 \text{Re}\, s \cdot \log(\pi e) - \text{Re}\, s \cdot \text{Re} \log(1-s^2) + \text{Im}\, s \cdot \text{Im} \log(1-s^2). $$
It is easy to see that $\text{Im} \log(1-s^2)$ has the same sign as $-\text{Im}\, s$. Consequently,
$$ \log \left| \frac{(\pi e)^{2s}}{(1-s^2)^s} \right| \leqslant \text{Re}\, s \cdot (2 \log(\pi e) - \text{Re} \log(1-s^2)), $$
which, as required, tends to $-\infty$ as $s \to \infty$ on the set $\{\text{Re}\, s \geqslant \frac{1}{2}\}$, since $\text{Re} \log(1-s^2) = \log|1-s^2|$ behaves like $2 \log|s|$.
It remains to investigate the neighborhood of zero for the sector in which the argument of $s$ belongs to $[-\delta, \delta]$. It was proved that the function under consideration is bounded in the half-plane $\{\text{Re}\, s \geqslant \frac{1}{2}\}$ excluding the angle where the argument of $s$ is small. From the Phragmén-Lindelöf principle, boundedness in the entire half-plane follows. Since the established properties of the function

---
**Page 124**

it cannot have a non-trivial inner factor. Consequently, it is outer, and since it is boundedly invertible on the boundary, this property is preserved in the entire half-plane, as required.
As an alternative method of proof, one can use estimate (7.17) from [7]; see also Sec. 8.3 there, where this estimate is extended to a generality covering our case. $\square$

### References

[1] Romanov R., *Canonical systems and de Branges spaces*, Lecture Notes in Math. (to appear).
[2] Kac I. S., Krein M. G., *Criterion for the discreteness of the spectrum of a singular string*, Izv. vuzov. Mat. **1958**, No 2, 136–153.
[3] Romanov R., Woracek H., *Canonical systems with discrete spectrum*, J. Funct. Anal. **278** (2020), no. 4, 108318.
[4] Lagarias J. C., *The Schrödinger operator with Morse potential on the right half-line*, Commun. Number Theory Phys. **3** (2009), no. 2, 323–361.
[5] Bateman H., Erdélyi A., *Higher Transcendental Functions*, Vol. 2, McGraw-Hill, New York, 1953. [Russian translation: Nauka, Moscow, 1974].
[6] Pólya G., *Über trigonometrische Integrale mit nur reellen Nullstellen*, J. Reine Angew. Math. **158** (1927), 6–18.
[7] Olver F. W. J., *Asymptotics and Special Functions*, Comp. Sci. Appl. Math., Acad. Press, New York, 1974.

**St. Petersburg Department of the Steklov Mathematical Institute of the Russian Academy of Sciences,**
**nab. r. Fontanki, d. 27,**
**191023 St. Petersburg, Russia**
*E-mail:* `kapustin@pdmi.ras.ru`

**Received June 23, 2020**
