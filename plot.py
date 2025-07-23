import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# Read data from csv file
df = pd.read_csv('output.csv')

# Strip whitespace from column names
df.columns = df.columns.str.strip()

# Define bin size along the time axis (number of rows per bin)
bin_size = 5  # Feel free to adjust based on your data density

# Create bins: group df rows into bins of fixed size
# Calculate how many full bins we will have
num_bins = len(df) // bin_size

# Trim the dataframe so it fits into an integer number of bins
df_trimmed = df.iloc[:num_bins * bin_size]

# Reshape and average data inside bins
binned_time = df_trimmed['Time'].values.reshape(-1, bin_size).mean(axis=1)

# For voltage columns
binned_real_voltage = df_trimmed['Real Voltage'].values.reshape(-1, bin_size).mean(axis=1)
binned_sim_voltage = df_trimmed['Simulated Voltage'].values.reshape(-1, bin_size).mean(axis=1)

# For concentration columns
concentration_cols = ['c1c', 'c0c', 'c1a', 'c2a']
binned_concentrations = {}
for col in concentration_cols:
    binned_concentrations[col] = df_trimmed[col].values.reshape(-1, bin_size).mean(axis=1)

# Plotting setup
fig, axs = plt.subplots(1, 2, figsize=(14, 6))

line_color = 'black'
markers_voltage = ['o', 's']
markers_conc = ['x', '^', 'v', 'D']
markersize = 4

# Plot 1: Binned voltage data
for marker, data, label in zip(markers_voltage,
                               [binned_real_voltage, binned_sim_voltage],
                               ['Real Voltage', 'Simulated Voltage']):
    axs[0].plot(binned_time, data, marker=marker, color=line_color, linestyle='-', markersize=markersize, label=label)

axs[0].set_xlabel('Time (s)')
axs[0].set_ylabel('Voltage (V)')
axs[0].legend()
axs[0].grid(True)

# Plot 2: Binned concentration data
for marker, col in zip(markers_conc, concentration_cols):
    axs[1].plot(binned_time, binned_concentrations[col], marker=marker,
                color=line_color, linestyle='-', markersize=markersize, label=col)

axs[1].set_xlabel('Time (s)')
axs[1].set_ylabel('Concentration (mol/m³)')
axs[1].legend()
axs[1].grid(True)

plt.tight_layout()
plt.show()
