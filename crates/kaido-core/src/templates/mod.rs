use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::features::types::{DatumField, RedeemerAction};

/// Available contract templates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Template {
    SimpleMint,
    Vesting,
    Escrow,
    MultisigTreasury,
    NftMarketplace,
    StakingPool,
    OracleSettlement,
    ReferralSystem,
    DexPool,
    LendingPool,
    DaoGovernance,
    StreamingPayments,
    Custom,
}

impl Template {
    /// All available templates
    pub fn all() -> &'static [Template] {
        &[
            Template::SimpleMint,
            Template::Vesting,
            Template::Escrow,
            Template::MultisigTreasury,
            Template::NftMarketplace,
            Template::StakingPool,
            Template::OracleSettlement,
            Template::ReferralSystem,
            Template::DexPool,
            Template::LendingPool,
            Template::DaoGovernance,
            Template::StreamingPayments,
            Template::Custom,
        ]
    }

    /// Template slug for directory/file naming
    pub fn slug(&self) -> &'static str {
        match self {
            Template::SimpleMint => "simple_mint",
            Template::Vesting => "vesting",
            Template::Escrow => "escrow",
            Template::MultisigTreasury => "multisig_treasury",
            Template::NftMarketplace => "nft_marketplace",
            Template::StakingPool => "staking_pool",
            Template::OracleSettlement => "oracle_settlement",
            Template::ReferralSystem => "referral_system",
            Template::DexPool => "dex_pool",
            Template::LendingPool => "lending_pool",
            Template::DaoGovernance => "dao_governance",
            Template::StreamingPayments => "streaming_payments",
            Template::Custom => "custom",
        }
    }

    /// Human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Template::SimpleMint => {
                "CIP-25 minting policy with admin signature and optional time-lock"
            }
            Template::Vesting => {
                "Time-locked fund release with beneficiary claim and optional cancel"
            }
            Template::Escrow => {
                "Two-party escrow with deadline, completion, and mutual cancellation"
            }
            Template::MultisigTreasury => {
                "N-of-M multisig treasury with deposit, withdraw, datum continuity, and 2 ADA floor"
            }
            Template::NftMarketplace => "NFT marketplace with list, buy, and delist actions",
            Template::StakingPool => "Staking pool with deposit, withdraw, and admin rewards",
            Template::OracleSettlement => "Oracle-gated settlement with deadline and buyer reclaim",
            Template::ReferralSystem => {
                "On-chain referral system with mint, treasury, and anti-sybil protection"
            }
            Template::DexPool => {
                "DEX/AMM pool with constant-product swaps, liquidity, and fee management"
            }
            Template::LendingPool => {
                "Lending pool with supply, borrow, repay, and collateral ratio enforcement"
            }
            Template::DaoGovernance => {
                "DAO governance with token-gated treasury and proposal execution"
            }
            Template::StreamingPayments => {
                "Streaming payments with time-based tranches and cancel/top-up"
            }
            Template::Custom => {
                "Custom validator with composable features (sig, timelock, datum-continuity, ...)"
            }
        }
    }

    /// Whether this template currently has TypeScript SDK templates available.
    pub fn supports_sdk(&self) -> bool {
        matches!(
            self,
            Template::SimpleMint
                | Template::Vesting
                | Template::Escrow
                | Template::MultisigTreasury
                | Template::NftMarketplace
                | Template::StakingPool
                | Template::OracleSettlement
                | Template::ReferralSystem
        )
    }
}

impl FromStr for Template {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "simple_mint" | "simple-mint" | "mint" => Ok(Template::SimpleMint),
            "vesting" => Ok(Template::Vesting),
            "escrow" => Ok(Template::Escrow),
            "multisig_treasury" | "multisig-treasury" | "treasury" => {
                Ok(Template::MultisigTreasury)
            }
            "nft_marketplace" | "nft-marketplace" | "marketplace" => Ok(Template::NftMarketplace),
            "staking_pool" | "staking-pool" | "staking" => Ok(Template::StakingPool),
            "oracle_settlement" | "oracle-settlement" | "oracle" => Ok(Template::OracleSettlement),
            "referral_system" | "referral-system" | "referral" => Ok(Template::ReferralSystem),
            "dex_pool" | "dex-pool" | "dex" => Ok(Template::DexPool),
            "lending_pool" | "lending-pool" | "lending" => Ok(Template::LendingPool),
            "dao_governance" | "dao-governance" | "governance" => Ok(Template::DaoGovernance),
            "streaming_payments" | "streaming-payments" | "streaming" => {
                Ok(Template::StreamingPayments)
            }
            "custom" => Ok(Template::Custom),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.slug())
    }
}

