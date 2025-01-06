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
source ${HOME}/.zsh_kraft_completion;
echo 'source ${HOME}/.zsh_kraft_completion;' >> ${HOME}/.zshrc;
```

### Run