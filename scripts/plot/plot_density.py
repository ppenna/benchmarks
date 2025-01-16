#!/usr/bin/env python

import csv
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt
import os
from typing import List
import matplotlib.ticker as ticker



def plot_density(csv_file: str, save_path_directory: str = None):
    # Read the CSV file into a DataFrame
    df = pd.read_csv(csv_file)

    # The columns in the CSV file are SYSTEM,INITIAL_MEM,FINAL_MEM,MAX_INSTANCES
    # Make sure the relevant columns are present
    if 'SYSTEM' not in df.columns or 'INITIAL_MEM' not in df.columns or 'FINAL_MEM' not in df.columns or 'MAX_INSTANCES' not in df.columns:
        raise ValueError("CSV file must contain columns SYSTEM, INITIAL_MEM, FINAL_MEM, and MAX_INSTANCES")
    
    # Sort the  df by TYPE by the order: Unikraft, Firecracker, Hyperlight, Process
    order = ['Unikraft', 'Firecracker', 'Hyperlight', 'Process']
    df['SYSTEM'] = pd.Categorical(df['SYSTEM'], categories=order, ordered=True)
    df = df.sort_values('SYSTEM')
    df.reset_index(drop=True, inplace=True)

    # Overwrite Unikraft to be "Unikraft\n+ QEMU"
    df['SYSTEM'] = df['SYSTEM'].replace('Unikraft', 'Unikraft\n+ QEMU')

    # Print the average memory usage per system
    for row in df.iterrows():
        print(f"Average memory usage for {row[1]['SYSTEM']}: { (row[1]['INITIAL_MEM'] - row[1]['FINAL_MEM']) / row[1]['MAX_INSTANCES'] } MB")

    
     # Create the plot
    plt.figure(figsize=(5.5, 3))
    sns.barplot(x='SYSTEM', y='MAX_INSTANCES', data=df)
    plt.xlabel('')
    plt.ylabel('Max Number of Instances')
    plt.grid(axis='y', linestyle='--', linewidth=0.7)
    # Set y-axis to be in the range [1, 10000]
    plt.ylim(10, 10000)
    # log scale y axis
    plt.yscale('log')
    # Set the y-axis ticks to be powers of 10
    plt.yticks([10**1, 10**2, 10**3, 10**4, 10**5], labels=['10', '100', '1000', '10⁴', '10⁵'])

    # Make all the bars the same color: blue
    # Make all the bars have the same color as the first bar
    for i, bar in enumerate(plt.gca().patches):
        if i == 0:
            color = bar.get_facecolor()
        bar.set_color(color)
    
    # Display the plot
    plt.tight_layout(pad=0)  # Adjust layout to fit labels
    if save_path_directory:
        # Ensure the directory exists
        os.makedirs(save_path_directory, exist_ok=True)
        save_path = os.path.join(save_path_directory, 'density_plot.pdf')
        # Save the figure as a PDF
        plt.savefig(save_path, format='pdf')
        print(f"Figure saved to {save_path}")
    else:
        plt.show()

if __name__ == '__main__':
    import sys
    if len(sys.argv) < 2:
        print("Usage: python plot_density.py <csv_file> [save_path_directory]")
    else:
        plot_density(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else None)