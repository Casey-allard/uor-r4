\documentclass[11pt,reqno]{amsart}

\usepackage{amsmath, amssymb, amsthm}
\usepackage{geometry}
\usepackage{url}
\usepackage{mathrsfs}

% Geometry settings to match the look of the document roughly
\geometry{a4paper, margin=1in}

% Theorem and environment definitions
\newtheorem{theorem}{Theorem}
\newtheorem{proposition}{Proposition}
\newtheorem{lemma}{Lemma}
\theoremstyle{definition}
\newtheorem{definition}{Definition}
\theoremstyle{remark}
\newtheorem{remark}{Remark}

% Custom commands for symbols found in the text
\newcommand{\E}{\mathcal{E}}
\newcommand{\He}{\mathcal{H}_{\mathcal{E}}}
\newcommand{\diff}{\mathrm{d}}
\newcommand{\ii}{\mathrm{i}}
\newcommand{\Lag}{\mathcal{L}}
\newcommand{\Mellin}{\mathcal{M}}

\begin{document}

\title{RIEMANN XI FUNCTION AND MODIFIED BESSEL FUNCTIONS}
\author{V. V. KAPUSTIN}

\address{St. Petersburg Department of the Steklov Mathematical Institute, RAS, Fontanka 27, St. Petersburg 191023, Russia}
\email{kapustin@pdmi.ras.ru}

\thanks{2020 \textit{Mathematics Subject Classification}. Primary 30H45.\\
\textit{Key words and phrases}. Riemann zeta function, de Branges spaces, generalized Fourier transform.\\
\copyright 2025 American Mathematical Society}

\begin{abstract}
In a recent paper the author constructed a canonical system of differential equations with a diagonal Hamiltonian related to the Riemann zeta function. With this system an integral representation in terms of modified Bessel functions is associated for elements of the de Branges space related to the canonical system. Such an integral representation is constructed for the element of the Hilbert space of the canonical system related to the zeta function modified in an appropriate way.
\end{abstract}

\maketitle

% Header Information approximation
\thispagestyle{empty}
\noindent
\footnotesize{
Algebra i analiz \hfill St. Petersburg Math. J. \\
Tom 36 (2024), No 2 \hfill Vol. 36 (2025), No. 2, Pages 203--216\\
\url{https://doi.org/10.1090/spmj/1854}\\
Article electronically published on July 11, 2025
}
\normalsize
\vspace{1cm}

The entire function $\E$, $\E(z) = 2K_s(2\pi)$, where
$$
s = s(z) = \frac{1-iz}{2}
$$
and $K_s$ is the modified Bessel function defined in the standard way, belongs to the Hermite--Biehler class; this means that $|\E(\bar{z})| < |\E(z)|$ for $\text{Im}\, z > 0$. By virtue of this property the function $\E$ is the structure function of a de Branges space $\He$ (the definition will be given below). From the results of the paper [3] one can see that the space $\He$ contains the Riemann xi function $\xi$,
$$
\xi(s) = \frac{1}{2} s(s-1)\pi^{-\frac{s}{2}} \Gamma\left(\frac{s}{2}\right) \zeta(s),
$$
where $\zeta$ is the Riemann zeta function, divided by a polynomial with appropriate zeros of degree three or higher, or modified in a different way to achieve the same decay at infinity. For the canonical system with the diagonal Hamiltonian
\begin{equation} \label{1}
H(t) = \begin{pmatrix} e^{2t} & 0 \\ 0 & e^{-2t} \end{pmatrix}
\end{equation}
on the negative semiaxis, in the paper [3] the structure function for the chain of de Branges spaces was written out explicitly. Then $\He$ is the de Branges space corresponding to the canonical system on the interval $(-\infty, -2\pi)$. The Hilbert space of the canonical system is sent to the de Branges space corresponding to the system by a standard isometric mapping. This naturally leads to the problem of describing the element of the space of the canonical system associated with the modified xi function. This paper is devoted to a solution of the problem. A description of the construction will be presented in \S1, and from it one can see that the theorem stated below gives us such a description.

The Jacobi theta function is defined by the formula
$$
\theta(iz) = \sum_{n=-\infty}^{\infty} e^{-\pi n^2 z}, \quad \text{Re}\, z > 0,
$$
and for it the following relation is well known:
$$
\theta\left(\frac{i}{z}\right) = \sqrt{z} \cdot \theta(iz).
$$
Therefore, the function $\frac{z^{3/4}}{1+z} \cdot \theta(iz)$ is invariant for the substitution $z \leftrightarrow \frac{1}{z}$, and hence it can be written as a function of $z + \frac{1}{z}$. The function obtained can be represented as the Laplace transform of a certain tempered distribution $\varkappa$. This distribution is the derivative of an ordinary function. The following auxiliary assertion will be needed for the formulation of the main result of the paper.

\begin{proposition}
There exists a function $\lambda$ on the positive semiaxis, which belongs to $L^1(0, 2\pi)$ and $L^2(2\pi, +\infty)$, so that for the distribution $\varkappa$ defined by the formula
\begin{equation} \label{2}
\varkappa = 2t\dot{\lambda} + (1-2t)\lambda
\end{equation}
we have
\begin{equation} \label{3}
\int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) d\varkappa(t) = \frac{z^{3/4}}{1+z} \cdot \theta(iz), \quad \text{Re}\, z > 0.
\end{equation}
\end{proposition}

In the relation from the proposition, the Laplace transform of the distribution $\varkappa$ calculated at the point $\frac{1}{2}(z + \frac{1}{z})$ is considered.

\begin{theorem}
We have
$$
\int_{2\pi}^\infty \big( K_s(t) - K_{s-1}(t) \big) \lambda(t) \, dt = \frac{\xi\left(2s - \frac{1}{2}\right) - R(s)}{\left(s-\frac{1}{4}\right)\left(s-\frac{1}{2}\right)\left(s-\frac{3}{4}\right)},
$$
where $R(s)$ is the linear combination of the functions
$$
K_s(2\pi) + K_{s-1}(2\pi), \quad \left(s-\frac{1}{2}\right) \big( K_s(2\pi) - K_{s-1}(2\pi) \big) \quad \text{and}
$$
$$
(s^2 - s) \big( K_s(2\pi) + K_{s-1}(2\pi) \big).
$$
\end{theorem}

According to formula (6) below, after the change of variables $s = \frac{1-iz}{2}$ the image of the function given by the integral on the left-hand side of the formula from the theorem turns out to be an element of the de Branges space $\He$, and the integral representation constructed relates the modified xi function to the corresponding element of the Hilbert space of the canonical system with the Hamiltonian (1) on the semiaxis $(-\infty, -2\pi)$. In this way, earlier the author obtained a proof of the fact that the function on the right-hand side belongs to the space $\He$; a shorter and more straightforward proof of this fact presented in the paper [3] is based on Lemma 8 from [3], which was communicated to the author by R. V. Romanov.

