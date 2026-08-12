
# qimc_simulations.py
from qutip import *
import numpy as np
import matplotlib.pyplot as plt

# Define system Hilbert space
H = sigmax() + sigmaz()

# Initial state
psi0 = basis(2, 0)
rho0 = ket2dm(psi0)

# Decoherence operators
L1 = np.sqrt(0.3) * sigmaz()
L2 = np.sqrt(0.3) * sigmax()
L3 = np.sqrt(0.3) * sigmay()

tlist = np.linspace(0, 10, 200)

result1 = mesolve(H, rho0, tlist, [L1], [])
result2 = mesolve(H, rho0, tlist, [L2], [])
result3 = mesolve(H, rho0, tlist, [L3], [])

def bloch_components(rho_list):
    x, y, z = [], [], []
    for rho in rho_list:
        x.append(expect(sigmax(), rho))
        y.append(expect(sigmay(), rho))
        z.append(expect(sigmaz(), rho))
    return np.array(x), np.array(y), np.array(z)

x1, y1, z1 = bloch_components(result1.states)
x2, y2, z2 = bloch_components(result2.states)
x3, y3, z3 = bloch_components(result3.states)

plt.figure(figsize=(10, 6))
plt.plot(tlist, z1, label="Observer 1 (Z basis)")
plt.plot(tlist, z2, label="Observer 2 (X basis)")
plt.plot(tlist, z3, label="Observer 3 (Y basis)")
plt.title("Observer Decoherence Trajectories")
plt.xlabel("Time")
plt.ylabel("Expectation Value ⟨Z⟩")
plt.legend()
plt.grid(True)
plt.tight_layout()
plt.savefig("observer_decoherence_plot.png")
