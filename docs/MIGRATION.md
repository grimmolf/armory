# Migration Guide

This guide walks you through migrating from the legacy Armory Bitcoin wallet to the modern Rust implementation.

## 🚨 Important Safety Notice

**BEFORE STARTING**: Always create complete backups of your existing wallets and verify your backup restoration process before proceeding with migration.

### Pre-Migration Checklist

- [ ] **Backup all wallet files** to multiple secure locations
- [ ] **Record all wallet passphrases** in a secure location
- [ ] **Export private keys** from critical addresses (optional but recommended)
- [ ] **Test backup restoration** on a non-production system
- [ ] **Verify Bitcoin Core synchronization** (if using local node)
- [ ] **Document wallet structure** (labels, accounts, watching-only addresses)

## 📋 Migration Overview

The Rust implementation can import the following legacy Armory wallet formats:

| Format | File Extension | Encryption | Status |
|--------|---------------|------------|---------|
| Standard Wallet | `.wallet` | Optional | ✅ Supported |
| Watching-Only | `.wallet` | Optional | ✅ Supported |
| Encrypted Wallet | `.wallet` | ROMIX KDF | ✅ Supported |
| Backup Files | `.backup` | Various | 🔄 Planned |
| Paper Backups | Manual | N/A | 🔄 Planned |

## 🛠️ Migration Process

### Step 1: Prepare Your Environment

1. **Install the Rust implementation:**
   ```bash
   cd armory-rust
   cargo build --release
   ```

2. **Verify installation:**
   ```bash
   cargo run -- --version
   ```

3. **Check legacy wallet files:**
   ```bash
   # List your legacy wallets (typical locations)
   ls ~/.armory/armory_*.wallet
   ls ~/Armory/armory_*.wallet
   ```

### Step 2: Analyze Legacy Wallets

Before importing, analyze your legacy wallets to understand their structure:

```bash
# Analyze wallet without importing
cargo run -- analyze legacy "path/to/your.wallet"
```

**Example output:**
```
Legacy Wallet Analysis
=====================
File: armory_3J2K4L5M_encrypt.wallet
Version: 1.35c
Network: Main Network
Encryption: Yes (ROMIX KDF)
Address Count: 127
Last Activity: 2023-12-15
Estimated Import Time: ~15 seconds

Address Types Found:
- Legacy (P2PKH): 127 addresses
- Multisig: 0 addresses
- Watching-only: 0 addresses

Comments/Labels: 23 addresses have labels
```

### Step 3: Import Legacy Wallets

#### Basic Import (Unencrypted)

```bash
# Import unencrypted legacy wallet
cargo run -- import legacy \
  --source "armory_ABC123_unencrypt.wallet" \
  --name "imported-main-wallet" \
  --network bitcoin
```

#### Encrypted Wallet Import

```bash
# Import encrypted legacy wallet
cargo run -- import legacy \
  --source "armory_ABC123_encrypt.wallet" \
  --name "imported-encrypted-wallet" \
  --network bitcoin \
  --decrypt
```

You'll be prompted for:
- Legacy wallet passphrase (for decryption)
- New wallet passphrase (for Rust implementation)

#### Advanced Import Options

```bash
# Import with specific options
cargo run -- import legacy \
  --source "armory_ABC123.wallet" \
  --name "my-imported-wallet" \
  --network bitcoin \
  --preserve-labels \
  --import-watching-only \
  --verify-addresses \
  --backup-original
```

**Options explained:**
- `--preserve-labels`: Import address labels and comments
- `--import-watching-only`: Include watching-only addresses
- `--verify-addresses`: Verify all addresses after import
- `--backup-original`: Create backup of original file

### Step 4: Verify Migration

After importing, verify that everything was migrated correctly:

1. **Check wallet creation:**
   ```bash
   cargo run -- list
   ```

2. **Verify address count:**
   ```bash
   cargo run -- info "imported-main-wallet"
   ```

3. **Check balance (if applicable):**
   ```bash
   cargo run -- balance "imported-main-wallet"
   ```

4. **Verify specific addresses:**
   ```bash
   # List first 10 addresses
   cargo run -- addresses "imported-main-wallet" --limit 10
   
   # Check specific address
   cargo run -- address-info "imported-main-wallet" "1YourBitcoinAddress..."
   ```

5. **Test address generation:**
   ```bash
   # Generate new address to ensure derivation works
   cargo run -- address new "imported-main-wallet" --type legacy
   ```

## 🔄 Feature Mapping

### Legacy vs Modern Features

| Legacy Feature | Modern Equivalent | Migration Notes |
|---------------|-------------------|-----------------|
| **Root Address** | First derived address | Automatically mapped |
| **Address Pool** | HD derivation | Expanded to include all types |
| **Comments/Labels** | Address metadata | Preserved during import |
| **Watching-Only** | Descriptor without private keys | Supported with limitations |
| **Multisig** | Descriptor-based multisig | Import + modern recreation |
| **Paper Backup** | BIP-39 mnemonic | New backup format recommended |
| **Fragment Backup** | Shamir's Secret Sharing | Not yet implemented |

### Address Type Conversion

Legacy Armory used primarily P2PKH addresses. The Rust implementation supports all modern types:

```bash
# After import, you can generate new address types
cargo run -- address new "imported-wallet" --type native-segwit  # bc1q...
cargo run -- address new "imported-wallet" --type taproot        # bc1p...
```

**Important**: Legacy addresses remain accessible, but new addresses use modern formats for better efficiency and privacy.