Since the left-hand side of the relation from the theorem contains an entire function, the expression in the numerator on the right-hand side must vanish for $s = \frac{1}{4}, s = \frac{1}{2}$, and $s = \frac{3}{4}$. If this is fulfilled for one of the points $\frac{1}{4}, \frac{3}{4}$, then this is also true for the other, because the expressions in the numerator are invariant for the replacement $s \leftrightarrow 1-s$. This imposes a restriction on the coefficients by the summands of the function $R$. Taking this into account, one more relation is required to find all the coefficients, which is not discussed here.

\section{The construction}

The de Branges space $\He$ with a structure function $\E$ from the Hermite--Biehler class is defined as the set of all entire functions $F$ on the complex plane such that $F/\E$ and $F^\sharp/\E$, where
$$
F^\sharp(z) = \overline{F(\bar{z})},
$$
belong to the Hardy space $H^2$ on the upper halfplane $\{\text{Im}\, z > 0\}$. The norms of the functions $F/\E$ and $F^\sharp/\E$ in $H^2$ coincide and define the norm of the function $F$ in $\He$.

The canonical system of differential equations with a Hamiltonian on an interval $\mathcal{I}$ of the real axis is the equation of the form
$$
J \dot{f}(t) = zH(t)f(t),
$$
where $J = \begin{pmatrix} 0 & -1 \\ 1 & 0 \end{pmatrix}$, $f = \begin{pmatrix} f_+ \\ f_- \end{pmatrix}$, $H$ is the Hamiltonian, which is a nonnegative locally integrable $2 \times 2$ matrix-function. By the structure function of the canonical system we will understand the function $E = A + iB$, where $A, B$ are entire functions of $z$ for each $t \in \mathcal{I}$ such that $\binom{A(t,z)}{B(t,z)}$ is the solution of the system that has the limit $\binom{1}{0}$ at the left endpoint of the interval $\mathcal{I}$ for each $z$. It is well known that then for each $t \in \mathcal{I}$ the function $E(t, z)$ belongs to the Hermite--Biehler class and hence it is the structure function of some de Branges space.

For the canonical system on the negative semiaxis with the Hamiltonian (1), the structure function has the form $E = A + iB$ with
$$
A(t, z) = \sqrt{\frac{-t}{2\pi}} e^{-t} \big( K_s(-t) + K_{s-1}(-t) \big),
$$
$$
B(t, z) = -i \sqrt{\frac{-t}{2\pi}} e^{t} \cdot \big( K_s(-t) - K_{s-1}(-t) \big),
$$
see [3], where, as above, $s = s(z) = \frac{1-iz}{2}$ (here and in what follows this formula should be applied whenever $s$ appears in expressions for functions of the variable $z$). We are interested in the restriction of this canonical system to the interval $(-\infty, -2\pi)$, at the right endpoint of which, that is, for $t = -2\pi$, we obtain the structure function in the form
\begin{multline*}
E(-2\pi, z) = A(-2\pi, z) + iB(-2\pi, z) \\
= e^{2\pi} \big( K_s(2\pi) + K_{s-1}(2\pi) \big) + e^{-2\pi} \big( K_s(2\pi) - K_{s-1}(2\pi) \big) = \frac{1}{\sqrt{1 - |\beta|^2}} \left( \E - \beta \overline{\E^\sharp} \right),
\end{multline*}
where $\E(z) = 2K_s(2\pi)$, and $\beta = \frac{1-e^{-4\pi}}{1+e^{-4\pi}}$, $|\beta| < 1$. The de Branges space corresponding to it coincides (including equality of the norms) with the de Branges space $\He$.

The Hilbert space of the canonical system is (if there are no so-called indivisible intervals like in our case) the weighted space that consists of pairs of functions on the interval $\mathcal{I}$ for which the weight is the Hamiltonian $H$. In the case where the Hamiltonian is diagonal, it is the direct sum of a pair of scalar weighted spaces. In our case of the system with the Hamiltonian (1), the space of the canonical system can be represented as the set of all pairs of functions $\binom{f_+}{f_-}$ on the negative semiaxis written in the form
\begin{equation} \label{4}
f_+(-t) = \sqrt{2t}\, e^t \cdot u_+(t), \quad f_-(-t) = \sqrt{2t}\, e^{-t} \cdot u_-(t), \quad t > 0,
\end{equation}
where the functions $u_\pm$ belong to the unweighted space $L^2(0, +\infty)$, see also [4]. The norm of the pair $\binom{f_+}{f_-}$ in the space of the canonical system equals the norm of the pair $\binom{u_+}{u_-}$ in the direct sum of the pair of spaces $L^2(0, +\infty)$.

The spectral measure of the canonical system with the Hamiltonian (1) on the negative semiaxis is the measure absolutely continuous with respect to the Lebesgue measure on the real axis with the density
$$
w(x) = \cosh \frac{\pi x}{2} = \frac{1}{2} (e^{\frac{\pi x}{2}} + e^{-\frac{\pi x}{2}}).
$$
This means that all the spaces from the chain of de Branges spaces corresponding to the system are isometrically embedded into the weighted space $L^2_w$ in a natural way, and their union is dense in $L^2_w$. It will be convenient to work on the critical line $\{\text{Re}\, s = \frac{1}{2}\}$ directly, considering functions of the variable $s$ without transplanting them to the real axis by the change of variables $s = \frac{1-iz}{2}$ as it is required when we study de Branges spaces. Then the space $L^2_w$ on the real axis isometrically corresponds to the space $L^2_W$ on the critical line with the weight
$$
W(s) = \exp(\pi \, \text{Im}\, s) + \exp(-\pi \, \text{Im}\, s).
$$
There exists a standard isometric isomorphism between the Hilbert space of the canonical system and the space $L^2_w$. We write it in terms of the functions $u_\pm$ and for the variable $s$ on the space $L^2_W$: the operators
\begin{align}
u_+ & \mapsto \frac{1}{2\pi} \int_0^\infty \big( K_s(t) + K_{s-1}(t) \big) u_+(t) \, dt \quad \text{and} \label{5} \\
u_- & \mapsto \frac{1}{2\pi} \int_0^\infty \big( K_s(t) - K_{s-1}(t) \big) u_-(t) \, dt \label{6}
\end{align}
isometrically send the space $L^2(0, +\infty)$ onto the subspaces of the spaces $L^2_W$ that correspond to the even and the odd subspace of the space $L^2_w$, namely, the subspaces of the functions $F \in L^2_W$ for which $F(1-s) = F(s)$ and $F(1-s) = -F(s)$, respectively. These subspaces are mutually orthogonal and their sum is the whole space $L^2_W$.

Clearly, the relation from the theorem represents the modified xi function as the image of the function $\lambda$ restricted to the ray $(2\pi, +\infty)$ under the mapping (6).

\section{Factorization}

In the papers [4, 5] we studied factorizations of the operators (5) and (6) as superpositions of a few standard isometric mappings. For the both operators, the space $L^2(0, +\infty)$ is first transplanted by the Laplace transform to the Hardy space $H^2$ on the right halfplane $\{\text{Re}\, z > 0\}$. Next, the isometric substitution of the Joukowsky function $\frac{1}{2}(z + \frac{1}{z})$ is applied on the space $H^2$ on the right halfplane. Its image is the subspace of $H^2$, which consists of functions that are invariant for the change of variables $z \leftrightarrow \frac{1}{z}$. Then isometric operators of multiplication are applied: by the function $\frac{1}{\sqrt{2}}(1 + \frac{1}{z})$ for the operator (5) and by the function $\frac{1}{\sqrt{2}}(1 - \frac{1}{z})$ for the operator (6). The last mapping in the factorization of the operators (5) and (6) is the Mellin transform acting as an isometry from $H^2(\{\text{Re}\, z > 0\})$ onto the weighted space $L^2_W$ on the line $\{\text{Re}\, s = \frac{1}{2}\}$; being interpreted in this way, it will be denoted by $\Mellin$.

