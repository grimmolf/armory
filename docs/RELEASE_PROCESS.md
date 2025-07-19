# Armory Rust Release Process

This document outlines the process for creating cross-platform releases of Armory Rust.

## Supported Platforms

### macOS
- **Intel (x86_64)**: macOS 10.12+ 
- **Apple Silicon (aarch64)**: macOS 11.0+

### Linux
- **Ubuntu/Debian x86_64**: Ubuntu 18.04+, Debian 10+
- **Ubuntu/Debian ARM64**: Ubuntu 20.04+, Debian 11+
- **Fedora/RHEL/CentOS x86_64**: Fedora 32+, RHEL 8+, CentOS 8+

## Local Development Builds

### Prerequisites

**All platforms:**
- Rust 1.78+ with Cargo
- Git

**Linux cross-compilation:**
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install gcc-aarch64-linux-gnu musl-tools

# Fedora/RHEL
sudo dnf install gcc-aarch64-linux-gnu musl-gcc
```

**macOS cross-compilation:**
- Xcode Command Line Tools
- Docker (for Linux target builds)

### Building Binaries

**Build all available platforms:**
```bash
./scripts/build-binaries.sh all
```

**Build specific platform:**
```bash
./scripts/build-binaries.sh macos-intel
./scripts/build-binaries.sh ubuntu-x86_64
./scripts/build-binaries.sh fedora-x86_64
```

**Clean build:**
```bash
./scripts/build-binaries.sh --clean all
```

**Check dependencies:**
```bash
./scripts/build-binaries.sh --check-deps
```

### Output

Binaries are created in `build-output/`:
```
build-output/
├── armory-rust-macos-intel.tar.gz
├── armory-rust-macos-apple-silicon.tar.gz
├── armory-rust-ubuntu-x86_64.tar.gz
├── armory-rust-ubuntu-aarch64.tar.gz
└── armory-rust-fedora-x86_64.tar.gz
```

Each archive contains:
- `armory-rust` - The binary executable
- `install.sh` - Installation script
- `README.md` - Project documentation
- `LICENSE` - License file
- `RUST_README.md` - Rust-specific documentation

## Automated GitHub Releases

### Triggering Releases

**Tagged release:**
```bash
git tag v1.0.0
git push origin v1.0.0
```

**Manual workflow dispatch:**
1. Go to GitHub Actions tab
2. Select "Cross-Platform Release Build"
3. Click "Run workflow"

### GitHub Actions Workflow

The workflow (`/.github/workflows/release.yml`) automatically:

1. **Builds for all platforms** using GitHub's hosted runners
2. **Cross-compiles** where supported
3. **Creates release archives** with documentation
4. **Uploads artifacts** for each platform
5. **Creates GitHub release** with all binaries attached

### Release Assets

Each release includes:
- `armory-rust-macos-intel.tar.gz` - macOS Intel binary
- `armory-rust-macos-apple-silicon.tar.gz` - macOS Apple Silicon binary  
- `armory-rust-ubuntu-x86_64.tar.gz` - Ubuntu/Debian x86_64 binary
- `armory-rust-ubuntu-aarch64.tar.gz` - Ubuntu/Debian ARM64 binary
- `armory-rust-fedora-x86_64.tar.gz` - Fedora/RHEL/CentOS x86_64 binary

## Installation Instructions

### For End Users

**1. Download the appropriate binary:**
```bash
# Example for Ubuntu x86_64
wget https://github.com/your-org/armory/releases/latest/download/armory-rust-ubuntu-x86_64.tar.gz
```

**2. Extract and install:**
```bash
tar -xzf armory-rust-ubuntu-x86_64.tar.gz
cd armory-rust-ubuntu-x86_64
./install.sh
```

**3. Add to PATH (if not already done):**
```bash
echo 'export PATH="$PATH:$HOME/.local/bin"' >> ~/.bashrc
source ~/.bashrc
```

**4. Verify installation:**
```bash
armory-rust --help
```

### Package Managers (Future)

We plan to support additional distribution methods:

- **Homebrew** (macOS): `brew install armory-rust`
- **Snap** (Linux): `snap install armory-rust`
- **Flatpak** (Linux): `flatpak install armory-rust`
- **AUR** (Arch Linux): `yay -S armory-rust`

## Release Checklist

### Pre-Release
- [ ] All tests pass: `cargo test` in `armory-rust/`
- [ ] Documentation updated
- [ ] Version bumped in `Cargo.toml`
- [ ] CHANGELOG.md updated
- [ ] Security audit completed if needed

### Local Testing
- [ ] Local build succeeds: `./scripts/build-binaries.sh all`
- [ ] All target platforms build successfully
- [ ] Installation scripts work on target platforms
- [ ] Basic functionality tested on each platform

### Release Process
- [ ] Create git tag: `git tag v1.x.x`
- [ ] Push tag: `git push origin v1.x.x`
- [ ] Verify GitHub Actions workflow completes
- [ ] Test download and installation from GitHub release
- [ ] Update release notes if needed

### Post-Release
- [ ] Announce release in relevant channels
- [ ] Update documentation website if applicable
- [ ] Monitor for any platform-specific issues

## Binary Verification

### Checksums

Each release includes SHA256 checksums for verification:

```bash
# Verify downloaded binary
sha256sum armory-rust-ubuntu-x86_64.tar.gz
```

### Code Signing (Future)

We plan to implement code signing for enhanced security:
- **macOS**: Apple Developer ID signing
- **Windows**: Authenticode signing
- **Linux**: GPG signing

## Troubleshooting

### Common Build Issues

**Cross-compilation linker errors:**
```bash
# Install cross-compilation tools
sudo apt-get install gcc-aarch64-linux-gnu musl-tools
```

**macOS signing issues:**
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine armory-rust
```

**Permission denied:**
```bash
# Make binary executable
chmod +x armory-rust
```

### Platform-Specific Notes

**macOS:**
- Binaries are not code-signed yet - users may need to allow in Security & Privacy
- Apple Silicon binaries won't run on Intel Macs and vice versa

**Linux:**
- musl builds (Fedora target) are statically linked and should work on most distributions
- glibc builds (Ubuntu target) require compatible glibc version

**General:**
- All binaries require glibc 2.27+ or musl
- Bitcoin Core compatibility: Works with Bitcoin Core 22.0+

## Performance Optimizations

Release binaries are built with:
- **LTO (Link Time Optimization)**: Enabled for smaller, faster binaries
- **Strip symbols**: Debug symbols removed for smaller size
- **Panic abort**: Reduces binary size
- **Optimization level 3**: Maximum performance optimization

Typical binary sizes:
- macOS: ~8-12MB
- Linux (glibc): ~6-10MB  
- Linux (musl): ~8-12MB

## Contact

For build issues or questions:
- GitHub Issues: [Project Issues](https://github.com/your-org/armory/issues)
- Development Documentation: `docs/DEVELOPMENT.md`