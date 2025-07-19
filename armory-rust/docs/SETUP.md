# Setup Guide - Armory Rust

> **Complete installation and configuration guide for the Armory Bitcoin wallet Rust implementation**

## 🚀 Quick Start

### System Requirements

**Minimum Requirements**:
- **OS**: Linux, macOS, or Windows 10+
- **RAM**: 4GB (8GB recommended)
- **Storage**: 10GB free space
- **Internet**: Required for Bitcoin network connectivity

**Recommended Requirements**:
- **OS**: Linux (Ubuntu 20.04+) or macOS 12+
- **RAM**: 16GB+ for full node operation
- **Storage**: 1TB+ SSD for full blockchain
- **CPU**: 4+ cores for optimal performance

### Prerequisites Installation

#### 1. Install Rust

**Linux/macOS**:
```bash
# Install rustup (Rust toolchain installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add Rust to PATH
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

**Windows**:
1. Download and run [rustup-init.exe](https://rustup.rs/)
2. Follow the installer prompts
3. Restart your terminal
4. Verify: `rustc --version`

#### 2. Install Git

**Linux (Ubuntu/Debian)**:
```bash
sudo apt update
sudo apt install git
```

**macOS**:
```bash
# Using Homebrew
brew install git

# Or install Xcode Command Line Tools
xcode-select --install
```

**Windows**:
- Download and install [Git for Windows](https://git-scm.com/download/win)

#### 3. Install Bitcoin Core (Optional but Recommended)

**Linux**:
```bash
# Add Bitcoin Core PPA
sudo add-apt-repository ppa:bitcoin/bitcoin
sudo apt update
sudo apt install bitcoind bitcoin-qt
```

**macOS**:
```bash
# Using Homebrew
brew install bitcoin
```

**Windows**:
- Download from [bitcoincore.org](https://bitcoincore.org/en/download/)

## 📦 Installation Methods

### Method 1: From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/armory/armory-rust.git
cd armory-rust

# Build in release mode
cargo build --release

# Run tests to verify installation
cargo test

# Install CLI tool globally
cargo install --path .

# Verify installation
armory-rust --version
```

### Method 2: Using Cargo

```bash
# Install from crates.io (when published)
cargo install armory-rust

# Or install from Git repository
cargo install --git https://github.com/armory/armory-rust.git
```

### Method 3: Pre-built Binaries

