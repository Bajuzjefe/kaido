pub mod compose;
pub mod types;

use std::str::FromStr;

use crate::error::{KaidoError, Result};

/// Composable security features for custom validators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Feature {
    /// Require a specific signer in extra_signatories
    SignatureAuth,
    /// Enforce validity_range before/after a deadline field
    TimeLock,
    /// Find continuing output and validate datum preservation
    DatumContinuity,
    /// Verify lovelace math (input vs output) — requires DatumContinuity
    ValuePreservation,
    /// Reject reference script injection on continuing output — requires DatumContinuity
    ReferenceSafety,
    /// Check all minted quantities are negative (mint-purpose only)
    BurnVerification,
    /// Enforce a minimum lovelace floor on continuing output — requires DatumContinuity
    BoundedOperations,
}

impl FromStr for Feature {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "sig" | "signature" | "signature_auth" => Ok(Feature::SignatureAuth),
            "timelock" | "time_lock" => Ok(Feature::TimeLock),
            "datum_continuity" | "datum" | "continuity" => Ok(Feature::DatumContinuity),
            "value_preservation" | "value" | "preservation" => Ok(Feature::ValuePreservation),
            "reference_safety" | "ref_safety" | "refsafety" => Ok(Feature::ReferenceSafety),
            "burn" | "burn_verification" => Ok(Feature::BurnVerification),
            "bounded" | "bounded_operations" | "floor" => Ok(Feature::BoundedOperations),
            _ => Err(()),
        }
    }
}

impl Feature {
    /// All available features
    pub fn all() -> &'static [Feature] {
        &[
            Feature::SignatureAuth,
            Feature::TimeLock,
            Feature::DatumContinuity,
            Feature::ValuePreservation,
            Feature::ReferenceSafety,
            Feature::BurnVerification,
            Feature::BoundedOperations,
        ]
    }

    /// Human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Feature::SignatureAuth => "signature-auth",
            Feature::TimeLock => "timelock",
            Feature::DatumContinuity => "datum-continuity",
            Feature::ValuePreservation => "value-preservation",
            Feature::ReferenceSafety => "reference-safety",
            Feature::BurnVerification => "burn-verification",
            Feature::BoundedOperations => "bounded-operations",
        }
    }

    /// Description for each feature
    pub fn description(&self) -> &'static str {
        match self {
            Feature::SignatureAuth => "Require a specific signer in extra_signatories",
            Feature::TimeLock => "Enforce validity_range before/after a deadline field",
            Feature::DatumContinuity => "Find continuing output and validate datum preservation",
            Feature::ValuePreservation => "Verify lovelace math (input >= output)",
            Feature::ReferenceSafety => "Reject reference script injection on continuing output",
            Feature::BurnVerification => "Check all minted quantities are negative (mint-only)",
            Feature::BoundedOperations => "Enforce minimum lovelace floor on continuing output",
        }
    }

    /// Sort order for composition (lower = earlier in code)
    pub fn order(&self) -> u8 {
        match self {
            Feature::SignatureAuth => 0,
            Feature::TimeLock => 1,
            Feature::DatumContinuity => 2,
            Feature::ReferenceSafety => 3,
            Feature::ValuePreservation => 4,
            Feature::BoundedOperations => 5,
            Feature::BurnVerification => 6,
        }
    }

    /// Dependencies for this feature
    pub fn depends_on(&self) -> Vec<Feature> {
        feature_spec(*self).depends_on
    }

    /// Purpose restriction: None = any, Some("mint") or Some("spend")
    pub fn purpose_restriction(&self) -> Option<&'static str> {
        feature_spec(*self).purpose
    }
}

