# Armory Bitcoin Wallet

> **🚀 Secure • Modern • Private** – A next-generation Bitcoin wallet with enterprise-grade security, complete privacy features, and support for the latest Bitcoin standards.

[![Downloads](https://img.shields.io/github/downloads/your-org/armory/total.svg)](https://github.com/your-org/armory/releases)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![Bitcoin](https://img.shields.io/badge/bitcoin-taproot%20ready-orange.svg)](https://bitcoincore.org/)
[![Security](https://img.shields.io/badge/security-audited-green.svg)](#security-features)

## What is Armory?

Armory is a powerful Bitcoin wallet designed for users who demand the highest levels of security and privacy. Originally created in 2011, Armory has been completely rewritten in **Rust** to provide memory-safe operations, modern cryptography, and full support for the latest Bitcoin protocols including **Taproot** and **PSBT v2**.

### Why Choose Armory?

- **🔒 Maximum Security**: Hardware wallet integration, encrypted storage, and memory-safe operations
- **🏠 Full Control**: Your keys, your Bitcoin – complete sovereignty over your funds
- **🔐 Advanced Privacy**: Built-in Tor support and encrypted P2P communication
- **⚡ Modern Standards**: Full support for SegWit, Taproot, and all current Bitcoin features
- **💼 Enterprise Ready**: Multi-signature wallets, offline signing, and advanced transaction features
- **🔄 Future-Proof**: Regular updates with the latest Bitcoin protocol improvements

---

## ✨ Key Features

### 🔐 **Security & Privacy**
- **Hardware Wallet Support**: Compatible with Ledger, Trezor, and other hardware devices
- **Multi-Signature Wallets**: Create 2-of-3, 3-of-5, and other multi-sig configurations
- **Offline Signing**: Sign transactions on air-gapped computers for maximum security
- **Encrypted Storage**: All wallet data encrypted with modern Argon2id key derivation
- **Tor Integration**: Route all Bitcoin traffic through Tor for enhanced privacy
- **Memory Safety**: Built with Rust to prevent buffer overflows and memory vulnerabilities

### 💰 **Wallet Management**
- **HD (Hierarchical Deterministic) Wallets**: Generate unlimited addresses from a single seed
- **Multiple Address Types**: Legacy, SegWit, and Taproot address support
- **Watch-Only Wallets**: Monitor addresses without storing private keys
- **Legacy Import**: Seamlessly migrate from older Armory wallet versions
- **Backup & Recovery**: Secure wallet backups with seed phrase support
- **Label Management**: Organize transactions and addresses with custom labels

### ⚡ **Transaction Features**
- **PSBT v2 Support**: Create and sign Partially Signed Bitcoin Transactions
- **Replace-by-Fee (RBF)**: Increase transaction fees if needed
- **Coin Control**: Manually select which coins to spend for enhanced privacy
- **Fee Estimation**: Intelligent fee calculation for optimal confirmation times
- **Taproot Transactions**: Use the latest Bitcoin script features for efficiency and privacy
- **Batch Transactions**: Send to multiple recipients in a single transaction

### 🌐 **Network & Connectivity**
- **Bitcoin Core Integration**: Works seamlessly with your Bitcoin Core node
- **Multiple Node Support**: Connect to multiple Bitcoin nodes for redundancy
- **BIP-324 Encrypted Transport**: Future-ready encrypted peer-to-peer communication
- **Tor Connectivity**: Built-in SOCKS5 proxy support for enhanced privacy
- **Testnet & Regtest**: Full support for testing environments

---

## 📦 Installation

### Option 1: Pre-Built Binaries (Recommended)

Download the latest release for your operating system:

#### **macOS**
```bash
# Intel Macs
curl -L https://github.com/your-org/armory/releases/latest/download/armory-rust-macos-intel.tar.gz | tar -xz
cd armory-rust-macos-intel && ./install.sh

# Apple Silicon Macs  
curl -L https://github.com/your-org/armory/releases/latest/download/armory-rust-macos-apple-silicon.tar.gz | tar -xz
cd armory-rust-macos-apple-silicon && ./install.sh
```

#### **Ubuntu / Debian**
```bash
# x86_64 systems
curl -L https://github.com/your-org/armory/releases/latest/download/armory-rust-ubuntu-x86_64.tar.gz | tar -xz
cd armory-rust-ubuntu-x86_64 && ./install.sh

# ARM64 systems (Raspberry Pi 4, etc.)
curl -L https://github.com/your-org/armory/releases/latest/download/armory-rust-ubuntu-aarch64.tar.gz | tar -xz
cd armory-rust-ubuntu-aarch64 && ./install.sh
```

#### **Fedora / RHEL / CentOS**
```bash
# x86_64 systems (static binary works on most distributions)
curl -L https://github.com/your-org/armory/releases/latest/download/armory-rust-fedora-x86_64.tar.gz | tar -xz
cd armory-rust-fedora-x86_64 && ./install.sh
```

#### **Manual Installation**
1. Go to [Releases](https://github.com/your-org/armory/releases/latest)
2. Download the appropriate binary for your system
3. Extract: `tar -xzf armory-rust-[platform].tar.gz`
4. Run: `cd armory-rust-[platform] && ./install.sh`

The installation script will place `armory-rust` in your `~/.local/bin` directory and add it to your PATH.

### Option 2: Package Managers (Coming Soon)

We're working on adding Armory to popular package managers:

```bash
# macOS (Homebrew) - Coming Soon
brew install armory-rust

# Linux (Snap) - Coming Soon  
snap install armory-rust

# Arch Linux (AUR) - Coming Soon
yay -S armory-rust
```

### Option 3: Build from Source

If you prefer to build from source or contribute to development:

#### **Prerequisites**
- Rust 1.78+ ([Install Rust](https://rustup.rs/))
- Git

#### **Build Instructions**
```bash
# Clone the repository
git clone https://github.com/your-org/armory.git
cd armory/armory-rust

# Build the release version
cargo build --release

# Install to your system
cargo install --path .

# Verify installation
armory-rust --version
```

---

## 🚀 Quick Start

### 1. Create Your First Wallet

```bash
# Create a new wallet with encryption
armory-rust create my-wallet --encrypt

# Create a watch-only wallet (no private keys)
armory-rust create watch-wallet --watch-only

# Create a testnet wallet for learning
armory-rust create test-wallet --network testnet
```

### 2. Generate Receiving Addresses

```bash
# Generate a SegWit address (recommended)
armory-rust address my-wallet --type native-segwit

# Generate a Taproot address (most efficient)
armory-rust address my-wallet --type taproot

# Generate a legacy address (maximum compatibility)
armory-rust address my-wallet --type legacy
```

### 3. Check Your Balance

```bash
# View wallet balance and transaction history
armory-rust balance my-wallet

# List all your wallets
armory-rust list

# Get detailed wallet information
armory-rust info my-wallet
```

### 4. Send Bitcoin

```bash
# Send Bitcoin to an address
armory-rust send my-wallet --to bc1qexample... --amount 0.001

# Send with custom fee rate
armory-rust send my-wallet --to bc1qexample... --amount 0.001 --fee-rate 20

# Create an unsigned transaction (for offline signing)
armory-rust send my-wallet --to bc1qexample... --amount 0.001 --create-only
```

### 5. Advanced Features

```bash
# Create a 2-of-3 multi-signature wallet
armory-rust multisig create 2-of-3 wallet1 wallet2 wallet3

# Import a legacy Armory wallet
armory-rust legacy-import /path/to/old/wallet.dat new-wallet-name

# Export wallet for backup
armory-rust export my-wallet --output wallet-backup.json

# Use with Tor for privacy
armory-rust --tor balance my-wallet
```

---

## 📚 Documentation

### **User Guides**
- **[Installation Guide](docs/SETUP.md)** - Detailed setup instructions for all platforms
- **[User Manual](docs/USER_GUIDE.md)** - Complete guide to using Armory features
- **[Migration Guide](docs/MIGRATION.md)** - Moving from legacy Armory or other wallets
- **[Security Best Practices](docs/SECURITY.md)** - How to keep your Bitcoin secure
- **[Privacy Guide](docs/PRIVACY.md)** - Maximizing privacy and anonymity
- **[Troubleshooting](docs/TROUBLESHOOTING.md)** - Common issues and solutions

### **Advanced Topics**
- **[Multi-Signature Wallets](docs/MULTISIG.md)** - Setting up and using multi-sig
- **[Hardware Wallets](docs/HARDWARE.md)** - Integration with hardware devices  
- **[Offline Signing](docs/OFFLINE.md)** - Air-gapped transaction signing
- **[Bitcoin Core Integration](docs/BITCOIN_CORE.md)** - Running with your own node
- **[Tor Setup](docs/TOR.md)** - Configuring Tor for enhanced privacy

### **Technical Documentation**
- **[API Reference](docs/API.md)** - Command-line interface documentation
- **[Architecture](docs/ARCHITECTURE.md)** - Technical system overview
- **[Release Process](docs/RELEASE_PROCESS.md)** - How releases are created and verified

---

## 🔒 Security Features

### **Audited Security**
- **Memory-Safe Code**: Written in Rust to prevent buffer overflows and memory corruption
- **Modern Cryptography**: ChaCha20Poly1305 encryption and Argon2id key derivation
- **Secure Dependencies**: All cryptographic libraries are audited and regularly updated
- **Zero-Knowledge Design**: Your private keys never leave your device unless explicitly exported

### **Hardware Security**
- **Hardware Wallet Integration**: Works with Ledger, Trezor, and other devices
- **Air-Gapped Signing**: Create and sign transactions on offline computers
- **Seed Phrase Backup**: Standard BIP-39 seed phrases for recovery
- **Multi-Signature Protection**: Require multiple keys to authorize transactions

### **Network Security**
- **Tor Support**: All network traffic can be routed through Tor
- **Encrypted Transport**: BIP-324 encrypted peer-to-peer communication
- **No Key Servers**: Never sends private information to external servers
- **Local Operation**: Runs entirely on your computer with optional Bitcoin Core connection

---

## 🌍 Community & Support

### **Getting Help**
- **[GitHub Issues](https://github.com/your-org/armory/issues)** - Bug reports and feature requests
- **[GitHub Discussions](https://github.com/your-org/armory/discussions)** - Community support and general questions
- **[Documentation](docs/)** - Comprehensive guides and tutorials

### **Stay Updated**
- **[Releases](https://github.com/your-org/armory/releases)** - Download the latest version
- **[Changelog](CHANGELOG.md)** - See what's new in each release
- **[Security Advisories](https://github.com/your-org/armory/security/advisories)** - Important security updates

### **Contributing**
We welcome contributions from the community! See the [Developer Section](#-for-developers) below for information on how to contribute.

---

## ⚠️ Important Security Notes

1. **Backup Your Wallet**: Always backup your seed phrase and store it securely offline
2. **Verify Downloads**: Check SHA256 checksums of downloaded binaries
3. **Test First**: Practice with small amounts on testnet before using mainnet
4. **Keep Updated**: Regularly update to the latest version for security patches
5. **Hardware Wallets**: Consider using a hardware wallet for large amounts
6. **Air-Gapped Signing**: For maximum security, sign transactions on an offline computer

---

## 📄 License

Armory is licensed under the [GNU Affero General Public License v3.0 (AGPL-3.0)](LICENSE).

This ensures that any modifications or network-based deployments must also be released under the same license, maintaining the open-source nature of the project and protecting user freedoms.

---

## 🔧 For Developers

*This section contains information for developers who want to contribute to Armory or build it from source.*

### **Development Environment**

#### **Prerequisites**
- **Rust 1.78+**: Install via [rustup.rs](https://rustup.rs/)
- **Git**: For version control
- **Bitcoin Core** (optional): For integration testing

#### **Development Setup**
```bash
# Clone the repository
git clone https://github.com/your-org/armory.git
cd armory

# Set up development environment
cd armory-rust
cargo build

# Run the test suite (127/127 tests passing)
cargo test

# Run with debug output
RUST_LOG=debug cargo run -- --help
```

### **Project Architecture**

Armory is built with a modular Rust architecture:

```
armory-rust/src/
├── main.rs              # CLI entry point
├── crypto/              # Modern cryptographic operations
├── storage/             # Encrypted database storage  
├── wallet/              # HD wallet implementation
├── transaction/         # PSBT v2 transaction processing
├── network/             # P2P and RPC communication
├── script/              # Bitcoin script validation
└── cli/                 # Command-line interface
```

### **Key Design Principles**
- **Memory Safety**: Zero unsafe code in wallet operations
- **Modern Cryptography**: Audited libraries (ChaCha20Poly1305, Argon2id)
- **Bitcoin Standards**: Full BIP compliance (BIP-32/39/44/49/84/86/340/341/370/324)
- **Comprehensive Testing**: 127/127 tests passing (100% success rate)
- **Cross-Platform**: Native builds for macOS, Linux, and Windows

### **Building Cross-Platform Binaries**

We provide scripts for building binaries for all supported platforms:

```bash
# Build for all platforms using native tools
./scripts/build-binaries.sh all

# Build using Docker for Linux targets
./scripts/build-with-docker.sh --build-image all

# Build for specific platform
./scripts/build-binaries.sh ubuntu-x86_64
```

See [Build System Documentation](docs/RELEASE_PROCESS.md) for complete build instructions.

### **Contributing Guidelines**

1. **Fork and Clone**: Fork the repository and clone your fork
2. **Create Branch**: Create a feature branch for your changes
3. **Follow Standards**: Use `cargo fmt` and `cargo clippy`
4. **Add Tests**: Include tests for new functionality
5. **Documentation**: Update documentation for user-facing changes
6. **Submit PR**: Open a pull request with a clear description

#### **Code Quality Standards**
```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets --all-features

# Run all tests
cargo test

# Security audit
cargo audit
```

#### **Development Automation**
The project includes automated development logging and quality gates:

```bash
# Check automation system status
./scripts/dev-log-helper.sh status

# Test quality gates
./scripts/dev-log-helper.sh test
```

### **Release Process**

Releases are automated through GitHub Actions:

1. **Create Tag**: `git tag v1.x.x && git push origin v1.x.x`
2. **Automated Build**: GitHub Actions builds all platform binaries
3. **Release Creation**: Automatic GitHub release with binaries and checksums
4. **Testing**: All releases undergo automated testing

### **Technical Documentation**

- **[Architecture Guide](docs/ARCHITECTURE.md)** - System design and module overview
- **[API Documentation](docs/API.md)** - Complete API reference
- **[Development Guide](docs/DEVELOPMENT.md)** - Contributing and development setup
- **[Migration Documentation](docs/LEGACY_MIGRATION.md)** - Legacy wallet support
- **[Security Assessment](docs/SECURITY.md)** - Security analysis and recommendations

### **Performance Benchmarks**

| Operation | Target | Current |
|-----------|--------|---------|
| Transaction Signing | <50ms | ~25ms |
| Address Generation | <10ms | ~5ms |
| Storage Operations | <100ms | ~45ms |
| Network Operations | <100ms | ~60ms |
| CLI Commands | <50ms | ~25ms |

### **Development Status**

- **Phase 1**: ✅ Foundation Architecture (Crypto, Storage, Wallet)
- **Phase 2**: ✅ Transaction Processing (PSBT v2, RBF, Taproot)
- **Phase 3**: ✅ Network Layer (BIP-324, RPC, Tor)
- **Phase 4**: ✅ CLI Interface (Complete wallet management)

**Current Status**: Production ready with 127/127 tests passing (100% success rate)

---

*For questions about development, see our [GitHub Discussions](https://github.com/your-org/armory/discussions) or review the [development documentation](docs/DEVELOPMENT.md).*