It is not difficult to deduce the factorization for the operator (5) from the relation
$$
K_s(t) + K_{1-s}(t) = \frac{1}{2} \int_0^\infty \exp\left( -\frac{t}{2}\left(x + \frac{1}{x}\right) \right) \left( 1 + \frac{1}{x} \right) x^{s-1} \, dx,
$$
which can be obtained from the well-known formula
$$
K_s(t) = \int_0^\infty \exp(-t \cosh y) \cosh sy \, dy,
$$
see, e.g., [2], formula (7.12.21); one should take into account that $K_{s-1}(t) = K_{1-s}(t)$. For the operator (6) the calculation looks similar; for more details see [4].

The operators
$$
\Pi_+, \Pi_- : L^2(0, +\infty) \to H^2(\{\text{Re}\, z > 0\})
$$
given by the formulas
\begin{align*}
(\Pi_+ u_+)(z) &= \frac{1}{2\sqrt{\pi}} \cdot \left( 1 + \frac{1}{z} \right) \cdot \int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) u_+(t) \, dt, \\
(\Pi_- u_-)(z) &= \frac{1}{2\sqrt{\pi}\, i} \cdot \left( 1 - \frac{1}{z} \right) \cdot \int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) u_-(t) \, dt
\end{align*}
are isometric, and their images are subspaces of the space $H^2$ in the halfplane $\{\text{Re}\, z > 0\}$, which consist, respectively, of functions invariant for the isometric involution $g \leftrightarrow \frac{1}{z} g(\frac{1}{z})$ of the space $H^2(\{\text{Re}\, z > 0\})$ for $\Pi_+$, and of functions with the anti-invariance property $\frac{1}{z} g(\frac{1}{z}) = -g(z)$ for $\Pi_-$. These two subspaces are mutually orthogonal and their direct sum is the whole space $H^2$ on the right halfplane. Therefore, the operator
$$
\Pi: L^2(0, +\infty) \oplus L^2(0, +\infty) \to H^2(\{\text{Re}\, z > 0\}),
$$
$$
\Pi \begin{pmatrix} u_+ \\ u_- \end{pmatrix} = \Pi_+ u_+ + \Pi_- u_-
$$
is an isometric isomorphism.

Thus, the mappings (5) and (6) are written as the superpositions $\Mellin \Pi_+$ and $\Mellin \Pi_-$, respectively. This means that when looking for the element of the Hilbert space of the canonical system that corresponds to a function from $L^2_W$ we have to do the following. In order to obtain the Laplace transforms of the functions $u_+, u_-$ on the positive semiaxis that correspond to an element of the space of the canonical system in the sense of formula (4), to the element of the space $L^2_W$ one should apply the inverse Mellin transform first. This will give us an element of the Hardy space $H^2$ on the right halfplane. Then one should apply the inverse mappings to the other simple mappings listed above in accordance with the formulas of the mappings $\Pi_+$ and $\Pi_-$.

The factorization described in this section explains the meaning of the result of this paper and of the proofs presented below.

The Mellin transform is defined by the formula
$$
(\Mellin g)(s) = \frac{1}{\sqrt{2\pi}} \int_0^{+\infty} g(x) x^{s-1} \, dx.
$$
In the case of the mapping $\Mellin$ acting by the same formula from $H^2(\{\text{Re}\, z > 0\})$ onto $L^2_W$, it should be applied to the trace on the positive semiaxis of a function from $H^2$ on the right halfplane. Now we present a proof of the fact that $\Mellin$ is an isometric isomorphism, which is different from that presented in [5].

A function $g$ belongs to the space $H^2(\{\text{Re}\, z > 0\})$ if and only if it is the Laplace transform of some function $h \in L^2(0, +\infty)$:
$$
g(z) = \frac{1}{\sqrt{2\pi}} \int_0^{+\infty} h(t) e^{-tz} \, dt.
$$
We calculate the Mellin transform of the function $g$ in terms of the function $h$:
\begin{align*}
(\Mellin g)(s) &= \frac{1}{\sqrt{2\pi}} \int_0^{+\infty} g(x) x^{s-1} \, dx = \frac{1}{\sqrt{2\pi}} \int_0^{+\infty} \left( \frac{1}{\sqrt{2\pi}} \int_0^{+\infty} h(t) e^{-tx} \, dt \right) x^{s-1} \, dx \\
&= \frac{1}{2\pi} \int_0^{+\infty} h(t) \cdot \left( \int_0^{+\infty} e^{-tx} x^{s-1} \, dx \right) dt \\
&= \frac{1}{2\pi} \int_0^{+\infty} h(t) \cdot t^{-s} \Gamma(s) \, dt = \frac{\Gamma(s)}{\sqrt{2\pi}} \cdot (\Mellin h)(1-s).
\end{align*}
The function $h$ belongs to $L^2(0, +\infty)$, and the standard Mellin transform isometrically sends this space onto the unweighted $L^2$-space with respect to the Lebesgue measure on the critical line $\{\text{Re}\, s = \frac{1}{2}\}$. The transition to the values $1-s$ is an isometric involution on the last space. If $\text{Re}\, s = \frac{1}{2}$, we have
$$
\left| \frac{\Gamma(s)}{\sqrt{2\pi}} \right|^2 = \frac{\Gamma(s) \cdot \overline{\Gamma(s)}}{2\pi} = \frac{\Gamma(s) \cdot \Gamma(1-s)}{2\pi} = \frac{1}{2\sin(\pi s)}
$$
$$
= \frac{1}{2\sin(\frac{\pi}{2} + i\pi \text{Im}\, s)} = \frac{1}{2\cos(i\pi \text{Im}\, s)} = \frac{1}{W(s)}.
$$
Hence multiplication by $\frac{\Gamma(s)}{\sqrt{2\pi}}$ isometrically sends the unweighted $L^2$-space on the critical line onto the weighted space $L^2_W$. Thus, the Mellin transform $\Mellin$ is represented as the superposition of isometric isomorphisms: the inverse Laplace transform; the Mellin transform $M$ (that is, understood in the usual sense); the reflection $f \mapsto f(1-s)$, and multiplication by $\frac{\Gamma(s)}{\sqrt{2\pi}}$.

\section{The operator of the canonical system}