/// Specification for a composable feature
pub struct FeatureSpec {
    /// Import lines this feature needs
    pub imports: Vec<&'static str>,
    /// Datum fields this feature requires (name, aiken_type)
    pub required_datum_fields: Vec<(&'static str, &'static str)>,
    /// Validator parameters this feature adds (name, aiken_type)
    pub validator_params: Vec<(&'static str, &'static str)>,
    /// Code emitted before the redeemer match (in validator body, after datum extract)
    pub preamble_code: &'static str,
    /// Code emitted inside each redeemer branch
    pub per_action_code: &'static str,
    /// Features this depends on
    pub depends_on: Vec<Feature>,
    /// Features this conflicts with
    pub conflicts_with: Vec<Feature>,
    /// Purpose restriction: None = any, Some("mint") or Some("spend")
    pub purpose: Option<&'static str>,
}

/// Get the specification for a feature
pub fn feature_spec(f: Feature) -> FeatureSpec {
    match f {
        Feature::SignatureAuth => FeatureSpec {
            imports: vec![
                "use aiken/collection/list",
            ],
            required_datum_fields: vec![],
            validator_params: vec![("admin_pkh", "ByteArray")],
            preamble_code: "",
            per_action_code: "    // Admin must sign the transaction\n    expect list.has(self.extra_signatories, admin_pkh)",
            depends_on: vec![],
            conflicts_with: vec![],
            purpose: None,
        },

        Feature::TimeLock => FeatureSpec {
            imports: vec![
                "use aiken/interval",
            ],
            required_datum_fields: vec![],
            validator_params: vec![],
            preamble_code: "",
            per_action_code: "    // Validity range must be entirely after the deadline\n    expect interval.is_entirely_after(self.validity_range, datum.deadline)",
            depends_on: vec![],
            conflicts_with: vec![],
            purpose: Some("spend"),
        },

        Feature::DatumContinuity => FeatureSpec {
            imports: vec![
                "use aiken/collection/list",
                "use cardano/address.{Address, Script}",
                "use cardano/assets",
                "use cardano/transaction.{InlineDatum, Input, Output}",
            ],
            required_datum_fields: vec![],
            validator_params: vec![],
            preamble_code: concat!(
                "    // Find own input to get address\n",
                "    expect Some(own_input) =\n",
                "      list.find(self.inputs, fn(i) { i.output_reference == own_ref })\n",
                "    let own_address = own_input.output.address\n",
                "\n",
                "    // Find continuing output at same address\n",
                "    expect Some(cont_output) =\n",
                "      list.find(self.outputs, fn(o) { o.address == own_address })\n",
                "\n",
                "    // Validate continuing datum\n",
                "    expect InlineDatum(raw) = cont_output.datum\n",
                "    expect out_datum: CustomDatum = raw",
            ),
            per_action_code: "",
            depends_on: vec![],
            conflicts_with: vec![],
            purpose: Some("spend"),
        },

        Feature::ValuePreservation => FeatureSpec {
            imports: vec![
                "use cardano/assets.{lovelace_of, without_lovelace}",
            ],
            required_datum_fields: vec![],
            validator_params: vec![],
            preamble_code: concat!(
                "    let input_lovelace = lovelace_of(own_input.output.value)\n",
                "    let output_lovelace = lovelace_of(cont_output.value)\n",
                "    let input_non_ada = without_lovelace(own_input.output.value)\n",
                "    let output_non_ada = without_lovelace(cont_output.value)",
            ),
            per_action_code: "    // Preserve non-ADA assets and prevent ADA decrease\n    expect output_non_ada == input_non_ada\n    expect output_lovelace >= input_lovelace",
            depends_on: vec![Feature::DatumContinuity],
            conflicts_with: vec![],
            purpose: Some("spend"),
        },

        Feature::ReferenceSafety => FeatureSpec {
            imports: vec![],
            required_datum_fields: vec![],
            validator_params: vec![],
            preamble_code: "    // Reference script injection protection\n    expect cont_output.reference_script == None",
            per_action_code: "",
            depends_on: vec![Feature::DatumContinuity],
            conflicts_with: vec![],
            purpose: Some("spend"),
        },

        Feature::BurnVerification => FeatureSpec {
            imports: vec![
                "use aiken/collection/dict",
                "use cardano/assets",
            ],
            required_datum_fields: vec![],
            validator_params: vec![],
            preamble_code: "",
            per_action_code: "",
            depends_on: vec![],
            conflicts_with: vec![],
            purpose: Some("mint"),
        },

        Feature::BoundedOperations => FeatureSpec {
            imports: vec![
                "use cardano/assets.{lovelace_of}",
            ],
            required_datum_fields: vec![],
            validator_params: vec![("min_lovelace", "Int")],
            preamble_code: "",
            per_action_code: "    // Enforce minimum lovelace floor on continuing output\n    expect lovelace_of(cont_output.value) >= min_lovelace",
            depends_on: vec![Feature::DatumContinuity],
            conflicts_with: vec![],
            purpose: Some("spend"),
        },
    }
}

