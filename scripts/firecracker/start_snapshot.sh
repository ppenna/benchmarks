#!/usr/bin/env bash

FC_SOCKET=${1:-"/tmp/firecracker.socket"}
SNAPSHOT_PATH=${2:-"/tmp/snapshot_file"}
MEMFILE_PATH=${3:-"/tmp/mem_file"}

curl --unix-socket ${FC_SOCKET} -i \
    -X PUT 'http://localhost/snapshot/load' \
    -H  'Accept: application/json' \
    -H  'Content-Type: application/json' \
    -d "{
            \"snapshot_path\": \"${SNAPSHOT_PATH}\",
            \"mem_file_path\": \"${MEMFILE_PATH}\"
    }"

curl --unix-socket ${FC_SOCKET} -i \
    -X PATCH 'http://localhost/vm' \
    -H 'Accept: application/json' \
    -H 'Content-Type: application/json' \
    -d '{
            "state": "Resumed"
    }'