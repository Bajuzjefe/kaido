use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "kaido",
    version,
    about = "Aiken smart contract generator — the OpenZeppelin Wizard for Cardano",
    long_about = "Generates security-focused, verification-ready Aiken smart contract templates."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a new Aiken smart contract project
    Generate {
        /// Contract template to use
        #[arg(short, long)]
        template: TemplateArg,

        /// Project namespace (e.g., "myorg")
        #[arg(short, long)]
        namespace: String,

        /// Project name (e.g., "my-token")
        #[arg(short = 'p', long)]
        project_name: String,

        /// Output directory (defaults to ./<project_name>)
        #[arg(short, long)]
        output: Option<String>,

        // --- Simple Mint options ---
        /// Token display name (for mint template)
        #[arg(long)]
        token_name: Option<String>,

        /// On-chain asset name (for mint template)
        #[arg(long)]
        asset_name: Option<String>,

        /// Enable time-lock on minting policy
        #[arg(long, default_value_t = false)]
        time_lock: bool,

        // --- Vesting options ---
        /// Allow owner to cancel vesting before lock period
        #[arg(long, default_value_t = false)]
        cancellable: bool,

        /// Allow partial claims with value preservation
        #[arg(long, default_value_t = false)]
        partial_claim: bool,

        // --- SDK ---
        /// Generate TypeScript SDK alongside Aiken code
        #[arg(long, default_value_t = false)]
        sdk: bool,

        // --- Custom template options ---
        /// Composable features (only with --template custom)
        /// e.g., "sig,timelock,datum-continuity"
        #[arg(long, value_delimiter = ',')]
        features: Vec<String>,

        /// Custom datum fields (only with --template custom)
        /// e.g., "owner:ByteArray,amount:Int,deadline:Int"
        #[arg(long)]
        datum: Option<String>,

        /// Custom redeemer actions (only with --template custom)
        /// e.g., "Claim,Cancel,Withdraw(amount:Int)"
        #[arg(long)]
        redeemer: Option<String>,

        /// Validator purpose: "spend" or "mint" (only with --template custom)
        #[arg(long, default_value = "spend")]
        purpose: String,

        // --- Verification ---
        /// Skip aiken build verification
        #[arg(long, default_value_t = false)]
        skip_verify: bool,
    },

    /// List all available templates
    List,

    /// Verify an existing generated project compiles
    Verify {
        /// Path to the Aiken project to verify
        #[arg(default_value = ".")]
        path: String,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum TemplateArg {
    /// CIP-25 minting policy with admin signature and optional time-lock
    Mint,
    /// Time-locked fund release with beneficiary claim
    Vesting,
    /// Two-party escrow with deadline and mutual cancellation
    Escrow,
    /// N-of-M multisig treasury with deposit/withdraw and datum continuity
    Treasury,
    /// NFT marketplace with list, buy, and delist
    Marketplace,
    /// Staking pool with deposit, withdraw, and admin rewards
    Staking,
    /// Oracle-gated settlement with deadline and buyer reclaim
    Oracle,
    /// On-chain referral system with mint, treasury, and anti-sybil
    Referral,
    /// DEX/AMM pool with constant-product swaps, liquidity, and fee management
    Dex,
    /// Lending pool with supply, borrow, repay, and collateral ratio enforcement
    Lending,
    /// DAO governance with token-gated treasury and proposal execution
    Governance,
    /// Streaming payments with time-based tranches and cancel/top-up
    Streaming,
    /// Custom validator with composable features
    Custom,
}
