import matplotlib.pyplot as plt
from matplotlib import cm, colors
from mpl_toolkits.mplot3d import Axes3D
import numpy as np
from scipy.special import sph_harm

phi = np.linspace(0, 2*np.pi, 100)
theta = np.linspace(0, np.pi, 100)
lmax = 10

# Create a 3D plot
ax = plt.subplot(111, projection='3d')

# Plot the spherical harmonics
for l in range(lmax+1):
    for m in range(-l, l+1):
        Ylm = sph_harm(m, l, phi, theta)
        ax.plot_surface(np.cos(phi)*np.sin(theta), np.sin(phi)*np.sin(theta), Ylm, cmap=cm.jet)

# Set the axes labels
ax.set_xlabel('X')
ax.set_ylabel('Y')
ax.set_zlabel('Z')

# Show the plot
plt.show()