On the Hilbert space of the canonical system, the operator of the canonical system is given by the formula $f \mapsto H(t)^{-1} J \dot{f}(t)$. For the Hamiltonian (1), in terms of the functions $u_+, u_-$ defined above this operator can be written as
$$
\left( \Lag \begin{pmatrix} u_+ \\ u_- \end{pmatrix} \right)(t) = \begin{pmatrix} 2t \, \dot{u}_-(t) + (1-2t)u_-(t) \\ -2t \, \dot{u}_+(t) - (1+2t)u_+(t) \end{pmatrix},
$$
see [4]. When applying the operator $\Lag$ to an element of the Hilbert space under consideration, one can obtain functions that do not belong to the space and even distributions. Moreover, the operator $\Lag$ will be applied to functions and distributions that are not elements of the space. In particular, in the proposition we have
$$
\Lag \begin{pmatrix} 0 \\ \lambda \end{pmatrix} = \begin{pmatrix} \varkappa \\ 0 \end{pmatrix}.
$$
The following intertwining property is fulfilled:
\begin{equation} \label{7}
\Pi \begin{pmatrix} u_+ \\ u_- \end{pmatrix} = g \implies i \cdot \left( \Pi \Lag \begin{pmatrix} u_+ \\ u_- \end{pmatrix} \right)(z) = 2z g'(z) + g(z).
\end{equation}
Here we present a sketch of the proof. Consider the case where $u_+ \equiv 0$, with which we will work primarily. We have
$$
2\sqrt{\pi} i \cdot g(z) = \left( 1 - \frac{1}{z} \right) \cdot \int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) u_-(t) \, dt
$$
and
\begin{multline*}
2\sqrt{\pi} \cdot \left( \Pi \Lag \begin{pmatrix} 0 \\ u_- \end{pmatrix} \right)(z) = \left( 1 + \frac{1}{z} \right) \cdot \left( 2 \int_0^\infty t \cdot \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) \dot{u}_-(t) \, dt \right. \\
\left. + \int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) (1-2t) u_-(t) \, dt \right) \\
= \left( 1 + \frac{1}{z} \right) \cdot \left( -2 \int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) \left( 1 - \frac{t}{2}\left(z + \frac{1}{z}\right) \right) u_-(t) \, dt \right. \\
\left. + \int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) (1-2t) u_-(t) \, dt \right) \\
= \left( 1 + \frac{1}{z} \right) \cdot \int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) \left[ -2 + t\left(z + \frac{1}{z}\right) + 1 - 2t \right] u_-(t) \, dt \\
= \left( 1 + \frac{1}{z} \right) \cdot \int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) \left[ \frac{t \cdot (z-1)^2}{z} - 1 \right] u_-(t) \, dt.
\end{multline*}
Since
\begin{multline*}
2\sqrt{\pi} i \cdot g'(z) = \left( 1 - \frac{1}{z} \right) \cdot \int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) \left( -\frac{t}{2} \right) \left( 1 - \frac{1}{z^2} \right) u_-(t) \, dt \\
+ \frac{1}{z^2} \cdot \int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) u_-(t) \, dt \\
= \int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) \left[ \left( 1 - \frac{1}{z} \right) \cdot \left( -\frac{t}{2} \right) \left( 1 - \frac{1}{z^2} \right) + \frac{1}{z^2} \right] u_-(t) \, dt \\
= \frac{1}{z^2} \cdot \int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) \left[ -t \cdot \frac{(z-1)^2}{z} \cdot \frac{z+1}{2} + 1 \right] u_-(t) \, dt,
\end{multline*}
we obtain
\begin{multline*}
2\sqrt{\pi} i \cdot (2z g'(z) + g(z)) \\
= \int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) \left[ \frac{-t \cdot (z-1)^2(z+1) + 2z}{z^2} + 1 - \frac{1}{z} \right] u_-(t) \, dt \\
= -\left( 1 + \frac{1}{z} \right) \cdot \int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) \left[ \frac{t \cdot (z-1)^2}{z} - 1 \right] u_-(t) \, dt \\
= -2\sqrt{\pi} \cdot \left( \Pi \Lag \begin{pmatrix} 0 \\ u_- \end{pmatrix} \right)(z),
\end{multline*}
which is equivalent to the required relation.
For the case of $u_- \equiv 0$ the proof is similar.

For analytic functions $g$ in the right halfplane $\{z : \text{Re}\, z > 0\}$, define
$$
(Sg)(z) = -zg'(z).
$$
The intertwining property (7) can be rewritten as
\begin{equation} \label{8}
i \Pi \Lag = (I - 2S) \Pi.
\end{equation}
Also we have yet another intertwining property
\begin{equation} \label{9}
\begin{split}
(\Mellin Sg)(s) &= -\frac{1}{\sqrt{2\pi}} \int_0^\infty xg'(x) x^{s-1} \, dx = -\frac{1}{\sqrt{2\pi}} \int_0^\infty g'(x) x^s \, dx \\
&= \frac{1}{\sqrt{2\pi}} \int_0^\infty g(x) s x^{s-1} \, dx = s \cdot (\Mellin g)(s),
\end{split}
\end{equation}
which must be understood in the sense of generalized functions.

\section{Proof of the proposition}

Let $\lambda$ be the function from the proposition, which we are looking for. For $g = \Pi_- \lambda = \Pi \binom{0}{\lambda}$ from relation (7) we have
$$
2z g'(z) + g(z) = i \cdot \left( \Pi \Lag \begin{pmatrix} 0 \\ \lambda \end{pmatrix} \right)(z) = i \cdot \left( \Pi \begin{pmatrix} \varkappa \\ 0 \end{pmatrix} \right)(z) = i \cdot (\Pi_+ \varkappa)(z),
$$
and we obtain the equation
$$
2z g'(z) + g(z) = -z^{-1/4} \cdot \theta(iz);
$$
moreover, $g(1) = 0$ because $g$ belongs to the image of the mapping $\Pi_-$.

In accordance with the standard approach to solving differential equations of this form via the solution of the homogeneous equation, write
$$
g(z) = -\frac{q(z)}{2\sqrt{z}},
$$
and we obtain
$$
2z g'(z) + g(z) = -z \cdot \frac{q'(z)\sqrt{z} - \frac{1}{2\sqrt{z}}q(z)}{z} - \frac{q(z)}{2\sqrt{z}} = -\sqrt{z} \cdot q'(z).
$$
Thus, we have the equation $-\sqrt{z} \cdot q'(z) = -z^{-1/4} \cdot \theta(iz)$, or
$$
q'(z) = \frac{\theta(iz)}{z^{3/4}},
$$
whence $q(z) = \int \frac{\theta(i\nu)}{\nu^{3/4}} \, d\nu + \text{const}$. We are interested in the solution $q$, for which $q(1) = 0$, that is, $q(z) = \int_1^z \frac{\theta(i\nu)}{\nu^{3/4}} \, d\nu$. For the proof of the proposition one should show that the corresponding function $g$ can be written in the form $g = \Pi_- \lambda$, where $\lambda$ is a function having the required properties.

\begin{lemma}
There exists a function $q_* \in H^2 = H^2(\{\text{Re}\, z > 0\})$ such that $q'_*(z) = \frac{\theta(iz)-1}{z^{3/4}}$.
\end{lemma}

