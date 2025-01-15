#!/usr/bin/env bash

SCRIPT_DIR=$(dirname $(realpath $0))
ROOT_DIR=${SCRIPT_DIR}/../

echo "Installing Rust"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"     
rustup target add x86_64-unknown-none
rustup target add x86_64-unknown-linux-musl
rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

echo "Installing Unikraft"
curl -sSfL https://get.kraftkit.sh | sh

# Configure KVM for unikraft 
sudo usermod -a -G kvm ${USER}
sudo chown -v root:kvm /dev/kvm && sudo chmod 660 /dev/kvm

# Install docker
sudo apt-get update

# Ensures not older packages are installed
sudo apt-get remove docker docker-engine docker.io containerd runc

# Ensure pre-requisites are installed
sudo apt-get update
sudo apt-get install \
     ca-certificates \
     curl \
     gnupg \
     lsb-release

# Adds docker apt key
sudo mkdir -p /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg

# Adds docker apt repository
echo \
    "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
    $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# Refreshes apt repos
sudo apt-get update

# Installs Docker CE
sudo apt-get install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# Ensures docker group exists
sudo groupadd docker

# Ensures you are part of it
sudo usermod -aG docker $USER

# Configuring firecrakcker
## Setup rootfs
pushd ${SCRIPT_DIR}/../scripts/firecracker
./create_rootfs.sh

## Install firecracker
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

# Configure files for firecracker
pushd ${ROOT_DIR}/scripts/firecracker/output
cp ${ROOT_DIR}/config/firecracker/vm_config_template.json .
touch /tmp/firecracker.log

# Set KVM permissions
sudo setfacl -m u:${USER}:rw /dev/kvm
[ $(stat -c "%G" /dev/kvm) = kvm ] && sudo usermod -aG kvm ${USER} \
&& echo "Access granted."

echo "Manually create a snapshot for firecracker following the README"