/// Options for generating a contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateOptions {
    /// Template to use
    pub template: Template,
    /// Project namespace (e.g., "myorg")
    pub namespace: String,
    /// Project name (e.g., "my-token") — used in aiken.toml
    pub project_name: String,
    /// Module name (snake_case, e.g., "my_token") — used in lib/ paths and `use` imports
    pub module_name: String,
    /// Human-readable description
    pub description: String,
    /// Validator name (snake_case)
    pub validator_name: String,

    // --- Simple Mint options ---
    /// Token display name
    pub token_name: Option<String>,
    /// On-chain asset name
    pub asset_name: Option<String>,
    /// Enable time-lock on minting
    pub time_lock: bool,

    // --- Vesting options ---
    /// Allow owner to cancel before lock period
    pub cancellable: bool,
    /// Allow partial claims with value preservation
    pub partial_claim: bool,

    // --- Custom template options ---
    /// Validator purpose: "spend" or "mint"
    pub purpose: String,
    /// Custom datum field definitions
    pub datum_fields: Vec<DatumField>,
    /// Custom redeemer action definitions
    pub redeemer_actions: Vec<RedeemerAction>,
    /// Selected feature names (resolved)
    pub feature_names: Vec<String>,
}

impl GenerateOptions {
    /// Validate user-controlled naming inputs before they are used in paths/imports.
    pub fn validate_namespace_and_project(
        namespace: &str,
        project_name: &str,
    ) -> Result<(), String> {
        validate_name_part(namespace, "namespace")?;
        validate_name_part(project_name, "project_name")?;
        Ok(())
    }

    /// Create options for a simple mint template
    pub fn simple_mint(
        namespace: &str,
        project_name: &str,
        token_name: &str,
        asset_name: &str,
        time_lock: bool,
    ) -> Self {
        let module_name = Self::to_snake_case(project_name);
        Self {
            template: Template::SimpleMint,
            namespace: namespace.to_string(),
            project_name: project_name.to_string(),
            module_name: module_name.clone(),
            description: format!("{} minting policy", token_name),
            validator_name: format!("{}_mint", module_name),
            token_name: Some(token_name.to_string()),
            asset_name: Some(asset_name.to_string()),
            time_lock,
            cancellable: false,
            partial_claim: false,
            purpose: "mint".to_string(),
            datum_fields: vec![],
            redeemer_actions: vec![],
            feature_names: vec![],
        }
    }

    /// Create options for a vesting template
    pub fn vesting(
        namespace: &str,
        project_name: &str,
        cancellable: bool,
        partial_claim: bool,
    ) -> Self {
        let module_name = Self::to_snake_case(project_name);
        Self {
            template: Template::Vesting,
            namespace: namespace.to_string(),
            project_name: project_name.to_string(),
            module_name: module_name.clone(),
            description: format!("{} vesting contract", project_name),
            validator_name: format!("{}_vesting", module_name),
            token_name: None,
            asset_name: None,
            time_lock: false,
            cancellable,
            partial_claim,
            purpose: "spend".to_string(),
            datum_fields: vec![],
            redeemer_actions: vec![],
            feature_names: vec![],
        }
    }

    /// Create options for an escrow template
    pub fn escrow(namespace: &str, project_name: &str) -> Self {
        let module_name = Self::to_snake_case(project_name);
        Self {
            template: Template::Escrow,
            namespace: namespace.to_string(),
            project_name: project_name.to_string(),
            module_name: module_name.clone(),
            description: format!("{} escrow contract", project_name),
            validator_name: format!("{}_escrow", module_name),
            token_name: None,
            asset_name: None,
            time_lock: false,
            cancellable: false,
            partial_claim: false,
            purpose: "spend".to_string(),
            datum_fields: vec![],
            redeemer_actions: vec![],
            feature_names: vec![],
        }
    }

