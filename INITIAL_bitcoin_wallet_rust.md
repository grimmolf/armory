
# INITIAL.md — Bitcoin Wallet Rust Refactor & Modernisation 🚀

> **Purpose**  
> Compile this INITIAL spec into a PRP that drives multi‑agent LLM work.  
> Deliverables:  
> 1. **MODERNIZATION_ANALYSIS.md** – comprehensive analysis of legacy codebase and modernization requirements.  
> 2. **REQUIREMENTS.md** – exhaustive functional & non‑functional spec reflecting 2025 Bitcoin standards.  
> 3. **RUST_REFACTORING.md** – module‑by‑module porting guidance & modernisation notes.  
> 4. **SECURITY_AUDIT.md** – list of legacy cryptographic practices and updated recommendations (BIP‑324, BIP‑340, PSBT v2, etc.).

---

## 📌 Project Metadata
| Field               | Value |
|---------------------|-------|
| **Repo Root**       | `<repo‑root>` |
| **Legacy Language** | C++ / Python blend (11 y old) |
| **Target Language** | Rust 1.78 (edition 2024) |
| **Domain**          | Bitcoin wallet w/ advanced functions (multi‑sig, PSBT, coin‑control) |
| **Supported OS**    | macOS (arm64/x86_64), Fedora Linux (x86_64) |
| **Foundation Doc**  | MODERNIZATION_ANALYSIS.md (legacy codebase analysis) |

> Replace `<repo‑root>` with the absolute path before running `/generate-prp`.

---

## 🎯 High‑Level Goal
Modernise and port the 11‑year‑old wallet to Rust, ensuring full feature parity **and** alignment with current Bitcoin protocol improvements (SegWit, Taproot, PSBT v2, BIP‑324 encrypted transport). See MODERNIZATION_ANALYSIS.md for detailed legacy codebase assessment.

---

## Context
* Original code predates SegWit (BIP‑141) and Schnorr/Taproot (BIP‑340/341).  
* Uses legacy OpenSSL ECDSA & raw JSON‑RPC calls.  
* UX is desktop CLI + minimal Qt GUI.  
* Modern wallet expectations: descriptor‑based, hardware‑wallet support, Tor V3, partially signed tx workflow.

---

## 🧩 Sub‑Agent Strategy
| Agent ID | Role | MCP Model | Input Scope | Output |
|----------|------|-----------|-------------|--------|
| `planning-agent-#` | Break refactor into epics/tasks | **OpenAI MCP · o3** | high‑level goals + MODERNIZATION_ANALYSIS.md | `plan_#.json` |
| `analysis-agent-#` | Parse legacy source, extract APIs, crypto, BIPs used | **Gemini MCP · 2.5 Pro** | ≤300 LOC slices + modernization notes | `analysis_#.json` |
| `sec-agent` | Identify outdated cryptography, recommend modern libs (rust-bitcoin, miniscript, bip324 crates) | Gemini 2.5 Pro | all code + security findings | `security.json` |
| `doc-agent` | Build REQUIREMENTS.md & SECURITY_AUDIT.md | skeleton + JSON + MODERNIZATION_ANALYSIS.md | final docs |
| `refactor-agent` | Draft RUST_REFACTORING.md | all summaries + modernization analysis | final doc |

Claude orchestrates: spawns agents, merges, iterates.

---

## ✔️ Allowed Actions
✓ OpenAI MCP (o3) in planning agents  
✓ Gemini 2.5 Pro in analysis / security agents  
✓ Executing shell/Cargo commands for build/tests  
✓ Installing Rust toolchain (`rustup`), Bitcoin libs, etc.  

## ❌ Forbidden Actions
✗ Hallucinating BIP numbers or crate APIs—verify or mark `TODO:`  
✗ Pushing to protected branches without confirmation.

---

## Deliverable Templates

### REQUIREMENTS.md
```
# Functional Requirements
## Wallet Operations
| Feature | Legacy impl | New spec | Comments |
|---------|-------------|----------|----------|
| Key gen | ... | BIP‑32 HD, Taproot descriptors | ... |
| ...

# References
- MODERNIZATION_ANALYSIS.md: Legacy feature inventory
- Security findings from codebase analysis

# Non‑Functional
- Cross‑platform build via `cargo build` (macOS/Fedora)
- Security: hardware wallet interface (HWI), Tor V3, BIP‑324
- Performance targets: <50 ms signing latency
```

### RUST_REFACTORING.md
```
# Module Refactor Plan
## src/wallet.cpp → src/wallet.rs
- Replace OpenSSL ECDSA → `secp256k1-zkp`
- Use `bip39` crate for mnemonics
- ...

# References
- MODERNIZATION_ANALYSIS.md: Detailed module breakdown and dependency analysis

# System‑Level
- Use `cargo-feature` flags for gui/cli
- Async RPC via `reqwest`
```

### SECURITY_AUDIT.md
```
# Legacy Issues
- Raw JSON‑RPC over http → Upgrade to BIP‑324 noise encrypted transport
- ECDSA only → Add Schnorr signatures (BIP‑340)
...

# Foundation Analysis
- MODERNIZATION_ANALYSIS.md: Comprehensive security vulnerability assessment

# Recommended Rust crates
| Concern | Crate | Reason |
|---------|-------|--------|
| PSBT v2 | `psbt` | Maintained, no unsafe |
```

---

## Success Criteria
* REQUIREMENTS.md covers 100 % legacy features + new BIPs.  
* RUST_REFACTORING.md lists crate choices & build tags for each module.  
* SECURITY_AUDIT.md flags all deprecated crypto & network practices.  
* All docs cite file names & line numbers where feasible; unknowns tagged `TODO:`.

---

## Milestones
| # | Deliverable | Owner | ETA |
|---|-------------|-------|-----|
| 0 | MODERNIZATION_ANALYSIS.md (foundation) | ✅ COMPLETED | baseline |
| 1 | Dependency matrix & security scan | dep-agent, sec-agent | 3 h |
| 2 | Draft REQUIREMENTS.md | doc-agent | 6 h |
| 3 | Draft RUST_REFACTORING.md | refactor-agent | 8 h |
| 4 | SECURITY_AUDIT.md | sec-agent | 9 h |
| 5 | Integrated review | operator | 12 h |

---

## Warnings
Double‑validate every BIP or crate reference.  
Mark speculative upgrades with `ASSUMPTION:`.  
Reference MODERNIZATION_ANALYSIS.md for legacy implementation details.

# END OF INITIAL.md