/// Parse a list of feature strings from CLI into Feature enums
pub fn parse_features(input: &[String]) -> Result<Vec<Feature>> {
    let mut features = Vec::new();
    for s in input {
        match s.parse::<Feature>() {
            Ok(f) => {
                if !features.contains(&f) {
                    features.push(f);
                }
            }
            Err(_) => {
                let available: Vec<&str> = Feature::all().iter().map(|f| f.name()).collect();
                return Err(KaidoError::InvalidOption(format!(
                    "Unknown feature '{}'. Available: {}",
                    s,
                    available.join(", ")
                )));
            }
        }
    }
    Ok(features)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_feature_aliases() {
        assert_eq!("sig".parse::<Feature>(), Ok(Feature::SignatureAuth));
        assert_eq!("signature".parse::<Feature>(), Ok(Feature::SignatureAuth));
        assert_eq!(
            "signature-auth".parse::<Feature>(),
            Ok(Feature::SignatureAuth)
        );
        assert_eq!("timelock".parse::<Feature>(), Ok(Feature::TimeLock));
        assert_eq!("time-lock".parse::<Feature>(), Ok(Feature::TimeLock));
        assert_eq!(
            "datum-continuity".parse::<Feature>(),
            Ok(Feature::DatumContinuity)
        );
        assert_eq!("datum".parse::<Feature>(), Ok(Feature::DatumContinuity));
        assert_eq!("value".parse::<Feature>(), Ok(Feature::ValuePreservation));
        assert_eq!("burn".parse::<Feature>(), Ok(Feature::BurnVerification));
        assert_eq!("bounded".parse::<Feature>(), Ok(Feature::BoundedOperations));
        assert_eq!("floor".parse::<Feature>(), Ok(Feature::BoundedOperations));
        assert_eq!(
            "ref-safety".parse::<Feature>(),
            Ok(Feature::ReferenceSafety)
        );
        assert!("unknown".parse::<Feature>().is_err());
    }

    #[test]
    fn test_parse_features_list() {
        let input = vec!["sig".to_string(), "timelock".to_string()];
        let features = parse_features(&input).unwrap();
        assert_eq!(features, vec![Feature::SignatureAuth, Feature::TimeLock]);
    }

    #[test]
    fn test_parse_features_dedup() {
        let input = vec!["sig".to_string(), "signature".to_string()];
        let features = parse_features(&input).unwrap();
        assert_eq!(features, vec![Feature::SignatureAuth]);
    }

    #[test]
    fn test_parse_features_unknown() {
        let input = vec!["sig".to_string(), "foobar".to_string()];
        assert!(parse_features(&input).is_err());
    }

    #[test]
    fn test_feature_spec_dependencies() {
        let spec = feature_spec(Feature::ValuePreservation);
        assert!(spec.depends_on.contains(&Feature::DatumContinuity));
    }

    #[test]
    fn test_feature_spec_purpose() {
        assert_eq!(feature_spec(Feature::SignatureAuth).purpose, None);
        assert_eq!(
            feature_spec(Feature::BurnVerification).purpose,
            Some("mint")
        );
        assert_eq!(
            feature_spec(Feature::DatumContinuity).purpose,
            Some("spend")
        );
    }
}
