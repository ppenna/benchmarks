#!/usr/bin/env bash

FC_SOCKET=${1:-"/tmp/firecracker.socket"}
SNAPSHOT_PATH=${2:-"/tmp/snapshot_file"}
MEMFILE_PATH=${3:-"/tmp/mem_file"}

echo "Creating snapshot for VM with socket path: ${FC_SOCKET} with snapshot path: ${SNAPSHOT_PATH} and memfile path: ${MEMFILE_PATH}"

# Pause the VM
curl --unix-socket ${FC_SOCKET} -i \
    -X PATCH 'http://localhost/vm' \
    -H 'Accept: application/json' \
    -H 'Content-Type: application/json' \
    -d '{
            "state": "Paused"
    }'

# Create a snapshot
curl --unix-socket ${FC_SOCKET} -i \
    -X PUT 'http://localhost/snapshot/create' \
    -H  'Accept: application/json' \
    -H  'Content-Type: application/json' \
    -d "{
            \"snapshot_type\": \"Full\",
            \"snapshot_path\": \"${SNAPSHOT_PATH}\",
            \"mem_file_path\": \"${MEMFILE_PATH}\"
    }"

# Resume the VM
curl --unix-socket ${FC_SOCKET} -i \
    -X PATCH 'http://localhost/vm' \
    -H 'Accept: application/json' \
    -H 'Content-Type: application/json' \
    -d '{
            "state": "Resumed"
    }'