# Geth

## Installation

Install both `geth` and `bootnode.

```sh
git clone https://github.com/ethereum/go-ethereum.git
cd go-ethereum
git checkout v1.14.11 # The latest tested version
make all
sudo cp build/bin/geth /usr/local/bin/geth # Make it globally accessible
sudo cp build/bin/bootnode /usr/local/bin/bootnode # Make it globally accessible
```

Or consult the [official page](https://geth.ethereum.org/docs/getting-started/installing-geth) for the installation.