    /// Create options for a multisig treasury template
    pub fn multisig_treasury(namespace: &str, project_name: &str) -> Self {
        let module_name = Self::to_snake_case(project_name);
        Self {
            template: Template::MultisigTreasury,
            namespace: namespace.to_string(),
            project_name: project_name.to_string(),
            module_name: module_name.clone(),
            description: format!("{} multisig treasury", project_name),
            validator_name: format!("{}_treasury", module_name),
            token_name: None,
            asset_name: None,
            time_lock: false,
            cancellable: false,
            partial_claim: false,
            purpose: "spend".to_string(),
            datum_fields: vec![],
            redeemer_actions: vec![],
            feature_names: vec![],
        }
    }

    /// Create options for an oracle settlement template
    pub fn oracle_settlement(namespace: &str, project_name: &str) -> Self {
        let module_name = Self::to_snake_case(project_name);
        Self {
            template: Template::OracleSettlement,
            namespace: namespace.to_string(),
            project_name: project_name.to_string(),
            module_name: module_name.clone(),
            description: format!("{} oracle settlement", project_name),
            validator_name: format!("{}_settlement", module_name),
            token_name: None,
            asset_name: None,
            time_lock: false,
            cancellable: false,
            partial_claim: false,
            purpose: "spend".to_string(),
            datum_fields: vec![],
            redeemer_actions: vec![],
            feature_names: vec![],
        }
    }

    /// Create options for a staking pool template
    pub fn staking_pool(namespace: &str, project_name: &str) -> Self {
        let module_name = Self::to_snake_case(project_name);
        Self {
            template: Template::StakingPool,
            namespace: namespace.to_string(),
            project_name: project_name.to_string(),
            module_name: module_name.clone(),
            description: format!("{} staking pool", project_name),
            validator_name: format!("{}_pool", module_name),
            token_name: None,
            asset_name: None,
            time_lock: false,
            cancellable: false,
            partial_claim: false,
            purpose: "spend".to_string(),
            datum_fields: vec![],
            redeemer_actions: vec![],
            feature_names: vec![],
        }
    }

    /// Create options for an NFT marketplace template
    pub fn nft_marketplace(namespace: &str, project_name: &str) -> Self {
        let module_name = Self::to_snake_case(project_name);
        Self {
            template: Template::NftMarketplace,
            namespace: namespace.to_string(),
            project_name: project_name.to_string(),
            module_name: module_name.clone(),
            description: format!("{} NFT marketplace", project_name),
            validator_name: format!("{}_marketplace", module_name),
            token_name: None,
            asset_name: None,
            time_lock: false,
            cancellable: false,
            partial_claim: false,
            purpose: "spend".to_string(),
            datum_fields: vec![],
            redeemer_actions: vec![],
            feature_names: vec![],
        }
    }

    /// Create options for a referral system template
    pub fn referral_system(namespace: &str, project_name: &str) -> Self {
        let module_name = Self::to_snake_case(project_name);
        Self {
            template: Template::ReferralSystem,
            namespace: namespace.to_string(),
            project_name: project_name.to_string(),
            module_name: module_name.clone(),
            description: format!("{} referral system", project_name),
            validator_name: format!("{}_referral", module_name),
            token_name: None,
            asset_name: None,
            time_lock: false,
            cancellable: false,
            partial_claim: false,
            purpose: "spend".to_string(),
            datum_fields: vec![],
            redeemer_actions: vec![],
            feature_names: vec![],
        }
    }

    /// Create options for a DEX/AMM pool template
    pub fn dex_pool(namespace: &str, project_name: &str) -> Self {
        let module_name = Self::to_snake_case(project_name);
        Self {
            template: Template::DexPool,
            namespace: namespace.to_string(),
            project_name: project_name.to_string(),
            module_name: module_name.clone(),
            description: format!("{} DEX pool", project_name),
            validator_name: format!("{}_pool", module_name),
            token_name: None,
            asset_name: None,
            time_lock: false,
            cancellable: false,
            partial_claim: false,
            purpose: "spend".to_string(),
            datum_fields: vec![],
            redeemer_actions: vec![],
            feature_names: vec![],
        }
    }

