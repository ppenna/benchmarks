## General configuration
- See [script](./scripts/setup.sh) for general configuration steps.

# Evaluation 
## Cold start echo
```bash
echo "First update all the files in the directory ./config/latency-eval to point to the right files"

make all-cold-start RELEASE=yes
./bin/cold-start-latency -config ./config/latency_eval/eval_config.json > /tmp/results.csv 
```

## Density echo
```bash
echo "First update all the files in the directory ./config/density-eval to point to the right files"

make all-density RELEASE=yes

# Setup networking
# Enable ip forwarding
sudo sh -c "echo 1 > /proc/sys/net/ipv4/ip_forward"
sudo nft add table firecracker
sudo nft 'add chain firecracker postrouting { type nat hook postrouting priority srcnat; policy accept; }'
sudo nft 'add chain firecracker filter { type filter hook forward priority filter; policy accept; }'

# Memory limit defines how much memory will be left in the system before stopping the creation of more instances
./bin/density -config ./config/density_eval/eval_config.json -memory-limit 16384 

sudo nft delete rule firecracker postrouting handle 1
sudo nft delete rule firecracker filter handle 2
sudo nft delete table firecracker
```

### Plot
```bash
cd ${ROOT_DIR}/scripts/plot
python3 -m venv plot-cold-start
source ./plot-cold-start/bin/activate
python3 -m pip install -r ./requirements.txt
python3 ./plot_cold_latency.py /tmp/results.csv /tmp/cold_latency.pdf
```

# Development
## WSL (Ubuntu)
### Hyperlight 
#### Setup
```bash
echo "Installing dependencies"
sudo apt update && sudo apt install clang-11
sudo update-alternatives --install /usr/bin/clang clang /usr/bin/clang-11 100

echo "Installing rust and targets"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"     
rustup target add x86_64-unknown-none
```

#### Compiling
```bash
echo "Compiling"
make all-hyperlight-host
make all-client
```

#### Run
```bash
# To run the host
make run-hyperlight-host

# To run the client
cd hyperlight
make run-client
```
### Unikraft
#### Setup
```bash
curl -sSfL https://get.kraftkit.sh | sh

# (Optional) For zsh run the following commands to allow completion
source ${HOME}/.zsh_kraft_completion;
echo 'source ${HOME}/.zsh_kraft_completion;' >> ${HOME}/.zshrc;
```

#### Compile
```bash
cd ${ROOT_DIR} 
# Suggestion: always run make clean-unikraft-server before, as unikraft compilation has issues sometimes
make all-unikraft-server
```

### Run
```bash
# To run the server
cd ${ROOT_DIR} 
kraft run --rm --plat qemu --arch x86_64 -p 8080:8080 .

# To test the server 
curl -i -X POST -H "Content-Type: application/json" -d '{"data": [1,2]}' 127.0.0.1:8080

# To run hyperlight client
cd hyperlight
make run-client
```

### Firecracker
#### Dependencies
- Install docker. (For WSL, see [this tutorial](https://dev.to/felipecrs/simply-run-docker-on-wsl2-3o8))

#### Setup
```bash
# Rust configurations
rustup target add x86_64-unknown-linux-musl

# To create the rootfs run:
cd scripts/firecracker
./create_rootfs.sh

mkdir -p output
pushd output
ARCH="$(uname -m)"

# Download the kernel/vmlinux (obtained from [here](https://github.com/firecracker-microvm/firecracker/blob/main/docs/getting-started.md)):
latest=$(wget "http://spec.ccfc.min.s3.amazonaws.com/?prefix=firecracker-ci/v1.11/$ARCH/vmlinux-6.1&list-type=2" -O - 2>/dev/null | grep -oP "(?<=<Key>)(firecracker-ci/v1.11/$ARCH/vmlinux-6\.1\.[0-9]{1,3})(?=</Key>)")
wget "https://s3.amazonaws.com/spec.ccfc.min/${latest}"
vm_filename=$(basename ${latest})
mv ${vm_filename} vmlinux.bin

# Download firecracker binary
release_url="https://github.com/firecracker-microvm/firecracker/releases"
latest=$(basename $(curl -fsSLI -o /dev/null -w  %{url_effective} ${release_url}/latest))
curl -L ${release_url}/download/${latest}/firecracker-${latest}-${ARCH}.tgz | tar -xz
mv release-${latest}-$(uname -m)/firecracker-${latest}-${ARCH} firecracker
rm -rf release-${latest}-$(uname -m)
popd
```

### Run
```bash
ROOT_DIR="<repo root dir>"
cd ${ROOT_DIR}/scripts/firecracker/output
cp ${ROOT_DIR}/config/firecracker/vm_config_template.json .
cp ${ROOT_DIR}/config/firecracker/vm_config.json .
touch /tmp/firecracker.log

# Configure network
${ROOT_DIR}/scripts/firecracker/setup_network.sh
# Run server
FC_SOCKET="/tmp/firecracker.socket"
${ROOT_DIR}/scripts/firecracker/output/firecracker --api-sock ${FC_SOCKET} --config-file ${ROOT_DIR}/scripts/firecracker/output/vm_config.json

# To test the server 
curl -i -X POST -H "Content-Type: application/json" -d '{"data": [1,2]}' 172.16.0.2:8080

# To run hyperlight client
cd ${ROOT_DIR}
make run-client 
```

### Eval
```bash
ROOT_DIR="<repo root dir>"
mkdir -p ${ROOT_DIR}/scripts/firecracker/output
touch /tmp/firecracker.log
cp ${ROOT_DIR}/config/firecracker/vm_config.json ${ROOT_DIR}/scripts/firecracker/output/vm_config.json.

pushd $ROOT_DIR
make all-cold-start
./bin/cold-start-latency -config ./config/latency_eval/config.json
popd
```

### Create a snapshot
```bash
# First, start the VM as shown in the Run section for Firecracker
FC_SOCKET="/tmp/firecracker-snapshot.socket"
SNAPSHOT_PATH="/tmp/snapshot_file"
MEMFILE_PATH="/tmp/mem_file"
${ROOT_DIR}/scripts/firecracker/create_snapshot.sh ${FC_SOCKET} ${SNAPSHOT_PATH} ${MEMFILE_PATH}
```