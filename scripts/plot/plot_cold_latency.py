import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt
import os

def plot_latency(csv_file: str, save_path: str = None):
    # Read the CSV file into a DataFrame
    df = pd.read_csv(csv_file)
    
    # Make sure the relevant columns are present
    if 'SYSTEM' not in df.columns or 'OP_TYPE' not in df.columns or 'LATENCY_MICROSECONDS' not in df.columns:
        print("Error: CSV must contain 'SYSTEM', 'OP_TYPE', and 'LATENCY_MICROSECONDS' columns.")
        return
    
    # Create the plot
    plt.figure(figsize=(10, 6))
    sns.boxplot(x='SYSTEM', y='LATENCY_MICROSECONDS', hue='OP_TYPE', data=df)
    # Set y-axis to log scale
    plt.yscale('log')

    # Set the title and labels
    plt.title('Latency Microseconds by System and Operation Type')
    plt.xlabel('System')
    plt.ylabel('Latency (Microseconds)')

    # Display the plot
    plt.xticks(rotation=45)  # Rotate x-axis labels for readability
    plt.tight_layout()  # Adjust layout to fit labels

    if save_path:
        # Ensure the directory exists
        os.makedirs(os.path.dirname(save_path), exist_ok=True)
        # Save the figure as a PDF
        plt.savefig(save_path, format='pdf')
        print(f"Figure saved to {save_path}")

# Example usage:
# Assuming you have a CSV file named 'data.csv'
# plot_latency('data.csv')

if __name__ == '__main__':
    import sys
    if len(sys.argv) < 2:
        print("Usage: python plot_cold_latency.py <csv_file> [save_path]")
    else:
        plot_latency(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else None)