    /// Create options for a lending pool template
    pub fn lending_pool(namespace: &str, project_name: &str) -> Self {
        let module_name = Self::to_snake_case(project_name);
        Self {
            template: Template::LendingPool,
            namespace: namespace.to_string(),
            project_name: project_name.to_string(),
            module_name: module_name.clone(),
            description: format!("{} lending pool", project_name),
            validator_name: format!("{}_lending", module_name),
            token_name: None,
            asset_name: None,
            time_lock: false,
            cancellable: false,
            partial_claim: false,
            purpose: "spend".to_string(),
            datum_fields: vec![],
            redeemer_actions: vec![],
            feature_names: vec![],
        }
    }

    /// Create options for a DAO governance template
    pub fn dao_governance(namespace: &str, project_name: &str) -> Self {
        let module_name = Self::to_snake_case(project_name);
        Self {
            template: Template::DaoGovernance,
            namespace: namespace.to_string(),
            project_name: project_name.to_string(),
            module_name: module_name.clone(),
            description: format!("{} DAO governance", project_name),
            validator_name: format!("{}_governance", module_name),
            token_name: None,
            asset_name: None,
            time_lock: false,
            cancellable: false,
            partial_claim: false,
            purpose: "spend".to_string(),
            datum_fields: vec![],
            redeemer_actions: vec![],
            feature_names: vec![],
        }
    }

    /// Create options for a streaming payments template
    pub fn streaming_payments(namespace: &str, project_name: &str) -> Self {
        let module_name = Self::to_snake_case(project_name);
        Self {
            template: Template::StreamingPayments,
            namespace: namespace.to_string(),
            project_name: project_name.to_string(),
            module_name: module_name.clone(),
            description: format!("{} streaming payments", project_name),
            validator_name: format!("{}_stream", module_name),
            token_name: None,
            asset_name: None,
            time_lock: false,
            cancellable: false,
            partial_claim: false,
            purpose: "spend".to_string(),
            datum_fields: vec![],
            redeemer_actions: vec![],
            feature_names: vec![],
        }
    }

    /// Create options for a custom composable template
    pub fn custom(
        namespace: &str,
        project_name: &str,
        purpose: &str,
        datum_fields: Vec<DatumField>,
        redeemer_actions: Vec<RedeemerAction>,
        feature_names: Vec<String>,
    ) -> Self {
        let module_name = Self::to_snake_case(project_name);
        let suffix = if purpose == "mint" {
            "mint"
        } else {
            "validator"
        };
        Self {
            template: Template::Custom,
            namespace: namespace.to_string(),
            project_name: project_name.to_string(),
            module_name: module_name.clone(),
            description: format!("{} custom {} validator", project_name, purpose),
            validator_name: format!("{}_{}", module_name, suffix),
            token_name: None,
            asset_name: None,
            time_lock: false,
            cancellable: false,
            partial_claim: false,
            purpose: purpose.to_string(),
            datum_fields,
            redeemer_actions,
            feature_names,
        }
    }

    fn to_snake_case(s: &str) -> String {
        s.replace('-', "_").to_lowercase()
    }
}

fn validate_name_part(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{field} cannot be empty"));
    }
    if value.contains('/') || value.contains('\\') || value.contains(':') || value.contains("..") {
        return Err(format!(
            "{field} contains invalid path characters; use letters, digits, '-' or '_' only"
        ));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(format!(
            "{field} contains invalid characters; use letters, digits, '-' or '_' only"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::GenerateOptions;

    #[test]
    fn validate_namespace_and_project_rejects_path_traversal() {
        let err =
            GenerateOptions::validate_namespace_and_project("../x", "safe").expect_err("must fail");
        assert!(err.contains("invalid path characters"));
    }

    #[test]
    fn validate_namespace_and_project_accepts_safe_values() {
        assert!(GenerateOptions::validate_namespace_and_project("my_org", "my-project").is_ok());
    }
}
