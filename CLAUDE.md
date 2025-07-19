# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Armory is a full-featured Bitcoin client offering advanced wallet management capabilities. It relies on Bitcoin Core (formerly Bitcoin-Qt) for networking and blockchain validation while providing its own wallet management, multi-signature support, and offline transaction capabilities.

## Architecture

**⚠️ IMPORTANT: This project has been completely modernized in Rust.**

The legacy Python/C++ implementation described below has been replaced with a modern Rust implementation located in `armory-rust/`. The information below is maintained for historical reference and migration context only.

### Modern Rust Implementation (Current)

- **Rust CLI Application** (`armory-rust/src/main.rs`) - Modern command-line interface
- **Rust Wallet Engine** (`armory-rust/src/wallet/`) - Memory-safe wallet operations
- **Modern Cryptography** (`armory-rust/src/crypto/`) - ChaCha20Poly1305, Argon2id, BIP-340
- **Network Layer** (`armory-rust/src/network/`) - BIP-324 encrypted transport, Bitcoin Core RPC

### Legacy Architecture (Historical Reference)

The following components were part of the original implementation and have been completely replaced:

- **~~Python GUI Layer~~** (~~`ArmoryQt.py`~~) - ⚠️ Removed - was PyQt4-based graphical interface
- **~~Armory Engine~~** (~~`armoryengine/`~~) - ⚠️ Removed - was Python wallet logic
- **~~C++ Backend~~** (~~`cppForSwig/`~~) - ⚠️ Removed - was C++/SWIG blockchain operations
- **~~Daemon Interface~~** (~~`armoryd.py`~~) - ⚠️ Removed - was JSON-RPC server

### Current Key Directories

- `armory-rust/` - **Modern Rust implementation (Active Development)**
  - `src/wallet/` - HD wallet with descriptor support
  - `src/crypto/` - Modern cryptographic operations
  - `src/transaction/` - PSBT v2 transaction processing
  - `src/network/` - BIP-324 and Bitcoin Core RPC
  - `src/cli/` - Command-line interface
  - `src/storage/` - Encrypted storage with legacy import
- `docs/` - Project documentation
- `PRPs/` - Project requirements and planning

### Build System

**Current**: The project now uses **Cargo** (Rust's native build system):
- **Rust Cargo** for all modern components in `armory-rust/`
- **Cross-platform support** with native Rust toolchain

**Legacy** (Removed): The original dual build system has been completely replaced:
- ~~**Make** for C++ components and SWIG bindings~~ - ⚠️ Removed
- ~~**Python setup.py** for packaging (py2exe for Windows)~~ - ⚠️ Removed

## Development Commands

### Building (Current Rust Implementation)

```bash
# Navigate to modern Rust implementation
cd armory-rust

# Build the Rust project
cargo build

# Build optimized release version
cargo build --release

# Install the CLI tool system-wide
cargo install --path .
```

### Testing (Current)

```bash
# Run all Rust tests (127/127 passing - 100%)
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test modules
cargo test crypto::tests
cargo test wallet::tests
cargo test cli::tests
```

### Legacy Commands (Historical Reference)

The following commands were used with the legacy implementation and are no longer applicable:

```bash
# ⚠️ REMOVED - Legacy build commands
# make                    # Built C++ backend + Python bindings
# make test              # Ran legacy test suite  
# make pytest            # Ran Python tests
# cd cppForSwig && make  # Built C++ shared library and SWIG bindings
```

## Dependencies

### Current (Rust Implementation)
- **Rust 1.78+** with Cargo package manager
- **Modern cryptographic libraries** via Cargo (ChaCha20Poly1305, Argon2id)
- **Bitcoin libraries** via Cargo (bitcoin, secp256k1, bdk_wallet)
- **Cross-platform support** - Windows, macOS, Linux

### Legacy (Historical Reference - Removed)
- ~~Python 2.6/2.7 with development headers~~ - ⚠️ Removed
- ~~PyQt4 for GUI~~ - ⚠️ Removed  
- ~~Twisted for asynchronous networking~~ - ⚠️ Removed
- ~~SWIG for Python/C++ bindings~~ - ⚠️ Removed
- ~~Crypto++ for cryptographic operations~~ - ⚠️ Removed
- ~~GCC/G++ compiler~~ - ⚠️ Removed

### Platform-Specific (Legacy - Removed)
- ~~**Linux**: Standard package manager installations~~ - ⚠️ Removed
- ~~**Windows**: Visual Studio build environment, py2exe for packaging~~ - ⚠️ Removed
- ~~**macOS**: Xcode tools, special deployment scripts in `osxbuild/`~~ - ⚠️ Removed

## Key Development Notes

### Modern Rust Architecture
- **Memory-safe**: Rust ownership system prevents common vulnerabilities
- **Modern cryptography**: ChaCha20Poly1305, Argon2id replacing legacy Crypto++
- **Bitcoin standards**: Full BIP-32/39/44/49/84/86/340/341/370/324 support
- **CLI-focused**: Command-line interface with production-ready functionality

### Current Implementation (Rust)
- **Single language**: Pure Rust implementation eliminates multi-language complexity
- **Modern dependencies**: Cargo-managed crates for Bitcoin protocol support
- **Comprehensive testing**: 127/127 tests passing (100% success rate)
- **Cross-platform**: Native Rust compilation for Windows, macOS, Linux

### Legacy Architecture (Historical Reference - Removed)
- ~~**C++ Backend Integration**: Used SWIG to generate Python bindings~~ - ⚠️ Removed
- ~~**Multi-Language Codebase**: UI in Python, performance operations in C++~~ - ⚠️ Removed
- ~~**LMDB Storage**: Local blockchain indexing and wallet data storage~~ - ⚠️ Replaced with SLED
- ~~**Separate test suites**: Python (`pytest/`) and C++ (`cppForSwig/gtest/`)~~ - ⚠️ Replaced with unified Rust tests

### Maintained Features
- **Offline/Online Architecture**: Still designed for offline signing workflows
- **Watching-only wallets**: Online monitoring capabilities maintained
- **Legacy wallet import**: Can import original Armory wallet files
- **Bitcoin Core integration**: RPC compatibility with Bitcoin Core nodes

## Development Logging

**CRITICAL:** All Claude Code development sessions MUST use the project development logging system.

### Development Log Location
- **Primary Log**: `docs/CLAUDE_DEVELOPMENT_LOG.md`
- **Format**: Structured entries with timestamp, branch, context, files modified, and implementation notes
- **Purpose**: Track architectural decisions, feature development, and maintain session continuity

### Logging Requirements
1. **Session Start**: Add entry with timestamp, branch, and planned work
2. **During Development**: Update entry with files modified and key changes
3. **Session End**: Finalize with implementation notes and cross-references
4. **Branch Tracking**: Create separate entries for each branch worked on

### Usage Guidelines
- **Always update the log** when starting development work
- **Include branch context** for multi-branch feature development  
- **Reference related entries** when building on previous work
- **Use consistent formatting** for searchability
- **Document rationale** for architectural and implementation decisions

This ensures continuity between Claude Code sessions and maintains a complete record of project evolution.