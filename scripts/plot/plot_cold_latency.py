import csv
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt
import os
from typing import List

def get_system_name(name: str) -> str:
    if name == "Firecracker-Snapshot": 
        return "Firecracker\nSnapshot"
    return name

def plot_cold_start_latency(csv_file_path: str, save_path_directory: str = None):
    # Read the csv file. The CSV columns are SYSTEM, OP_TYPE, and LATENCY_MICROSECONDS
    new_data: List = [
        ["TYPE", "COLD_START_LATENCY_MICROSECONDS"]
    ]
    with open(csv_file_path) as csv_file:

        reader = csv.reader(csv_file, delimiter=',', quotechar='|')
        current_row: int = 0
        presetup: int = None
        setup: int = None
        first_execution: int = None
        current_system: str = None
        row_counter: int = 0
        for row in reader:
            if row[1] == "PRESETUP":
                current_row = row_counter
                current_system = row[0]
                presetup = int(row[2])
            elif row[1] == "SETUP_SANDBOX":
                # Check that row_counter is equal to current_row + 1
                assert current_row + 1 == row_counter, "Row counter is not equal to current row + 1"
                assert current_system == row[0], "Current system is not equal to row[0]"
                setup = int(row[2]) 
            elif row[1] == "FIRST_EXECUTION":
                # Check that row_counter is equal to current_row + 2
                assert current_row + 2 == row_counter, "Row counter is not equal to current row + 2"
                assert current_system == row[0], "Current system is not equal to row[0]"
                first_execution = int(row[2])
                current_system = get_system_name(current_system)
                new_data.append([current_system, presetup + setup + first_execution])
                # Clean all variables
                current_row = None
                setup = None
                first_execution = None
                current_system = None
            row_counter += 1

    # Create a DataFrame from the new data
    df = pd.DataFrame(new_data[1:], columns=new_data[0])
    # Write down the average per system
    df = df.groupby('TYPE').mean().reset_index() 
    print(df)

    # Sort the  df by TYPE by the order: Unikraft, Firecracker, Fireckare-Snapshot, Hyperlight, Process
    order = ['Unikraft', 'Firecracker', 'Firecracker\nSnapshot', 'Hyperlight', 'Process']
    df['TYPE'] = pd.Categorical(df['TYPE'], categories=order, ordered=True)
    df = df.sort_values('TYPE')
    df.reset_index(drop=True, inplace=True)

    # Create the plot
    plt.figure(figsize=(5.5, 3))
    sns.barplot(x='TYPE', y='COLD_START_LATENCY_MICROSECONDS', data=df, capsize=0.2)
    # Set the title and labels
    # plt.title('Cold Start Execution Latency by System')
    plt.xlabel('')
    plt.ylabel('Cold Start')
    # log scale
    plt.yscale('log')
    plt.yticks([10**3, 10**4, 10**5, 10**6, 10**7], labels=['1ms', '10ms', '100ms', '1s', '10s'])
    plt.grid(axis='y', linestyle='--', linewidth=0.7)



    # Display the plot
    # plt.xticks(rotation=45)  # Rotate x-axis labels for readability
    plt.tight_layout(pad=0)  # Adjust layout to fit labels
    if save_path_directory:
        # Ensure the directory exists
        os.makedirs(save_path_directory, exist_ok=True)
        save_path = os.path.join(save_path_directory, 'cold_start_latency.pdf')
        # Save the figure as a PDF
        plt.savefig(save_path, format='pdf')
        print(f"Figure saved to {save_path}")
    else:
        plt.show()



def plot_warm_start_latency(csv_file_path: str, save_path_directory: str = None):
    # Read the csv file. The CSV columns are SYSTEM, OP_TYPE, and LATENCY_MICROSECONDS
    new_data: List = [
        ["TYPE", "WARM_START_LATENCY_MICROSECONDS"]
    ]
    with open(csv_file_path) as csv_file:

        reader = csv.reader(csv_file, delimiter=',', quotechar='|')

        for row in reader:
            if row[1] == "EXECUTION":
                # Check that row_counter is equal to current_row + 3
                
                new_data.append([get_system_name(row[0]), int(row[2])])

    # Create a DataFrame from the new data
    df = pd.DataFrame(new_data[1:], columns=new_data[0])

    # Sort the  df by TYPE by the order: Unikraft, Firecracker, Fireckare-Snapshot, Hyperlight, Process
    order = ['Unikraft', 'Firecracker', 'Firecracker\nSnapshot', 'Hyperlight', 'Process']
    df['TYPE'] = pd.Categorical(df['TYPE'], categories=order, ordered=True)
    df = df.sort_values('TYPE')
    df.reset_index(drop=True, inplace=True)

    # Create the plot
    plt.figure(figsize=(5.5, 3))

    sns.barplot(x='TYPE', y='WARM_START_LATENCY_MICROSECONDS', data=df, capsize=0.2)

    # Set the title and labels
    # plt.title('Warm Start Execution Latency by System')
    plt.xlabel('')
    plt.ylabel('Warm Start (Microseconds)')
    plt.grid(axis='y', linestyle='--', linewidth=0.7)

    # Display the plot
    # plt.xticks(rotation=45)  # Rotate x-axis labels for readability
    plt.tight_layout()  # Adjust layout to fit labels
    if save_path_directory:
        # Ensure the directory exists
        os.makedirs(save_path_directory, exist_ok=True)
        save_path = os.path.join(save_path_directory, 'warm_start_latency.pdf')
        # Save the figure as a PDF
        plt.savefig(save_path, format='pdf')
        print(f"Figure saved to {save_path}")
    else:
        plt.show()

def original_plot_latency(csv_file: str, save_path: str = None):
    # Read the CSV file into a DataFrame
    df = pd.read_csv(csv_file)
    
    # Make sure the relevant columns are present
    if 'SYSTEM' not in df.columns or 'OP_TYPE' not in df.columns or 'LATENCY_MICROSECONDS' not in df.columns:
        print("Error: CSV must contain 'SYSTEM', 'OP_TYPE', and 'LATENCY_MICROSECONDS' columns.")
        return
    
    # Create the plot
    plt.figure(figsize=(10, 6))
    sns.barplot(x='SYSTEM', y='LATENCY_MICROSECONDS', hue='OP_TYPE', data=df)
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
        print("Usage: python plot_cold_latency.py <csv_file> [save_path_directory]")
    else:
        original_plot_latency(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else None)
        # plot_cold_start_latency(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else None)
        # plot_warm_start_latency(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else None)