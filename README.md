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
cd hyperlight
make all
```

#### Run
```bash
# To run the host
cd hyperlight
make run-host

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
cd unikraft
KRAFTKIT_TARGET=rust-http-echo cargo +nightly build -Z bu
ild-std=std,panic_abort --target x86_64-unikraft-linux-musl
```

### Run
```bash
# To run the server
cd unikraft
kraft run --rm --plat qemu --arch x86_64 -p 8080:8080 .

# To test the server 
curl localhost:8080

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
cd firecracker
./create_rootfs.sh

mkdir output
pushd output
ARCH="$(uname -m)"

# Downlood the kernel/vmlinux (obtained from [here](https://github.com/firecracker-microvm/firecracker/blob/main/docs/getting-started.md)):
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
cd firecracker
# Configure network
./setup_network.sh
# Run server
sudo ./output/firecracker --api-sock /tmp/firecracker39.socket --config-file ./vm_config.json

# To test the server 
curl -i -X POST -H "Content-Type: application/json" -d '{"data": [1,2]}' 172.16.0.2:8080

# To run hyperlight client
cd hyperlight
make run-client 
```