\begin{proof}
For positive integers $n$ consider the equations $q'_n(z) = \frac{\exp(-\pi n^2 z)}{z^{3/4}}$, in which the right-hand sides are taken in accordance with the summands from the definition of the function $\theta$. (For negative $n$ the summands coincide with those corresponding to $-n$; the zeroth summand is out of consideration.) Take the solutions $q_n$ that exponentially decrease at infinity; they have the form
\begin{align*}
q_n(z) &= -\int_z^\infty \frac{\exp(-\pi n^2 \nu)}{\nu^{3/4}} \, d\nu = -\frac{\pi^{-1/4}}{\sqrt{n}} \int_z^\infty \frac{\exp(-\pi n^2 \nu)}{(\pi n^2 \nu)^{3/4}} \pi n^2 \, d\nu \\
&= -\frac{\pi^{-1/4}}{\sqrt{n}} \int_{\pi n^2 z}^\infty \frac{e^{-\nu}}{\nu^{3/4}} \, d\nu = -\frac{\pi^{-1/4}}{\sqrt{n}} \Gamma(1/4, \pi n^2 z),
\end{align*}
where $\Gamma(s, z) = \int_z^\infty \nu^{s-1} e^{-\nu} \, d\nu$ is the incomplete gamma function. The function $\Gamma(1/4, z)$ behaves like $z^{-3/4}$ as $z \to \infty$. Hence it belongs to the space $H^2$ on the halfplane $\{\text{Re}\, z > 0\}$, and thus all the functions $q_n$, $n > 0$, belong to $H^2$. For their norms we have
$$
\|q_n\|^2 = \int_{-\infty}^{+\infty} |q_n(iy)|^2 \, dy = \frac{1}{\sqrt{\pi} n} \int_{-\infty}^{+\infty} |\Gamma(1/4, i\pi n^2 y)|^2 \, dy = \frac{1}{\pi^{3/2} n^3} \cdot \|\Gamma(1/4, z)\|^2,
$$
whence it is clear that
$$
q_* = 2 \sum_{n>0} q_n \in H^2.
$$
By construction, we have
$$
q'_*(z) = 2 \sum_{n>0} q'_n(z) = \sum_{n \neq 0} \frac{e^{-\pi n^2 z}}{z^{3/4}} = \frac{\theta(iz) - 1}{z^{3/4}}.
$$
The lemma is proved.
\end{proof}

Clearly, the desired function $q$ has the form
$$
q(z) = q_*(z) + 4z^{1/4} + 2c,
$$
where $q_*$ is the function from the lemma and $c$ is the constant chosen so that $q(1) = 0$.
For $g(z) = -\frac{q(z)}{2\sqrt{z}}$ we obtain the expression
$$
g(z) = -\frac{q_*(z)}{2\sqrt{z}} - \frac{2}{z^{1/4}} - \frac{c}{\sqrt{z}}.
$$
From the formula $q(z) = \int_1^z \frac{\theta(i\nu)}{\nu^{3/4}} \, d\nu$ we obtain
$$
g(z) = -\frac{1}{2\sqrt{z}} \int_1^z \frac{\theta(i\nu)}{\nu^{3/4}} \, d\nu;
$$
moreover,
$$
\frac{1}{z} g\left(\frac{1}{z}\right) = -\frac{1}{2\frac{1}{\sqrt{z}}} \int_1^{1/z} \frac{\theta(i\nu)}{\nu^{3/4}} \, d\nu = -\frac{1}{2/\sqrt{z}} \int_1^z \frac{\theta(\frac{i}{\nu})}{\nu^{-3/4}} \frac{-d\nu}{\nu^2} = \frac{1}{2\sqrt{z}} \int_1^z \frac{\sqrt{\nu}\theta(i\nu)}{\nu^{5/4}} \, d\nu = -g(z).
$$
From the formula for $g$ we obtain an expression for the function $(1 - \frac{1}{z})^{-1} \cdot g(z)$, which is invariant under the substitution $z \leftrightarrow \frac{1}{z}$:
\begin{align*}
\left( 1 - \frac{1}{z} \right)^{-1} \cdot g(z) &= -\frac{q_*(z) \cdot \sqrt{z}}{2(z-1)} - \frac{2z^{3/4}}{z-1} - \frac{c\sqrt{z}}{z-1} \\
&= -\frac{q_*(z)\sqrt{z} - q_*(1)}{2(z-1)} - 2\frac{z^{3/4}-1}{z-1} - c\frac{\sqrt{z}-1}{z-1}.
\end{align*}
This function must be represented as the image of the function $\lambda$ for the mapping
$$
u \mapsto \left( 1 - \frac{1}{z} \right)^{-1} \cdot (\Pi_- u) = \int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) u(t) \, dt.
$$
Since $q_* \in H^2$, the first summand is also a function in $H^2$. For the work with the second summand observe that the differences
$$
\frac{z^{3/4}-1}{z-1} - \frac{z^{3/4}-z^{1/4}}{z-1} = \frac{z^{1/4}-1}{z-1}, \quad \frac{z^{3/4}+z^{1/4}}{z+1} - \left( z + \frac{1}{z} \right)^{-1/4}
$$
are functions in $H^2$; finally, the difference
$$
\frac{\sqrt{z}-1}{z-1} - \frac{\sqrt{z}}{z+1} = -\frac{(\sqrt{z}-1)^2}{z^2-1}
$$
belongs to $H^2$. Thus, the function $(1 - \frac{1}{z})^{-1} \cdot g(z)$ is a linear combination of the functions $(z+\frac{1}{z})^{-1/4}$, $\frac{\sqrt{z}}{1+z}$, and some function from $H^2$, which by the invariance property under the substitution $z \leftrightarrow \frac{1}{z}$ for all other functions also turns out to be invariant.
We have
$$
\int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) t^{-3/4} \, dt = \left( z + \frac{1}{z} \right)^{-1/4},
$$
$$
\int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) t^{-1/2} e^{-t} \, dt = \frac{\sqrt{z}}{z+1}.
$$
The operator $u \mapsto (1-\frac{1}{z})^{-1}(\Pi_- u)$ is the superposition of the Laplace transform and the substitution of the Joukowsky function $\frac{1}{2}(z+\frac{1}{z})$. It sends the space $L^2$ on the semiaxis onto the subspace of all functions from $H^2$ that are invariant under the substitution $z \leftrightarrow \frac{1}{z}$. Hence the function from $H^2$ under consideration is the image of some function from $L^2$ on the semiaxis.
Thus, the desired function $\lambda$ is written as a linear combination of the functions $t^{-3/4}$, $t^{-1/2}e^{-t}$, and some function from $L^2(0, +\infty)$. Each of these functions belongs to $L^1(0, 2\pi)$ and $L^2(2\pi, +\infty)$, and hence this holds true for the function $\lambda$. \qed

\section{The Riemann xi function and the Jacobi theta function}

