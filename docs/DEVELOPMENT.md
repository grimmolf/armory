# Development Guide

This guide covers everything you need to know to contribute to the Armory Rust modernization project.

## 🚀 Getting Started

### Prerequisites

- **Rust 1.78+**: Install via [rustup.rs](https://rustup.rs/)
- **Git**: For version control
- **Bitcoin Core** (optional): For testing with real Bitcoin network
- **IDE/Editor**: VS Code with rust-analyzer recommended

### Development Environment Setup

1. **Install Rust toolchain:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   rustup component add rustfmt clippy
   ```

2. **Clone the repository:**
   ```bash
   git clone https://github.com/your-org/armory-rust-modernization.git
   cd armory-rust-modernization
   ```

3. **Install development tools:**
   ```bash
   # Code coverage
   cargo install cargo-tarpaulin
   
   # Security auditing
   cargo install cargo-audit
   
   # Dependency analysis
   cargo install cargo-machete
   
   # Documentation generation
   cargo install cargo-doc
   
   # Benchmarking
   cargo install cargo-criterion
   ```

4. **Set up pre-commit hooks:**
   ```bash
   # Install pre-commit (requires Python)
   pip install pre-commit
   pre-commit install
   ```

### First Build

```bash
cd armory-rust
cargo build
cargo test
cargo clippy
```

If everything passes, you're ready to develop!

## 🏗️ Project Structure

```
armory-rust/
├── Cargo.toml              # Project dependencies and metadata
├── src/
│   ├── lib.rs             # Public library API
│   ├── main.rs            # CLI application entry point
│   ├── error.rs           # Error types and handling
│   ├── crypto/            # Cryptographic operations
│   │   ├── mod.rs         # Module exports
│   │   ├── kdf.rs         # Key derivation functions
│   │   ├── encryption.rs  # Symmetric encryption
│   │   ├── signatures.rs  # Digital signatures
│   │   └── random.rs      # Secure random generation
│   ├── storage/           # Data persistence
│   │   ├── mod.rs
│   │   ├── wallet_storage.rs  # Encrypted storage engine
│   │   └── legacy_import.rs   # Legacy wallet import
│   ├── wallet/            # Wallet management
│   │   ├── mod.rs
│   │   ├── descriptor_wallet.rs  # Modern HD wallets
│   │   └── hd_wallet.rs   # BIP-32 key derivation
│   ├── transaction/       # Transaction processing
│   │   ├── mod.rs
│   │   ├── builder.rs     # Transaction construction
│   │   └── psbt.rs        # PSBT implementation
│   ├── network/           # Network communication
│   │   ├── mod.rs
│   │   ├── p2p.rs         # Bitcoin P2P protocol
│   │   └── rpc.rs         # JSON-RPC client
│   └── cli/               # Command-line interface
│       ├── mod.rs
│       ├── commands.rs    # CLI command definitions
│       └── config.rs      # Configuration management
├── tests/                 # Integration tests
├── benches/              # Performance benchmarks
├── examples/             # Usage examples
└── docs/                 # Additional documentation
```

## 🔧 Development Workflow

### 1. Choose an Issue

- Browse [open issues](https://github.com/your-org/armory-rust-modernization/issues)
- Look for issues labeled `good first issue` or `help wanted`
- Check the [project roadmap](../README.md#roadmap) for planned features

### 2. Create Feature Branch

```bash
git checkout main
git pull origin main
git checkout -b feature/your-feature-name
```

**Branch naming conventions:**
- `feature/description` - New features
- `fix/description` - Bug fixes  
- `refactor/description` - Code refactoring
- `docs/description` - Documentation updates
- `test/description` - Test improvements

### 3. Make Changes

Follow the coding standards and best practices outlined below.

### 4. Test Your Changes

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test crypto::tests

# Run with coverage
cargo tarpaulin --out Html

# Lint code
cargo clippy -- -D warnings

# Format code
cargo fmt

# Security audit
cargo audit
```

### 5. Commit Changes

```bash
git add .
git commit -m "feat: add new address generation for Taproot

- Implement BIP-86 key derivation
- Add Taproot address generation tests
- Update wallet API documentation

Fixes #123"
```

**Commit message format:**
```
<type>(<scope>): <description>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting changes
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance

### 6. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Create a pull request on GitHub with:
- Clear description of changes
- Link to related issues
- Test results and coverage reports
- Screenshots (if applicable)

## 📋 Coding Standards

### Rust Style Guide

We follow the official [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/) with some additions:

#### Code Formatting

```bash
# Format all code
cargo fmt

# Check formatting without changing files
cargo fmt -- --check
```

Configuration in `rustfmt.toml`:
```toml
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
```

#### Naming Conventions

- **Functions and variables**: `snake_case`
- **Types and traits**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`

```rust
// Good
const MAX_BUFFER_SIZE: usize = 1024;
pub struct WalletStorage;
pub trait SignatureProvider;
pub fn generate_address() -> Result<Address, Error>;

// Bad
const maxBufferSize: usize = 1024;
pub struct wallet_storage;
pub trait signatureProvider;
pub fn GenerateAddress() -> Result<Address, Error>;
```

#### Error Handling

Always use `Result<T, E>` for fallible operations:

```rust
// Good
pub fn create_wallet(name: &str) -> WalletResult<Wallet> {
    validate_name(name)?;
    let storage = WalletStorage::new()?;
    Ok(Wallet::new(name, storage))
}

// Bad - using panic
pub fn create_wallet(name: &str) -> Wallet {
    assert!(!name.is_empty());
    let storage = WalletStorage::new().unwrap();
    Wallet::new(name, storage)
}
```

#### Documentation

Document all public APIs with rustdoc:

```rust
/// Creates a new wallet with the specified name and network.
///
/// # Arguments
///
/// * `name` - Unique identifier for the wallet
/// * `network` - Bitcoin network (mainnet, testnet, etc.)
/// * `storage` - Storage backend for wallet data
///
/// # Returns
///
/// A new `Wallet` instance on success, or a `WalletError` on failure.
///
/// # Examples
///
/// ```rust
/// use armory_rust::wallet::Wallet;
/// use armory_rust::Network;
///
/// let wallet = Wallet::create_new(
///     "my-wallet".to_string(),
///     Network::Bitcoin,
///     storage
/// )?;
/// ```
///
/// # Errors
///
/// Returns `WalletError::InvalidInput` if the name is empty or invalid.
/// Returns `WalletError::Storage` if the storage backend fails.
pub fn create_new(
    name: String,
    network: Network,
    storage: WalletStorage,
) -> WalletResult<Self> {
    // Implementation
}
```

### Security Coding Practices

#### Memory Safety

```rust
// Use zeroize for sensitive data
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(ZeroizeOnDrop)]
pub struct SecureKey {
    key: [u8; 32],
}