## 🔐 Security Considerations

### Passphrase Changes

The migration process allows you to change passphrases:

```bash
# During import, you can set a new passphrase
cargo run -- import legacy \
  --source "old.wallet" \
  --name "new-wallet" \
  --new-passphrase
```

### Encryption Upgrade

Legacy ROMIX KDF is replaced with Argon2id for better security:

- **Legacy**: Custom ROMIX with variable parameters
- **Modern**: Argon2id with memory-hard properties

### Key Derivation Changes

- **Legacy**: Custom deterministic key generation
- **Modern**: BIP-32 HD wallet standard

**Note**: Original private keys are preserved exactly, but new keys use BIP-32 derivation.

## 📁 File Structure Changes

### Legacy Directory Structure
```
~/.armory/
├── armory_ABC123_unencrypt.wallet
├── armory_DEF456_encrypt.wallet
├── ArmoryDB/
├── settings.txt
└── armorylog.txt
```

### Modern Directory Structure
```
~/.armory-rust/
├── config.toml
├── wallets.db/           # SLED database
├── backups/
│   ├── wallet_20240101_120000.backup
│   └── wallet_20240101_130000.backup
└── logs/
    └── armory.log
```

## 🐛 Troubleshooting

### Common Issues

#### "Wallet file not found or corrupted"

```bash
# Check file permissions
ls -la your.wallet

# Verify file is not corrupted
cargo run -- verify legacy "your.wallet"
```

#### "Invalid passphrase"

- Ensure you're using the correct passphrase
- Check for encoding issues (UTF-8 vs Latin-1)
- Try on a copy of the wallet file first

#### "Address count mismatch"

```bash
# Run import with verbose logging
RUST_LOG=debug cargo run -- import legacy "your.wallet" --name "test"

# Compare with legacy Armory address count
# Legacy: Tools → Address Book → Count
```

#### "Balance discrepancy"

- The Rust implementation may show different balances initially
- Ensure Bitcoin Core is fully synchronized
- Run wallet rescan if needed:
  ```bash
  cargo run -- rescan "imported-wallet"
  ```

### Legacy Wallet Issues

#### Corrupted wallet files

```bash
# Attempt repair (limited capability)
cargo run -- repair legacy "corrupted.wallet" --output "repaired.wallet"
```

#### Very old wallet versions

Wallets created before Armory 0.90 may need special handling:
```bash
# Import with compatibility mode
cargo run -- import legacy "old.wallet" --compatibility-mode --name "old-wallet"
```

## 📊 Migration Verification

### Post-Migration Checklist

After migration, verify the following:

- [ ] **All addresses imported correctly**
  ```bash
  cargo run -- addresses "wallet" --all | wc -l
  ```

- [ ] **Labels preserved**
  ```bash
  cargo run -- labels "wallet"
  ```

- [ ] **Balance matches** (if wallet has funds)
  ```bash
  cargo run -- balance "wallet" --confirmed
  ```

- [ ] **Can generate new addresses**
  ```bash
  cargo run -- address new "wallet" --type native-segwit
  ```

- [ ] **Private key access works**
  ```bash
  cargo run -- export-key "wallet" "address" --format wif
  ```

### Performance Comparison

| Operation | Legacy Armory | Rust Implementation |
|-----------|---------------|-------------------|
| Wallet startup | 10-30 seconds | <2 seconds |
| Address generation | 1-2 seconds | <100ms |
| Transaction signing | 2-5 seconds | <50ms |
| Balance calculation | 5-15 seconds | <1 second |

## 🔄 Rollback Plan

If you need to return to the legacy implementation:

1. **Legacy wallets are preserved** during import (unless `--delete-original` used)
2. **Original wallet files remain unchanged** during migration
3. **Can continue using legacy Armory** alongside Rust implementation
4. **Export private keys** from Rust implementation if needed:
   ```bash
   cargo run -- export "wallet" --format legacy-armory
   ```

## 🆘 Getting Help

If you encounter issues during migration:

1. **Check logs:**
   ```bash
   tail -f ~/.armory-rust/logs/armory.log
   ```

2. **Run with debug output:**
   ```bash
   RUST_LOG=debug cargo run -- import legacy "your.wallet"
   ```

3. **Verify wallet integrity:**
   ```bash
   cargo run -- verify "imported-wallet"
   ```

4. **Create GitHub issue** with:
   - Wallet file version (from analysis output)
   - Error messages
   - Import command used
   - System information

## 📝 Migration Report

After successful migration, generate a report:

```bash
cargo run -- migration-report "imported-wallet" --output "migration-report.txt"
```

This creates a detailed report including:
- Addresses imported vs addresses generated
- Label preservation status
- Security improvements applied
- Recommended next steps

## 🎯 Post-Migration Recommendations

1. **Create new backup using modern format:**
   ```bash
   cargo run -- backup "imported-wallet" --format encrypted
   ```

2. **Generate BIP-39 mnemonic backup:**
   ```bash
   cargo run -- mnemonic export "imported-wallet"
   ```

3. **Test transaction signing:**
   ```bash
   # Create a small test transaction (testnet recommended)
   cargo run -- send "imported-wallet" --to "address" --amount 1000 --network testnet
   ```

4. **Configure automatic backups:**
   ```bash
   cargo run -- config set auto-backup true
   ```

5. **Update any external tools** to use new wallet format

---

**⚠️ Remember**: Always keep your legacy wallet files as backup until you're completely satisfied with the migration results.