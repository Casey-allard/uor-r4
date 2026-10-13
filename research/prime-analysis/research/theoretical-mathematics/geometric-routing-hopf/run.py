import matplotlib.pyplot as plt
import numpy as np

# Define residues and anchors for the example
residues = [7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 1]
anchors = [12, 182, 870, 1892, 3782, 6320, 9506, 11990, 18906, 24806, 32220, 
           39006, 51756, 58222, 72630, 80252, 98282, 120756, 135056, 151710, 6]

# Convert residues and anchors into polar coordinates
theta = np.linspace(0, 2 * np.pi, len(anchors))
radii = np.array(anchors)
z = np.array(residues)

fig = plt.figure(figsize=(10, 8))
ax = fig.add_subplot(111, projection='3d')

# Plot the spiral
ax.plot(np.sin(theta) * radii, np.cos(theta) * radii, z, linestyle='--', color='gray', alpha=0.6)
ax.scatter(np.sin(theta) * radii, np.cos(theta) * radii, z, c=z, cmap='viridis', s=50)

# Annotate key points
for i in range(len(anchors)):
    ax.text(np.sin(theta[i]) * radii[i], np.cos(theta[i]) * radii[i], z[i], 
            f'{anchors[i]}', fontsize=8, color='black')

# Labels and Title
ax.set_title('Refined 3D Polar Prime Residue Spiral')
ax.set_xlabel('Polar X')
ax.set_ylabel('Polar Y')
ax.set_zlabel('Residue (Z-Axis)')

plt.show()