impl Drop for SecureKey {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}
```

#### Input Validation

```rust
pub fn derive_key(password: &str, salt: &[u8]) -> CryptoResult<Vec<u8>> {
    // Validate inputs
    if password.is_empty() {
        return Err(CryptoError::InvalidInput("Password cannot be empty".into()));
    }
    
    if salt.len() < 16 {
        return Err(CryptoError::InvalidInput("Salt must be at least 16 bytes".into()));
    }
    
    // Proceed with operation
    todo!()
}
```

#### Constant-Time Operations

```rust
// Use constant-time comparison for sensitive data
use subtle::ConstantTimeEq;

pub fn verify_password(input: &[u8], stored_hash: &[u8]) -> bool {
    input.ct_eq(stored_hash).into()
}
```

### Testing Standards

#### Unit Tests

Place unit tests in the same file as the code they test:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_address_generation() {
        let wallet = create_test_wallet();
        let address = wallet.get_new_address(AddressType::NativeSegwit).unwrap();
        
        assert!(address.to_string().starts_with("bc1q"));
        assert!(wallet.owns_address(&address).is_some());
    }
    
    #[test]
    fn test_invalid_input() {
        let result = derive_key("", &[]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CryptoError::InvalidInput(_)));
    }
}
```

#### Integration Tests

Place integration tests in the `tests/` directory:

