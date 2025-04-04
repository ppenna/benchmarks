#!/usr/bin/env python3

import os
import json

def generate_base_configs():
    # Get the absolute path of the script
    script_directory = os.path.dirname(os.path.abspath(__file__))
    code_root_dir = os.path.dirname(script_directory) 
    generate_latency_configs(code_root_dir)
    generate_density_configs(code_root_dir)

def generate_latency_configs(base_dir):
    latency_config_output = f"{base_dir}/config/latency_eval"
    make_directory(latency_config_output)
    latency_unikraft_config = generate_latency_unikraft_config(base_dir)
    save_json_to_file(latency_unikraft_config, f"{latency_config_output}/unikraft_config.json")
    latency_process_config = generate_latency_process_config(base_dir)
    save_json_to_file(latency_process_config, f"{latency_config_output}/process_config.json")
    latency_hyperlight_config = generate_latency_hyperlight_config(base_dir)
    save_json_to_file(latency_hyperlight_config, f"{latency_config_output}/hyperlight_config.json")
    latency_firecracker_snapshot_config = generate_firecracker_snapshot_config(base_dir)
    save_json_to_file(latency_firecracker_snapshot_config, f"{latency_config_output}/firecracker_snapshot_config.json")
    latency_firecracker_config = generate_firecracker_config(base_dir)
    save_json_to_file(latency_firecracker_config, f"{latency_config_output}/firecracker_config.json")
    latency_eval_config = generate_latency_eval_config(base_dir)
    save_json_to_file(latency_eval_config, f"{latency_config_output}/eval_config.json")

def generate_density_configs(base_dir):
    density_config_output = f"{base_dir}/config/density_eval"
    make_directory(density_config_output)
    density_unikraft_config = generate_density_unikraft_config(base_dir)
    save_json_to_file(density_unikraft_config, f"{density_config_output}/unikraft_config.json")
    density_process_config = generate_density_process_config(base_dir)
    save_json_to_file(density_process_config, f"{density_config_output}/process_config.json")
    density_hyperlight_config = generate_density_hyperlight_config(base_dir)
    save_json_to_file(density_hyperlight_config, f"{density_config_output}/hyperlight_config.json")
    density_firecracker_config = generate_density_firecracker_config(base_dir)
    save_json_to_file(density_firecracker_config, f"{density_config_output}/firecracker_config.json")
    density_eval_config = generate_density_eval_config(base_dir)
    save_json_to_file(density_eval_config, f"{density_config_output}/eval_config.json")

def make_directory(directory):
    if not os.path.exists(directory):
        os.makedirs(directory)

def save_json_to_file(json_data, file_path):
    with open(file_path, 'w') as json_file:
        json.dump(json_data, json_file, indent=4)

def generate_latency_unikraft_config(base_dir):
    base_json = {
        "guest_port": 8080,
        "host_port": 8080,
        "run_dir": base_dir,
        "memory": "128Mi",
        "output_dir": "/tmp"
    }

    return base_json

def generate_latency_process_config(base_dir):
    base_json = {    
        "ip": "127.0.0.1",
        "port": 8080,
        "binary_path": f"{base_dir}/bin/rust-http-echo",
        "output_dir": "/tmp"
    } 

    return base_json

def generate_latency_hyperlight_config(base_dir):
    base_json = {
        "guest_binary": f"{base_dir}/bin/hyperlight-guest-nanvix",
        "host_binary": f"{base_dir}/bin/hyperlight-host-nanvix",
        "listen_ip": "127.0.0.1",
        "listen_port": 8080,
        "output_dir": "/tmp"   
    } 
    return base_json

def generate_firecracker_snapshot_config(base_dir):
    base_json = {
        "firecracker_binary_dir": f"{base_dir}/scripts/firecracker/output", 
        "start_snapshot_script": f"{base_dir}/scripts/firecracker/start_snapshot.sh",
        "firecracker_socket_prefix": "/tmp/firecracker",
        "snapshot_file": "/tmp/snapshot_file",
        "mem_file": "/tmp/mem_file",
        "network_setup_file": f"{base_dir}/scripts/firecracker/setup_network.sh",
        "network_cleanup_file": f"{base_dir}/scripts/firecracker/clean_network.sh",
        "output_dir": "/tmp"
    } 

    return base_json

def generate_firecracker_config(base_dir):
    base_json = {
        "firecracker_binary_dir": f"{base_dir}/scripts/firecracker/output", 
        "firecracker_socket_prefix": "/tmp/firecracker",
        "config_file_template": f"{base_dir}/scripts/firecracker/output/vm_config_template.json",
        "network_setup_file": f"{base_dir}/scripts/firecracker/setup_network.sh",
        "network_cleanup_file": f"{base_dir}/scripts/firecracker/clean_network.sh"
    }

    return base_json

def generate_latency_eval_config(base_dir):
    base_json = {
        "evals": [
            {
                "type_of_eval": "process",
                "config_location": f"{base_dir}/config/latency_eval/process_config.json"
            },
            {
                "type_of_eval": "unikraft",
                "config_location": f"{base_dir}/config/latency_eval/unikraft_config.json"
            },
            {
                "type_of_eval": "hyperlight",
                "config_location": f"{base_dir}/config/latency_eval/hyperlight_config.json"
            },
            {
                "type_of_eval": "firecracker-snapshot",
                "config_location": f"{base_dir}/config/latency_eval/firecracker_snapshot_config.json"
            },
            {
                "type_of_eval": "firecracker",
                "config_location": f"{base_dir}/config/latency_eval/firecracker_config.json"
            }
        ]
    }

    return base_json

def generate_density_unikraft_config(base_dir):
    base_json = {
        "guest_port": 8080,
        "host_port": 8080,
        "run_dir": base_dir,
        "memory": "128Mi",
        "output_dir": "/tmp"
    }

    return base_json

def generate_density_process_config(base_dir):
    base_json = {    
        "ip": "127.0.0.1",
        "port": 8080,
        "binary_path": f"{base_dir}/bin/rust-http-echo",
        "output_dir": "/tmp"
    }

    return base_json

def generate_density_hyperlight_config(base_dir):
    base_json = {
        "guest_binary": f"{base_dir}/bin/hyperlight-guest-nanvix",
        "host_binary": f"{base_dir}/bin/hyperlight-host-nanvix",
        "listen_ip": "127.0.0.1",
        "listen_port": 8080,
        "output_dir": "/tmp"   
    }

    return base_json

def generate_density_firecracker_config(base_dir):
    base_json = {
        "firecracker_binary_dir": f"{base_dir}/scripts/firecracker/output", 
        "firecracker_socket_prefix": "/tmp/firecracker",
        "config_file_template": f"{base_dir}/scripts/firecracker/output/vm_config_template.json",
        "network_setup_file": f"{base_dir}/scripts/firecracker/setup_network.sh",
        "network_cleanup_file": f"{base_dir}/scripts/firecracker/clean_network.sh"
    }

    return base_json

def generate_density_eval_config(base_dir):
    base_json = {
        "evals": [
            {
                "type_of_eval": "unikraft",
                "config_location": f"{base_dir}/config/density_eval/unikraft_config.json"
            },
            {
                "type_of_eval": "firecracker",
                "config_location": f"{base_dir}/config/density_eval/firecracker_config.json"
            },
            {
                "type_of_eval": "process",
                "config_location": f"{base_dir}/config/density_eval/process_config.json"
            },
            {
                "type_of_eval": "hyperlight",
                "config_location": f"{base_dir}/config/density_eval/hyperlight_config.json"
            }
        ]
    }

    return base_json

if __name__ == "__main__":
    generate_base_configs()