The classical representation of the function $\xi$ via the Jacobi theta function looks like this:
$$
\int_0^\infty (\theta(ix) - 1) x^{\frac{s}{2}-1} \, dx = 2\pi^{-\frac{s}{2}} \Gamma\left(\frac{s}{2}\right) \zeta(s) = \frac{4\xi(s)}{s(s-1)}, \quad \text{Re}\, s > 1.
$$
The integral $\int_1^\infty (\theta(ix)-1)x^{\frac{s}{2}-1} \, dx$ converges for all complex $s$. By the change of variables $\frac{1}{x} \leftrightarrow x$ we also obtain the convergence for all $s$ for the integral
\begin{align*}
\int_0^1 \left( \theta\left(\frac{i}{x}\right) - 1 \right) \left( \frac{1}{x} \right)^{\frac{s}{2}-1} \frac{dx}{x^2} &= \int_0^1 (\sqrt{x}\theta(ix) - 1) x^{-\frac{s}{2}-1} \, dx \\
&= \int_0^1 \left( \theta(ix) - \frac{1}{\sqrt{x}} \right) x^{-\frac{s+1}{2}} \, dx,
\end{align*}
where we have used the classical formula $\theta(\frac{i}{x}) = \sqrt{x} \cdot \theta(ix)$. Replacing $s$ with $1-s$, we find that also for all $s$ the integral $\int_0^1 (\theta(ix) - \frac{1}{\sqrt{x}}) x^{\frac{s}{2}-1} dx$ converges. In order to find the sum of the two integrals mentioned above, at first suppose that $\text{Re}\, s > 1$; then
\begin{multline*}
\int_1^\infty (\theta(ix) - 1) x^{\frac{s}{2}-1} \, dx + \int_0^1 \left( \theta(ix) - \frac{1}{\sqrt{x}} \right) x^{\frac{s}{2}-1} \, dx \\
= \int_0^\infty (\theta(ix) - 1) x^{\frac{s}{2}-1} \, dx + \int_0^1 \left( 1 - \frac{1}{\sqrt{x}} \right) x^{\frac{s}{2}-1} \, dx = \frac{4\xi(s)}{s(s-1)} + \frac{2}{s} - \frac{2}{s-1};
\end{multline*}
by the analyticity this formula holds for all $s$. Now for $\text{Re}\, s \in (0, 1)$ write
\begin{align*}
\int_0^\infty & \{ \theta(ix) - (1 + x^{-\frac{1}{2}}) \} x^{\frac{s}{2}-1} \, dx \\
&= \int_0^1 (\theta(ix) - x^{-\frac{1}{2}}) x^{\frac{s}{2}-1} \, dx - \int_0^1 x^{\frac{s}{2}-1} \, dx + \int_1^\infty (\theta(ix) - 1) x^{\frac{s}{2}-1} \, dx - \int_1^\infty x^{\frac{s-3}{2}} \, dx \\
&= \int_0^1 (\theta(ix) - x^{-\frac{1}{2}}) x^{\frac{s}{2}-1} \, dx - \frac{2}{s} + \int_1^\infty (\theta(ix) - 1) x^{\frac{s}{2}-1} \, dx + \frac{2}{s-1} = \frac{4\xi(s)}{s(s-1)}.
\end{align*}
Rewriting this formula for $2s - \frac{1}{2}$ in place of $s$ we obtain
\begin{equation} \label{10}
\int_0^\infty \{ \theta(ix) x^{-\frac{1}{4}} - (x^{-\frac{1}{4}} + x^{-\frac{3}{4}}) \} x^{s-1} \, dx = \frac{\xi\left(2s - \frac{1}{2}\right)}{\left(s-\frac{1}{4}\right)\left(s-\frac{3}{4}\right)}, \quad \frac{1}{4} < \text{Re}\, s < \frac{3}{4}.
\end{equation}
Observe that the left-hand side is the Mellin transform of a function that is invariant under the involution $g \leftrightarrow \frac{1}{z} g(\frac{1}{z})$.

\section{Proof of the theorem}

We must prove that
$$
\left(s-\frac{1}{4}\right)\left(s-\frac{1}{2}\right)\left(s-\frac{3}{4}\right) \cdot \int_{2\pi}^\infty (K_s(t) - K_{s-1}(t)) \lambda(t) \, dt = \xi\left(2s - \frac{1}{2}\right) - R(s),
$$
where $R(s)$ is a function of a special form from the theorem. Applying the factorization from \S2, rewrite the left-hand side as
\begin{align*}
\left(s-\frac{1}{4}\right)\left(s-\frac{1}{2}\right)\left(s-\frac{3}{4}\right) &\cdot \Mellin \Pi_- (\chi_{(2\pi, \infty)} \cdot \lambda) \\
&= \left(s-\frac{1}{4}\right)\left(s-\frac{1}{2}\right)\left(s-\frac{3}{4}\right) \cdot \Mellin \Pi_- (\lambda - \chi_{(0, 2\pi)} \cdot \lambda),
\end{align*}
where $\chi_e$ stands for the indicator of a set $e$.
We have
\begin{equation} \label{11}
\left(s-\frac{1}{4}\right)\left(s-\frac{1}{2}\right)\left(s-\frac{3}{4}\right) \cdot \Mellin \Pi_- \lambda = \Mellin \left( \left(S-\frac{1}{4}I\right)\left(S-\frac{3}{4}I\right)\left(S-\frac{1}{2}I\right) \Pi_- \lambda \right)
\end{equation}
by the intertwining property (9). Since
\begin{gather*}
\left( \left(S - \frac{1}{2}I\right) \Pi_- \lambda \right)(z) = (\Pi_+ \varkappa)(z) = z^{-1/4}\theta(iz), \\
\left( S - \frac{1}{4}I \right) \left( S - \frac{3}{4}I \right) (z^{-1/4} + z^{-3/4}) = 0,
\end{gather*}
we continue the calculation (11), using also relation (10):
\begin{align*}
&= \Mellin \left( \left( S - \frac{1}{4}I \right) \left( S - \frac{3}{4}I \right) (z^{-1/4}\theta(iz) - (z^{-1/4} + z^{-3/4})) \right) \\
&= \left( s - \frac{1}{4} \right) \left( s - \frac{3}{4} \right) \cdot \Mellin (z^{-1/4}\theta(iz) - (z^{-1/4} + z^{-3/4})) = \xi\left( 2s - \frac{1}{2} \right).
\end{align*}
It remains to verify the formula for
\begin{equation} \label{12}
R(s) = \left(s-\frac{1}{4}\right)\left(s-\frac{1}{2}\right)\left(s-\frac{3}{4}\right) \cdot (\Mellin \Pi_- (\chi_{(0, 2\pi)} \cdot \lambda))(s),
\end{equation}
namely, that this expression can be written as a linear combination of the functions from the formulation of the theorem.

Define the operator $\Omega$ from the relation
$$
\left( \Lag^2 + \frac{1}{4}I \right) \begin{pmatrix} 0 \\ \varphi \end{pmatrix} = \begin{pmatrix} 0 \\ \Omega \varphi \end{pmatrix},
$$
that is,
$$
(\Omega \varphi)(t) = -2t \frac{d}{dt} \big( 2t \dot{\varphi}(t) + (1-2t)\varphi(t) \big) - (1+2t) \big( 2t \dot{\varphi}(t) + (1-2t)\varphi(t) \big) + \frac{1}{4}\varphi(t).
$$

\begin{lemma}
On the interval $(0, 2\pi)$ the function $\lambda$ is a linear combination of solutions of the differential equations
\begin{equation} \label{13}
2t \dot{\varphi}(t) + (1-2t) \varphi(t) = 0
\end{equation}
and $\Omega \varphi = 0$.
\end{lemma}