```rust
// tests/wallet_integration.rs
use armory_rust::prelude::*;
use tempfile::tempdir;

#[tokio::test]
async fn test_full_wallet_workflow() {
    let temp_dir = tempdir().unwrap();
    let config = StorageConfig {
        storage_path: temp_dir.path().to_path_buf(),
        auto_backup: true,
        backup_count: 3,
    };
    
    let storage = WalletStorage::new(config).unwrap();
    let mut wallet = Wallet::create_new(
        "test-wallet".to_string(),
        Network::Regtest,
        storage
    ).unwrap();
    
    // Test address generation
    let address = wallet.get_new_address(AddressType::NativeSegwit).unwrap();
    assert!(address.to_string().starts_with("bcrt1q"));
    
    // Test balance (should be zero)
    assert_eq!(wallet.balance(), 0);
}
```

#### Property Tests

Use property testing for complex operations:

```rust
// In development - example structure
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_key_derivation_consistency(
        password in "[a-zA-Z0-9]{8,64}",
        salt in prop::collection::vec(any::<u8>(), 16..64)
    ) {
        let key1 = derive_key(&password, &salt).unwrap();
        let key2 = derive_key(&password, &salt).unwrap();
        prop_assert_eq!(key1, key2);
    }
}
```

### Performance Standards

#### Benchmarking

Create benchmarks for performance-critical code:

```rust
// benches/crypto_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use armory_rust::crypto::*;

fn benchmark_key_derivation(c: &mut Criterion) {
    c.bench_function("key_derivation", |b| {
        let password = "test_password";
        let salt = [0u8; 32];
        let params = KdfParams::default();
        
        b.iter(|| {
            derive_key_from_password(
                black_box(password),
                black_box(&salt),
                black_box(&params)
            ).unwrap()
        })
    });
}

criterion_group!(benches, benchmark_key_derivation);
criterion_main!(benches);
```

Run benchmarks:
```bash
cargo bench
```

#### Performance Targets

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Key derivation | <10ms | `cargo bench key_derivation` |
| Address generation | <5ms | `cargo bench address_generation` |
| Transaction signing | <50ms | `cargo bench transaction_signing` |
| Storage operations | <100ms | `cargo bench storage_*` |

## 🧪 Testing

### Test Categories

1. **Unit Tests**: Individual function testing
2. **Integration Tests**: Cross-module interaction
3. **Property Tests**: Randomized input validation
4. **Performance Tests**: Benchmarking and profiling
5. **Security Tests**: Vulnerability scanning

### Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test crypto::tests

# Integration tests only
cargo test --test '*'

# With output
cargo test -- --nocapture

# Parallel execution control
cargo test -- --test-threads=1

# With coverage
cargo tarpaulin --out Html --output-dir coverage/
```

### Test Utilities

Common test utilities in `src/test_utils.rs`:

```rust
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use tempfile::TempDir;
    
    pub fn create_test_storage() -> (WalletStorage, TempDir) {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = StorageConfig {
            storage_path: temp_dir.path().to_path_buf(),
            auto_backup: false,
            backup_count: 0,
        };
        let storage = WalletStorage::new(config).unwrap();
        (storage, temp_dir)
    }
    
    pub fn create_test_wallet() -> Wallet {
        let (storage, _temp_dir) = create_test_storage();
        Wallet::create_new(
            "test-wallet".to_string(),
            Network::Regtest,
            storage
        ).unwrap()
    }
}
```

## 🔍 Debugging

### Logging

The project uses structured logging with `tracing`:

```rust
use tracing::{debug, info, warn, error, span, Level};

pub fn create_wallet(name: &str) -> WalletResult<Wallet> {
    let span = span!(Level::INFO, "create_wallet", wallet_name = name);
    let _enter = span.enter();
    
    info!("Creating new wallet");
    
    if name.is_empty() {
        warn!("Empty wallet name provided");
        return Err(WalletError::InvalidInput("Name cannot be empty".into()));
    }
    
    debug!("Initializing storage");
    let storage = WalletStorage::new()?;
    
    info!("Wallet created successfully");
    Ok(Wallet::new(name, storage))
}
```

### Debug Builds

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Module-specific logging
RUST_LOG=armory_rust::crypto=trace cargo run

# With backtrace
RUST_BACKTRACE=1 cargo run
```

