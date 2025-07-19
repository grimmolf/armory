# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Armory is a full-featured Bitcoin client offering advanced wallet management capabilities. It relies on Bitcoin Core (formerly Bitcoin-Qt) for networking and blockchain validation while providing its own wallet management, multi-signature support, and offline transaction capabilities.

## Architecture

### Core Components

- **Python GUI Layer** (`ArmoryQt.py`) - Main PyQt4-based graphical interface
- **Armory Engine** (`armoryengine/`) - Core Bitcoin wallet and transaction logic written in Python
- **C++ Backend** (`cppForSwig/`) - High-performance blockchain data management and cryptographic operations
- **Daemon Interface** (`armoryd.py`) - JSON-RPC server for headless operation

### Key Directories

- `armoryengine/` - Core wallet engine (Python)
  - `PyBtcWallet.py` - Wallet management
  - `PyBtcAddress.py` - Address/key handling
  - `Transaction.py` - Transaction creation/parsing
  - `BDM.py` - Block Data Manager interface
- `cppForSwig/` - C++ backend with SWIG Python bindings
  - `BlockUtils.cpp/.h` - Blockchain data processing
  - `BtcWallet.cpp/.h` - Wallet operations
  - `EncryptionUtils.cpp/.h` - Cryptographic functions
- `ui/` - Additional UI components and dialogs
- `pytest/` - Python unit tests
- `cppForSwig/gtest/` - C++ unit tests

### Build System

The project uses a dual build system:
- **Make** for C++ components and SWIG bindings
- **Python setup.py** for packaging (py2exe for Windows)

## Development Commands

### Building

```bash
# Build entire project (C++ backend + Python bindings)
make

# Build only C++ tests
make all-test-tools

# Clean build artifacts
make clean

# macOS specific build
make osx
```

### Testing

```bash
# Run all tests
make test

# Run C++ tests only
make gtest

# Run Python tests only  
make pytest
# OR
python -m unittest discover
```

### C++ Development

```bash
# Build C++ shared library and SWIG bindings
cd cppForSwig && make

# Run specific C++ tests
cd cppForSwig/gtest && ./CppBlockUtilsTests
```

## Dependencies

### Required
- Python 2.6/2.7 with development headers
- PyQt4 for GUI
- Twisted for asynchronous networking
- SWIG for Python/C++ bindings
- Crypto++ for cryptographic operations
- GCC/G++ compiler

### Platform-Specific
- **Linux**: Standard package manager installations
- **Windows**: Visual Studio build environment, py2exe for packaging
- **macOS**: Xcode tools, special deployment scripts in `osxbuild/`

## Key Development Notes

### C++ Backend Integration
- Uses SWIG to generate Python bindings from C++ code
- Core blockchain operations are in C++ for performance
- Python code interfaces through `CppBlockUtils` module

### Blockchain Data Management
- Depends on Bitcoin Core for blockchain data and networking
- Uses LMDB for local blockchain indexing and wallet data storage
- Block Data Manager (BDM) coordinates between Bitcoin Core and Armory

### Multi-Language Codebase
- UI and wallet logic primarily in Python
- Performance-critical operations in C++
- Extensive use of SWIG for seamless integration

### Testing Strategy
- Separate test suites for Python (`pytest/`) and C++ (`cppForSwig/gtest/`)
- Both unit tests and integration tests included
- Test data includes sample blockchain files

### Offline/Online Architecture
- Designed for offline signing workflows
- Watching-only wallets for online monitoring
- Transaction building can be done offline, signed, then broadcast online

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