We will return to the lemma in \S7; now we complete the proof of the theorem based on the lemma.

First, let $\varphi$ be a solution of the differential equation (13) on $(0, 2\pi)$. Continuing the function $\varphi$ by zero to $(2\pi, \infty)$, we obtain a function for which $2t \dot{\varphi}(t) + (1-2t) \varphi(t)$ is a scalar multiple of the Dirac measure $\delta_{2\pi}$ at the point $2\pi$ (depending on the left limit of the function $\varphi$ at the point $2\pi$). Therefore,
$$
\left(s-\frac{1}{2}\right) \cdot \Mellin \Pi_- \varphi = \Mellin \left(S-\frac{1}{2}I\right) \Pi_- \varphi = \Mellin \Pi_+ (2t \dot{\varphi}(t) + (1-2t) \varphi(t))
$$
is a scalar multiple of the function
$$
(\Mellin \Pi_+ \delta_{2\pi})(s) = \frac{1}{2\pi} \big( K_s(2\pi) + K_{s-1}(2\pi) \big),
$$
and then $(s-\frac{1}{4})(s-\frac{1}{2})(s-\frac{3}{4}) \cdot (\Mellin \Pi_- \varphi)(s)$ is a scalar multiple of the function
\begin{equation} \label{14}
\left(s-\frac{1}{4}\right)\left(s-\frac{3}{4}\right) \cdot \big( K_s(2\pi) + K_{s-1}(2\pi) \big).
\end{equation}
Thus, the contribution of the summand we have considered above into the function from (12) has the desired form.

Now let $\varphi$ be a solution of the differential equation $\Omega \varphi = 0$ on $(0, 2\pi)$, and also continuing $\varphi$ by zero to $(2\pi, \infty)$, we obtain a function for which $\Omega \varphi$ is a linear combination of the Dirac measure $\delta_{2\pi}$ and its derivative. Then
$$
\left(s-\frac{1}{4}\right)\left(s-\frac{3}{4}\right) (\Mellin \Pi_- \varphi)(s) = \left( \Mellin \left( S - \frac{1}{4}I \right) \left( S - \frac{3}{4}I \right) \Pi_- \varphi \right)(s) = (\Mellin \Pi_- \Omega \varphi)(s)
$$
is a linear combination of the functions $(\Mellin \Pi_- \delta_{2\pi})(s) = \frac{1}{2\pi}(K_s(2\pi) - K_{s-1}(2\pi))$ and
$$
(\Mellin \Pi_- \dot{\delta}_{2\pi})(s) = \frac{1}{2\pi} \langle K_s(2\pi) - K_{s-1}(2\pi), \dot{\delta}_{2\pi} \rangle = \frac{1}{2\pi} \big( -\dot{K}_s(2\pi) + \dot{K}_{s-1}(2\pi) \big).
$$
From formulas (7.11.25--26) of the book [2] one can easily deduce the relation
$$
-\dot{K}_s(t) = K_{s-1}(t) + \frac{s}{t} K_s(t).
$$
The same formula for $1-s$ in place of $s$ has the form
$$
-\dot{K}_{s-1}(t) = K_s(t) + \frac{1-s}{t} K_{s-1}(t).
$$
Thus,
\begin{align}
-\dot{K}_s(t) + \dot{K}_{s-1}(t) &= K_{s-1}(t) + \frac{s}{t} K_s(t) - K_s(t) - \frac{1-s}{t} K_{s-1}(t) \label{15} \\
&= \frac{2s-1}{2t} \cdot (K_s(t) + K_{s-1}(t)) + \left( \frac{1}{2t} - 1 \right) (K_s(t) - K_{s-1}(t)), \notag
\end{align}
which for $t = 2\pi$ gives us a real linear combination of the functions $(s-\frac{1}{2}) \cdot (K_s(2\pi) + K_{s-1}(2\pi))$ and $K_s(2\pi) - K_{s-1}(2\pi)$. Hence $(s-\frac{1}{4})(s-\frac{1}{2})(s-\frac{3}{4}) \cdot (\Mellin \Pi_- \varphi)(s)$ is a linear combination of the functions
$$
\left(s-\frac{1}{2}\right)^2 \cdot \big( K_s(2\pi) + K_{s-1}(2\pi) \big), \quad \left(s-\frac{1}{2}\right) \cdot \big( K_s(2\pi) - K_{s-1}(2\pi) \big).
$$
Therefore, $R(s)$ is a linear combination of these functions and the function $K_s(2\pi) + K_{s-1}(2\pi)$. Clearly, the formulas obtained reduce to the form as in the theorem.
To conclude with the proof of the theorem, it remains to prove Lemma 2.

\section{Proof of Lemma 2}