Download pre-built binaries from the [releases page](https://github.com/armory/armory-rust/releases):

**Linux**:
```bash
wget https://github.com/armory/armory-rust/releases/latest/download/armory-rust-linux-x86_64.tar.gz
tar -xzf armory-rust-linux-x86_64.tar.gz
sudo mv armory-rust /usr/local/bin/
```

**macOS**:
```bash
wget https://github.com/armory/armory-rust/releases/latest/download/armory-rust-macos-x86_64.tar.gz
tar -xzf armory-rust-macos-x86_64.tar.gz
sudo mv armory-rust /usr/local/bin/
```

**Windows**:
- Download `armory-rust-windows-x86_64.zip`
- Extract to a directory in your PATH

## ⚙️ Configuration

### Initial Setup

```bash
# Create configuration directory
mkdir -p ~/.armory-rust

# Initialize configuration
armory-rust init

# Edit configuration file
nano ~/.armory-rust/config.toml
```

### Configuration File

**Default Configuration** (`~/.armory-rust/config.toml`):

```toml
[general]
# Bitcoin network to use
network = "mainnet"  # Options: mainnet, testnet, signet, regtest

# Data directory for wallets and storage
data_dir = "~/.armory-rust/data"

# Log level
log_level = "info"  # Options: trace, debug, info, warn, error

[storage]
# Enable automatic backup
auto_backup = true

# Number of backup files to keep
backup_count = 5

# Backup directory
backup_dir = "~/.armory-rust/backups"

[network]
# Bitcoin Core RPC connection (optional)
rpc_host = "127.0.0.1"
rpc_port = 8332
rpc_user = "bitcoin"
rpc_password = "your-rpc-password"

# P2P network settings
p2p_port = 8333
max_peers = 8

[fees]
# Default fee strategy
default_strategy = "normal"

# Fee rate limits (sat/vB)
min_fee_rate = 1
max_fee_rate = 1000

[security]
# Require hardware wallet for transactions above this amount (satoshis)
hw_threshold = 100000000  # 1 BTC

# Enable additional security measures
paranoid_mode = false
```

### Environment Variables

You can also configure Armory Rust using environment variables:

```bash
# Set network
export ARMORY_NETWORK=testnet

# Set data directory
export ARMORY_DATA_DIR=/custom/data/path

# Set log level
export RUST_LOG=armory_rust=debug

# Bitcoin Core RPC settings
export BITCOIN_RPC_HOST=127.0.0.1
export BITCOIN_RPC_PORT=18332
export BITCOIN_RPC_USER=testuser
export BITCOIN_RPC_PASSWORD=testpass
```

## 🏦 Wallet Setup

### Creating Your First Wallet

```bash
# Create a new wallet
armory-rust wallet create --name "my-wallet" --network mainnet

# Create a testnet wallet for testing
armory-rust wallet create --name "test-wallet" --network testnet

# Create wallet with custom path
armory-rust wallet create \
  --name "secure-wallet" \
  --network mainnet \
  --data-dir "/secure/location"
```

### Wallet Security Setup

```bash
# Set wallet password (recommended)
armory-rust wallet password --name "my-wallet" --set

# Enable hardware wallet integration
armory-rust wallet hw-setup --name "my-wallet" --device ledger

# Create encrypted backup
armory-rust wallet backup --name "my-wallet" --encrypt
```

### Importing Legacy Wallets

```bash
# Import legacy Armory wallet
armory-rust wallet import \
  --file "/path/to/legacy.wallet" \
  --name "imported-wallet" \
  --network mainnet

# Import with passphrase
armory-rust wallet import \
  --file "/path/to/legacy.wallet" \
  --name "imported-wallet" \
  --passphrase "your-passphrase"

# Import watching-only wallet
armory-rust wallet import \
  --file "/path/to/watching.wallet" \
  --name "watch-only" \
  --watch-only
```

## 🌐 Network Configuration

### Bitcoin Core Integration

**1. Configure Bitcoin Core** (`bitcoin.conf`):

```ini
# Enable RPC server
server=1
rpcallowip=127.0.0.1
rpcuser=armory
rpcpassword=your-secure-password

# Enable transaction indexing
txindex=1

# Reduce memory usage (optional)
dbcache=1000

# Enable SegWit and Taproot
deprecatedrpc=create_bdb
```

**2. Start Bitcoin Core**:
```bash
# Linux/macOS
bitcoind -daemon

# Windows
bitcoin-qt.exe -server
```

**3. Configure Armory Rust**:
```bash
# Test connection
armory-rust network test-rpc

# Sync wallet with Bitcoin Core
armory-rust wallet sync --name "my-wallet"
```

### Electrum Server Integration

```bash
# Configure Electrum server
armory-rust config set network.electrum_host "electrum.example.com"
armory-rust config set network.electrum_port 50002
armory-rust config set network.electrum_ssl true

# Test connection
armory-rust network test-electrum

# Use Electrum for syncing
armory-rust wallet sync --name "my-wallet" --electrum
```

### Tor Integration

```bash
# Install Tor
sudo apt install tor  # Linux
brew install tor      # macOS

# Configure Tor proxy
armory-rust config set network.proxy_host "127.0.0.1"
armory-rust config set network.proxy_port 9050
armory-rust config set network.use_tor true

# Start syncing over Tor
armory-rust wallet sync --name "my-wallet" --tor
```

## 🔐 Security Configuration

### Hardware Wallet Setup

**Ledger Setup**:
```bash
# Install HWI (Hardware Wallet Interface)
pip3 install hwi

# Detect Ledger device
armory-rust hw detect

# Set up Ledger with wallet
armory-rust wallet hw-setup \
  --name "my-wallet" \
  --device ledger \
  --derivation "m/84'/0'/0'"
```

**Trezor Setup**:
```bash
# Detect Trezor device
armory-rust hw detect --device trezor

# Set up Trezor with wallet
armory-rust wallet hw-setup \
  --name "my-wallet" \
  --device trezor \
  --derivation "m/84'/0'/0'"
```

### Advanced Security Features

```bash
# Enable multisig wallet (2-of-3)
armory-rust wallet create-multisig \
  --name "multisig-wallet" \
  --threshold 2 \
  --cosigners 3 \
  --network mainnet

# Set up time-locked wallet
armory-rust wallet create-timelock \
  --name "timelock-wallet" \
  --locktime 144  # blocks
  --network mainnet

# Enable paranoid mode
armory-rust config set security.paranoid_mode true
armory-rust config set security.require_confirmation true
```

## 🔧 Development Setup

### Development Environment

```bash
# Clone for development
git clone https://github.com/armory/armory-rust.git
cd armory-rust

# Install development dependencies
cargo install cargo-audit
cargo install cargo-machete
cargo install cargo-tarpaulin
cargo install flamegraph

# Set up pre-commit hooks
cp scripts/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

# Run development build
cargo build

# Run tests with coverage
cargo tarpaulin --out Html
```

### VS Code Setup

**Install Extensions**:
- Rust Analyzer
- Error Lens
- GitLens
- Better TOML

**VS Code Settings** (`.vscode/settings.json`):
```json
{
    "rust-analyzer.check.command": "clippy",
    "rust-analyzer.check.features": "all",
    "rust-analyzer.cargo.features": "all",
    "[rust]": {
        "editor.defaultFormatter": "rust-lang.rust-analyzer"
    }
}
```

### Testing Setup

```bash
# Set up test environment
export RUST_LOG=debug
export ARMORY_TEST_DATA_DIR=/tmp/armory-test

# Run unit tests
cargo test

# Run integration tests
cargo test --test integration

# Run with coverage
cargo tarpaulin --all-features --workspace --timeout 120

# Performance testing
cargo bench
```

## 🐛 Troubleshooting

### Common Issues

**Issue**: Build fails with missing dependencies
```bash
# Solution: Update Rust and dependencies
rustup update
cargo update
cargo clean && cargo build
```

**Issue**: Permission denied on Linux
```bash
# Solution: Fix permissions
sudo chown -R $USER:$USER ~/.armory-rust
chmod 700 ~/.armory-rust
```

**Issue**: Wallet won't sync
```bash
# Solution: Check network connectivity
armory-rust network test-connection
armory-rust wallet rescan --name "my-wallet"
```

**Issue**: Hardware wallet not detected
```bash
# Solution: Check device and permissions
lsusb | grep -i ledger  # Linux
system_profiler SPUSBDataType | grep -i ledger  # macOS

# Fix udev rules (Linux)
sudo cp contrib/udev/51-usb-armory.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
```

### Log Analysis

```bash
# Enable detailed logging
export RUST_LOG=armory_rust=trace

# View logs in real-time
tail -f ~/.armory-rust/logs/armory.log

# Search for errors
grep -i error ~/.armory-rust/logs/armory.log

# Analyze performance
grep -i "slow" ~/.armory-rust/logs/armory.log
```

### Recovery Procedures

**Wallet Recovery**:
```bash
# Recover from backup
armory-rust wallet recover \
  --backup-file "/path/to/backup.wallet" \
  --name "recovered-wallet"

# Recover from seed phrase
armory-rust wallet recover-seed \
  --name "recovered-wallet" \
  --words 24

# Check wallet integrity
armory-rust wallet verify --name "my-wallet"
```

**Database Recovery**:
```bash
# Repair corrupted database
armory-rust db repair --data-dir ~/.armory-rust/data

# Rebuild from scratch
armory-rust db rebuild --name "my-wallet"

# Export wallet data
armory-rust wallet export --name "my-wallet" --format json
```

## 📊 Performance Optimization

### System Optimization

**Linux**:
```bash
# Increase file descriptor limits
echo "* soft nofile 65536" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65536" | sudo tee -a /etc/security/limits.conf

# Optimize disk I/O
echo "deadline" | sudo tee /sys/block/sda/queue/scheduler

# Enable swap if needed
sudo swapon --show
```

**macOS**:
```bash
# Increase file descriptor limits
sudo launchctl limit maxfiles 65536 200000

# Optimize SSD
sudo trimforce enable
```

### Application Optimization

```bash
# Use release builds for better performance
cargo build --release

# Enable CPU-specific optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Reduce memory usage
armory-rust config set storage.cache_size 100MB
armory-rust config set network.max_peers 4
```

## 🔄 Backup and Recovery

### Automated Backups

```bash
# Enable automatic backups
armory-rust config set storage.auto_backup true
armory-rust config set storage.backup_interval 3600  # seconds

# Manual backup
armory-rust wallet backup --name "my-wallet" --encrypt

# Backup to external location
armory-rust wallet backup \
  --name "my-wallet" \
  --output "/backup/location/wallet-backup.enc"
```

### Recovery Testing

```bash
# Test recovery process
armory-rust wallet test-recovery \
  --backup-file "/path/to/backup.wallet" \
  --dry-run

# Verify backup integrity
armory-rust backup verify --file "/path/to/backup.wallet"
```

## 📱 Platform-Specific Notes

### Linux

- **Package Manager**: Use system package manager for dependencies
- **Permissions**: Set up proper udev rules for hardware wallets
- **Systemd**: Can run as system service for server deployment

### macOS

- **Homebrew**: Recommended for installing dependencies
- **Keychain**: Integrate with macOS Keychain for password storage
- **Sandboxing**: May require security permissions for hardware wallets

### Windows

- **Administrator**: Some operations may require administrator privileges
- **Antivirus**: Exclude Armory directories from real-time scanning
- **Windows Defender**: Add exclusions for better performance

---

## 🆘 Getting Help

If you encounter issues during setup:

1. **Check the logs**: `~/.armory-rust/logs/armory.log`
2. **Search existing issues**: [GitHub Issues](https://github.com/armory/armory-rust/issues)
3. **Join the community**: [Discord](https://discord.gg/armory) | [Telegram](https://t.me/ArmoryWallet)
4. **Create an issue**: Provide logs and system information

For security-related issues, please contact: security@armory.com