### Common Debug Patterns

```rust
// Debug prints (remove before committing)
dbg!(&wallet_state);

// Conditional compilation for debug
#[cfg(debug_assertions)]
{
    println!("Debug info: {:?}", sensitive_data);
}

// Logging instead of println
debug!("Processing wallet: {}", wallet.id());
```

## 📦 Dependencies

### Adding Dependencies

1. **Research the crate:**
   - Check crates.io for popularity and maintenance
   - Review the source code on GitHub
   - Check for security audits

2. **Add to Cargo.toml:**
   ```toml
   [dependencies]
   new-crate = "1.0"
   ```

3. **Run security audit:**
   ```bash
   cargo audit
   ```

4. **Update documentation** if needed

### Dependency Guidelines

- **Prefer well-maintained crates** with active development
- **Use semantic versioning** constraints (`"1.0"` not `"*"`)
- **Minimize dependencies** to reduce attack surface
- **Pin specific versions** for security-critical dependencies
- **Regular updates** using `cargo update`

### Security Dependencies

Special care for cryptographic dependencies:

```toml
[dependencies]
# Cryptographic libraries - pin exact versions
chacha20poly1305 = "=0.10.1"
argon2 = "=0.5.0"
secp256k1 = "=0.29.1"

# Core functionality - use semver
bitcoin = "0.32"
serde = "1.0"
```

## 🚀 Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Incompatible API changes
- **MINOR**: Backward-compatible functionality additions
- **PATCH**: Backward-compatible bug fixes

### Release Checklist

1. **Update version in Cargo.toml**
2. **Update CHANGELOG.md**
3. **Run full test suite**
4. **Security audit**
5. **Generate documentation**
6. **Create release tag**
7. **Publish to crates.io** (when ready)

### Pre-release Testing

```bash
# Full test suite
cargo test --all-features

# Security audit
cargo audit

# Clippy with pedantic lints
cargo clippy -- -W clippy::pedantic

# Documentation check
cargo doc --all-features --no-deps

# Benchmark regression test
cargo bench --all-features
```

## 🤝 Contributing Guidelines

### Pull Request Process

1. **Fork the repository**
2. **Create feature branch** from `main`
3. **Make changes** following coding standards
4. **Add tests** for new functionality
5. **Update documentation** as needed
6. **Run full test suite**
7. **Submit pull request** with clear description

### PR Requirements

- [ ] **All tests pass** (`cargo test`)
- [ ] **Code is formatted** (`cargo fmt`)
- [ ] **No clippy warnings** (`cargo clippy`)
- [ ] **Documentation updated** (if applicable)
- [ ] **Security audit clean** (`cargo audit`)
- [ ] **Backward compatibility maintained** (for minor releases)

### Code Review Process

1. **Automated checks** run on all PRs
2. **Peer review** by project maintainers
3. **Security review** for crypto/storage changes
4. **Performance review** for critical path changes
5. **Documentation review** for API changes

## 📚 Resources

### Learning Rust

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings Exercises](https://github.com/rust-lang/rustlings)

### Bitcoin Development

- [Bitcoin Developer Guide](https://bitcoin.org/en/developer-guide)
- [BIP Repository](https://github.com/bitcoin/bips)
- [Bitcoin Core Source](https://github.com/bitcoin/bitcoin)

### Cryptography

- [Cryptography Engineering](https://www.schneier.com/books/cryptography_engineering/)
- [NaCl: Networking and Cryptography Library](https://nacl.cr.yp.to/)
- [OWASP Crypto Guidelines](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html)

### Project Resources

- [GitHub Repository](https://github.com/your-org/armory-rust-modernization)
- [Issue Tracker](https://github.com/your-org/armory-rust-modernization/issues)
- [Project Documentation](../README.md)
- [API Reference](API.md)

## 🆘 Getting Help

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For general questions and ideas
- **Discord/Slack**: [Join our community] (if available)
- **Email**: [maintainer-email] for security issues

Remember: we're here to help! Don't hesitate to ask questions or seek clarification on any aspect of the development process.