We have
$$
\begin{pmatrix} 2t \dot{\varphi}(t) + (1-2t) \varphi(t) \\ 0 \end{pmatrix} = \Lag \begin{pmatrix} 0 \\ \varphi \end{pmatrix}, \quad \begin{pmatrix} 0 \\ \Omega \varphi \end{pmatrix} = \left( \Lag^2 + \frac{1}{4}I \right) \begin{pmatrix} 0 \\ \varphi \end{pmatrix},
$$
whence one can see that all linear combinations discussed in the lemma form a three-dimensional space of solutions of the equation
$$
\left( \Lag^2 + \frac{1}{4}I \right) \Lag \begin{pmatrix} 0 \\ \varphi \end{pmatrix} \equiv 0
$$
on $(0, 2\pi)$. Hence for the proof of the lemma we need to verify the last relation for $\varphi = \lambda$.
Since $\Lag \binom{0}{\lambda} = \binom{\varkappa}{0}$, we have to show that $(\Lag^2 + \frac{1}{4}I)\binom{\varkappa}{0} \equiv 0$ on $[0, 2\pi)$.
Let us find $\Pi(\Lag^2 + \frac{1}{4}I)\binom{\varkappa}{0}$. By the intertwining relation (8), we obtain
$$
\Pi \left( \Lag^2 + \frac{1}{4}I \right) = \left( -(I-2S)^2 + \frac{1}{4}I \right) \Pi = -4 \cdot \left( S^2 - S + \frac{3}{16}I \right) \Pi.
$$
We have $(\Pi \binom{\varkappa}{0})(z) = z^{-1/4} \cdot \theta(iz)$. Write the further calculations in terms of the functions $\psi$ from Riemann's paper [1],
$$
\psi(z) = \sum_{n=1}^\infty \exp(-\pi n^2 z),
$$
for which we have
$$
\theta(iz) = 1 + 2\psi(z).
$$
We obtain
\begin{gather*}
S(z^{-1/4} \theta(iz)) = -z(z^{-1/4} (2\psi + 1))' = -2z^{3/4}\psi' + \frac{1}{4}z^{-1/4}(2\psi + 1), \\
S^2(z^{-1/4} \theta(iz)) = -z \left( -2z^{3/4}\psi' + \frac{1}{4}z^{-1/4}(2\psi + 1) \right)' \\
= 2z^{7/4}\psi'' + z^{3/4}\psi' + \frac{1}{16}z^{-1/4}(2\psi + 1)
\end{gather*}
and
$$
\left( S^2 - S + \frac{3}{16}I \right) (z^{-1/4} \theta(iz)) = z^{-1/4} \cdot (2z^2\psi'' + 3z\psi').
$$
Thus,
\begin{equation} \label{16}
\Pi \left( \Lag^2 + \frac{1}{4}I \right) \begin{pmatrix} \varkappa \\ 0 \end{pmatrix} = -4 \left( S^2 - S + \frac{3}{16}I \right) (z^{-1/4} \theta(iz)) = -4z^{-1/4} (2z^2\psi'' + 3z\psi').
\end{equation}
The function obtained belongs to the image of the mapping $\Pi_+$. This means that there exists a distribution $\rho$ supported on the nonnegative semiaxis such that
$$
(\Pi_+ \rho)(z) = z^{-1/4} \cdot (2z^2\psi''(z) + 3z\psi'(z)),
$$
that is,
$$
\frac{1}{2\sqrt{\pi}} \cdot \int_0^\infty \exp\left( -\frac{t}{2}\left(z + \frac{1}{z}\right) \right) d\rho(t) = \frac{z^{3/4}}{1+z} \cdot (2z^2\psi''(z) + 3z\psi'(z)).
$$
Denote the function on the right-hand side by $\varphi$. From the formula for the function $\psi$ it is not difficult to see that it, and hence also $\varphi$, is the Laplace transform of some tempered distribution supported on the closed ray $[\pi, +\infty)$. By construction, the function $\varphi$ is invariant under the substitution $z \leftrightarrow \frac{1}{z}$. To complete the proof of Lemma 2, we will show that these two properties imply that the support of the distribution $\rho$ is a subset of the ray $[2\pi, +\infty)$. A careful verification of the assertions used in the proof below requires technical work by standard tools, and we do not present detailed calculations here. Notice that for the proofs of the assertions needed for obtaining results of the paper, we need not consider the general case of arbitrary tempered distributions, it suffices to restrict the arguments to lower derivatives of functions with power growth.

Laplace transforms of tempered distributions supported on the nonnegative semiaxis are analytic functions of tempered growth on the right halfplane $\{\text{Re}\, z > 0\}$. Denote the class of all such functions by $\mathfrak{G}$. From the properties of the function $\varphi$ we have $e^{\pi z} \cdot \varphi \in \mathfrak{G}$. The class $\mathfrak{G}$ is invariant under the substitution $z \leftrightarrow \frac{1}{z}$, and for the function $\varphi$ we have $\varphi(\frac{1}{z}) = \varphi(z)$. Hence $\exp(\frac{\pi}{z}) \cdot \varphi \in \mathfrak{G}$. Next, we obtain $e^{\pi z} \cdot \exp(\frac{\pi}{z}) \cdot \varphi \in \mathfrak{G}$. Indeed, the fact that a function belongs to the class $\mathfrak{G}$ can be verified separately on bounded sets and at infinity. For the both cases the property is fulfilled with one of the multipliers, while the other does not affect the picture. Thus, the function $\exp(\pi(z + \frac{1}{z})) \cdot \varphi$ belongs to $\mathfrak{G}$; obviously, it is invariant under the substitution $z \leftrightarrow \frac{1}{z}$ because the function $\varphi$ has this property. Therefore, it can be obtained by substitution of the Joukowsky function $\frac{1}{2}(z + \frac{1}{z})$ from a certain function for which it is not difficult to check that it belongs to the class $\mathfrak{G}$ as well, and hence it is the Laplace transform of a tempered distribution on the nonnegative semiaxis. If we shift this distribution by $2\pi$ to the right, we will get the distribution $\rho$ up to a multiplicative constant. Multiplication of the analytic function under investigation by $\exp(-\pi(z + \frac{1}{z}))$ corresponds to the shift. The claim follows immediately.
The lemma is proved, and thus the proof of the theorem is complete. \qed

\section{Concluding remarks}

To conclude with, we discuss the integral representation for the xi function from Riemann's paper [1] (in which a slightly different definition of the function $\xi$ was used):
\begin{equation} \label{17}
\xi(s) = 4 \int_1^\infty \frac{d[x^{3/2}\psi'(x)]}{dx} x^{-1/4} \cosh\left[ \frac{1}{2}\left(s - \frac{1}{2}\right) \log x \right] \, dx.
\end{equation}
Substitute $2s - \frac{1}{2}$ into it for $s$ and apply the formula
$$
\cosh\left[ \left(s - \frac{1}{2}\right) \log x \right] = \frac{1}{2} (x^{s-1/2} + x^{1/2-s});
$$
in one of the summands, make the change of variables $x \leftrightarrow \frac{1}{x}$. We obtain an integral over the whole positive semiaxis, which represents the Mellin transform of some function expressed via $\psi$.

On the other hand, since $(S^2 - S + \frac{3}{16}I)(z^{-1/4} + z^{-3/4}) = 0$, from the definition of the operator $S$ we see that
$$
\left( S^2 - S + \frac{3}{16}I \right) (z^{-1/4} \theta(iz) - (z^{-1/4} + z^{-3/4})) = (2z^{7/4}\psi'' + 3z^{3/4}\psi').
$$
Apply the Mellin transform to the both sides of this identity. From the intertwining relation (9) it follows that after the Mellin transform the action of the operator $S^2 - S + \frac{3}{16}I$ turns into multiplication by $s^2 - s + \frac{3}{16} = (s - \frac{1}{4})(s - \frac{3}{4})$. Now formula (10) yields the relation
$$
\xi\left(2s - \frac{1}{2}\right) = \int_0^\infty (2x^{7/4}\psi''(x) + 3x^{3/4}\psi'(x)) x^{s-1} \, dx,
$$
and it is not difficult to check that it is equivalent to Riemann's relation (17).

\bibliographystyle{amsplain}
\begin{thebibliography}{5}

\bibitem{1} B. Riemann, \textit{Über die Anzahl der Primzahlen unter einer gegebenen Grösse}, Monatsberichte Berliner Akad., November 1859, 671--680.

\bibitem{2} A. Erdelyi, W. Magnus, F. Oberhettinger, and F. G. Tricomi, \textit{Higher transcendental functions}. Vol. II, Based, in part, on notes left by Harry Bateman, McGraw-Hill Book Co., Inc., New York, 1953. MR58756

\bibitem{3} V. Kapustin, \textit{The set of zeros of the Riemann zeta function as the point spectrum of an operator}, Algebra i Analiz \textbf{33} (2021), 107--124; English transl., St. Petersburg Math. J. \textbf{33} (2022), no. 4, 661--673. MR4471258

\bibitem{4} \underline{\phantom{V. Kapustin}}, \textit{Five Hilbert space models related to the Riemann zeta functions}, Zap. Nauchn. Sem. S.-Peterburg. Otdel. Mat. Inst. Steklov. (POMI) \textbf{503} (2021), 84--96; English transl., J. Math. Sci. (N.Y.) \textbf{268} (2022), no. 6, 791--799. MR4337264

\bibitem{5} \underline{\phantom{V. Kapustin}}, \textit{The Mellin transform, de Branges spaces, and Bessel functions}, Zap. Nauchn. Sem. POMI \textbf{512} (2022), 88--94. (Russian).

\end{thebibliography}

\end{document}
