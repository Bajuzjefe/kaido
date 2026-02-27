<p align="center">
  <strong>Kaido</strong><br/>
  Smart Contract Wizard for Cardano
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue.svg" alt="License" /></a>
  <img src="https://img.shields.io/badge/rust-%3E%3D1.70-orange.svg" alt="Rust" />
  <img src="https://img.shields.io/badge/aiken-v1.1.21-2EFFB5.svg" alt="Aiken" />
  <img src="https://img.shields.io/badge/templates-13-2EFFB5.svg" alt="Templates" />
  <img src="https://img.shields.io/badge/detectors-75-2EFFB5.svg" alt="Aikido Detectors" />
  <img src="https://img.shields.io/badge/Plutus-V3-blueviolet.svg" alt="Plutus V3" />
</p>

---

Generate security-focused Aiken smart contract starter templates.

**[Web Wizard](https://kaido2.vercel.app)** | **[Docs](https://kaido2.vercel.app/docs)** | **[CLI](#quick-start)**

```
kaido generate --template mint --namespace myorg --project-name my_token
```

By default (when `--skip-verify` is not set), Kaido verifies each generated contract with:
- Compiles with Aiken v1.1.21 (Plutus V3)
- Passes all inline tests (`aiken check`)
- Includes static analysis verification with [Aikido](https://github.com/jakubstefanik/aikido) (75 security detectors)

Important:
- Generated contracts are starter templates for development and testing.
- Run your own threat modeling, integration tests, and independent security review before mainnet deployment.

TypeScript SDK generation is currently supported for:
`mint`, `vesting`, `escrow`, `treasury`, `marketplace`, `staking`, `oracle`, `referral`.

---

## Why Kaido

Writing Cardano smart contracts is hard. Silent compiler failures, subtle UTxO vulnerabilities, and no standard patterns mean even experienced developers ship bugs. Kaido helps with **security-focused templates** that encode best-practice checks by construction:

- **Reference script injection protection** in all spend validators
- **Datum continuity validation** with proper field checks
- **Signature verification** using `extra_signatories`
- **Time-lock safety** with validity range checks
- **Value preservation** with lovelace tracking

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [Aiken](https://aiken-lang.org/installation-instructions) v1.1.21

### Install

```bash
git clone https://github.com/Bajuzjefe/kaido.git
cd kaido
cargo install --path crates/kaido-cli
```

### Generate Your First Contract

```bash
# Generate a minting policy
kaido generate --template mint --namespace myorg --project-name my_token

# Check what was generated
cd my_token
aiken check   # runs tests
aiken build   # compiles to plutus.json
```

Output:
```
my_token/
  aiken.toml
  lib/myorg/my_token/types.ak
  validators/my_token_mint.ak
```

### Or Use the Web Wizard

No installation needed. Visit **[kaido2.vercel.app](https://kaido2.vercel.app)** to configure and download contracts directly in your browser. Everything runs locally via WebAssembly.

---

## Templates

13 security-focused templates covering the most common Cardano contract patterns.

| Template | CLI Arg | Description |
|----------|---------|-------------|
| **Simple Mint** | `mint` | CIP-25 minting policy with admin signature |
| **Vesting** | `vesting` | Time-locked fund release with beneficiary claim |
| **Escrow** | `escrow` | Two-party escrow with deadline and cancellation |
| **Multisig Treasury** | `treasury` | N-of-M multisig with deposit, withdraw, 2 ADA floor |
| **NFT Marketplace** | `marketplace` | List, buy, and delist NFTs |
| **Staking Pool** | `staking` | Stake/unstake with admin rewards distribution |
| **Oracle Settlement** | `oracle` | Oracle-gated settlement with buyer reclaim |
| **Referral System** | `referral` | On-chain referral with mint, treasury, anti-sybil |
| **DEX Pool** | `dex` | Constant-product AMM with swaps, liquidity, fees |
| **Lending Pool** | `lending` | Supply, borrow, repay with collateral ratio enforcement |
| **Governance** | `governance` | DAO governance with token-gated treasury and proposals |
| **Streaming** | `streaming` | Streaming payments with time-based tranches |
| **Custom** | `custom` | Compose from individual security features |

---

## Usage

### `kaido generate`

Generate a new Aiken smart contract project.

```bash
kaido generate --template <TEMPLATE> --namespace <NS> --project-name <NAME> [OPTIONS]
```

**Required:**
| Flag | Description |
|------|-------------|
| `--template <T>` | Template to use (see table above) |
| `--namespace <NS>` | Project namespace, e.g. `myorg` |
| `--project-name <NAME>` | Project name, e.g. `my_token` |

**Optional:**
| Flag | Description |
|------|-------------|
| `-o, --output <DIR>` | Output directory (default: `./<project-name>`) |
| `--sdk` | Generate TypeScript SDK alongside Aiken code (supported templates only) |
| `--skip-verify` | Skip `aiken build` + `aiken check` + `aikido scan` |

**Template-specific:**
| Flag | Templates | Description |
|------|-----------|-------------|
| `--time-lock` | mint | Enable time-lock on minting policy |
| `--cancellable` | vesting | Allow owner to cancel before lock period |
| `--partial-claim` | vesting | Allow partial claims with datum continuity |
| `--purpose <P>` | custom | Validator purpose: `spend` or `mint` |
| `--features <F>` | custom | Comma-separated feature list |
| `--datum <D>` | custom | Datum fields (e.g. `admin:ByteArray,amount:Int`) |
| `--redeemer <R>` | custom | Redeemer actions (e.g. `Claim,Cancel(reason:ByteArray)`) |

### `kaido list`

List all available templates with descriptions.

### `kaido verify`

Verify an existing Aiken project compiles, passes tests, and survives static analysis.

```bash
kaido verify [PATH]
```

---

## Examples

```bash
# Time-locked mint
kaido generate --template mint --namespace myorg --project-name my_token --time-lock

# Vesting with cancellation and partial claims
kaido generate --template vesting --namespace myorg --project-name my_vesting \
  --cancellable --partial-claim

# Multisig treasury with SDK
kaido generate --template treasury --namespace myorg --project-name dao_treasury --sdk

# Custom validator with composable features
kaido generate --template custom --namespace myorg --project-name my_validator \
  --purpose spend \
  --features signature-auth,timelock,datum-continuity \
  --datum "admin:ByteArray,deadline:Int,amount:Int" \
  --redeemer "Claim,Cancel"

# Verify an existing Aiken project
kaido verify ./my-existing-project
```

---

## Composable Features

The `custom` template lets you compose validators from individual security features:

| Feature | Purpose | Description |
|---------|---------|-------------|
| `signature-auth` | any | Require specific signer in `extra_signatories` |
| `timelock` | spend | Enforce `validity_range` before/after deadline |
| `datum-continuity` | spend | Validate datum preservation across transactions |
| `value-preservation` | spend | Verify lovelace conservation (input >= output) |
| `reference-safety` | spend | Reject reference script injection |
| `burn-verification` | mint | Check all minted quantities are negative |
| `bounded-operations` | spend | Enforce minimum lovelace floor |

Features auto-resolve dependencies (e.g. `value-preservation` auto-includes `datum-continuity`).

---

## Verification Pipeline

Every `kaido generate` runs a 3-layer verification:

```
  1. aiken build     Compiler check (Plutus V3)
  2. aiken check     Inline test suite (4-8 tests per template)
  3. aikido scan     Static analysis (75 security detectors)
```

Skip with `--skip-verify` if you just want the source files.

---

## SDK Support Matrix

`--sdk` is available for these templates:

- `mint`
- `vesting`
- `escrow`
- `treasury`
- `marketplace`
- `staking`
- `oracle`
- `referral`

`--sdk` is not yet available for:

- `dex`
- `lending`
- `governance`
- `streaming`
- `custom`

---

## MCP Server

Kaido includes an MCP server for AI agent integration (Claude Code, etc.):

```bash
# Build the MCP server
cargo build --release -p kaido-mcp

# Configure in Claude Code
# Add to .claude/settings.json:
# { "mcpServers": { "kaido": { "command": "/path/to/kaido-mcp" } } }
```

**Tools:** `kaido_list_templates`, `kaido_generate`, `kaido_verify`

---

## Project Structure

```
kaido/
  Cargo.toml                    Workspace root
  crates/
    kaido-core/                 Library (WASM target)
      src/
        lib.rs                  Crate root
        error.rs                Error types
        wasm_api.rs             WASM bindings (behind feature flag)
        templates/mod.rs        Template enum + GenerateOptions
        features/               Composable feature system
        generator/render.rs     Tera rendering engine
    kaido-cli/                  Binary (CLI)
      src/
        main.rs                 CLI entry point
        cli.rs                  Command definitions (clap 4)
        verify.rs               Aiken + Aikido verification
        writer.rs               Filesystem writer
    kaido-mcp/                  Binary (MCP server)
      src/
        main.rs                 JSON-RPC stdio server
        tools.rs                Tool implementations
  templates/                    Tera template files (13 templates)
  web/                          React web wizard
    src/
      components/               UI components
      pages/                    Landing, Wizard, Docs
      lib/wasm.ts               WASM loader + fallbacks
```

---

## Development

```bash
# Run all tests
cargo test --workspace

# Build workspace
cargo build --workspace

# Build WASM module
wasm-pack build crates/kaido-core --target web --features wasm

# Run web wizard dev server
cd web && npm run dev

# Full web build (WASM + Vite)
cd web && npm run build:full
```

---

## Requirements

- **Rust** 1.70+ (2021 edition)
- **Aiken** v1.1.21 (for verification; generation works without it)
- **Aikido** (for static analysis in `generate`/`verify` unless `--skip-verify`)
- **Node.js** 22+ (for web wizard development)

## Disclaimer

Generated smart contracts are provided as **starting points and foundations** for dApp development, rapid prototyping, and testing. They are **not intended for direct deployment to mainnet** without independent review. Always conduct your own security audit, thorough testing, and formal verification before deploying any smart contract that handles real funds. Use at your own risk.

## License

Apache-2.0
