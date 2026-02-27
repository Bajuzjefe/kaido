use serde::{Deserialize, Serialize};

use super::Feature;
use crate::error::{KaidoError, Result};

/// A field in a custom datum type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatumField {
    pub name: String,
    pub aiken_type: String,
}

/// A redeemer action (variant in the redeemer enum)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemerAction {
    pub name: String,
    /// Fields for this variant: (name, aiken_type)
    pub fields: Vec<(String, String)>,
}

const VALID_TYPES: &[&str] = &["ByteArray", "Int", "Bool", "List<ByteArray>", "List<Int>"];

/// Parse datum field definitions from a CLI string.
/// Format: "owner:ByteArray,amount:Int,deadline:Int"
pub fn parse_datum_fields(input: &str) -> Result<Vec<DatumField>> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(vec![]);
    }

    let mut fields = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        let (name, ty) = part.split_once(':').ok_or_else(|| {
            KaidoError::InvalidOption(format!(
                "Invalid datum field '{}'. Expected format: name:Type (e.g., owner:ByteArray)",
                part
            ))
        })?;

        let name = name.trim().to_string();
        let ty = ty.trim().to_string();

        // Validate the name is a valid Aiken identifier
        if name.is_empty() || !name.chars().next().unwrap().is_ascii_lowercase() {
            return Err(KaidoError::InvalidOption(format!(
                "Datum field name '{}' must start with a lowercase letter",
                name
            )));
        }

        // Validate the type
        if !VALID_TYPES.contains(&ty.as_str()) {
            return Err(KaidoError::InvalidOption(format!(
                "Unsupported type '{}'. Supported: {}",
                ty,
                VALID_TYPES.join(", ")
            )));
        }

        fields.push(DatumField {
            name,
            aiken_type: ty,
        });
    }

    Ok(fields)
}

/// Parse redeemer action definitions from a CLI string.
/// Format: "Claim,Cancel,Withdraw(amount:Int)"
pub fn parse_redeemer_actions(input: &str) -> Result<Vec<RedeemerAction>> {
    let input = input.trim();
    if input.is_empty() {
        return Err(KaidoError::InvalidOption(
            "At least one redeemer action is required".to_string(),
        ));
    }

    let mut actions = Vec::new();

    // Split on commas not inside parentheses
    let mut depth = 0;
    let mut current = String::new();
    let mut parts = Vec::new();

    for ch in input.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                parts.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }

    for part in parts {
        let (name, fields) = if let Some(paren_start) = part.find('(') {
            let name = part[..paren_start].trim().to_string();
            let fields_str = part[paren_start + 1..part.len() - 1].trim();
            let mut fields = Vec::new();
            for field_part in fields_str.split(',') {
                let field_part = field_part.trim();
                let (fname, ftype) = field_part.split_once(':').ok_or_else(|| {
                    KaidoError::InvalidOption(format!(
                        "Invalid field '{}' in redeemer action '{}'. Expected name:Type",
                        field_part, name
                    ))
                })?;
                let ftype = ftype.trim().to_string();
                if !VALID_TYPES.contains(&ftype.as_str()) {
                    return Err(KaidoError::InvalidOption(format!(
                        "Unsupported type '{}' in redeemer field. Supported: {}",
                        ftype,
                        VALID_TYPES.join(", ")
                    )));
                }
                fields.push((fname.trim().to_string(), ftype));
            }
            (name, fields)
        } else {
            (part.trim().to_string(), vec![])
        };

        // Validate name starts with uppercase
        if name.is_empty() || !name.chars().next().unwrap().is_ascii_uppercase() {
            return Err(KaidoError::InvalidOption(format!(
                "Redeemer action '{}' must start with an uppercase letter",
                name
            )));
        }

        actions.push(RedeemerAction { name, fields });
    }

    Ok(actions)
}

/// Validate that selected features are compatible with the provided datum fields.
pub fn validate_features_against_types(
    features: &[Feature],
    datum_fields: &[DatumField],
    purpose: &str,
) -> Result<()> {
    // TimeLock needs an Int field for the deadline
    if features.contains(&Feature::TimeLock) {
        let has_int_field = datum_fields.iter().any(|f| f.aiken_type == "Int");
        if !has_int_field {
            return Err(KaidoError::InvalidOption(
                "Feature 'timelock' requires at least one Int field in datum (e.g., deadline:Int)"
                    .to_string(),
            ));
        }
    }

    // DatumContinuity requires spend purpose
    if features.contains(&Feature::DatumContinuity) && purpose != "spend" {
        return Err(KaidoError::InvalidOption(
            "Feature 'datum-continuity' requires purpose 'spend'".to_string(),
        ));
    }

    // BurnVerification requires mint purpose
    if features.contains(&Feature::BurnVerification) && purpose != "mint" {
        return Err(KaidoError::InvalidOption(
            "Feature 'burn-verification' requires purpose 'mint'".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_datum_simple() {
        let fields = parse_datum_fields("owner:ByteArray,amount:Int").unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name, "owner");
        assert_eq!(fields[0].aiken_type, "ByteArray");
        assert_eq!(fields[1].name, "amount");
        assert_eq!(fields[1].aiken_type, "Int");
    }

    #[test]
    fn test_parse_datum_with_list() {
        let fields = parse_datum_fields("signers:List<ByteArray>,count:Int").unwrap();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].aiken_type, "List<ByteArray>");
    }

    #[test]
    fn test_parse_datum_empty() {
        let fields = parse_datum_fields("").unwrap();
        assert!(fields.is_empty());
    }

    #[test]
    fn test_parse_datum_invalid_type() {
        let result = parse_datum_fields("owner:String");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_datum_bad_format() {
        let result = parse_datum_fields("ownerByteArray");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_datum_uppercase_name() {
        let result = parse_datum_fields("Owner:ByteArray");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_redeemer_simple() {
        let actions = parse_redeemer_actions("Claim,Cancel").unwrap();
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].name, "Claim");
        assert!(actions[0].fields.is_empty());
        assert_eq!(actions[1].name, "Cancel");
    }

    #[test]
    fn test_parse_redeemer_with_fields() {
        let actions = parse_redeemer_actions("Claim,Withdraw(amount:Int)").unwrap();
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[1].name, "Withdraw");
        assert_eq!(actions[1].fields.len(), 1);
        assert_eq!(actions[1].fields[0].0, "amount");
        assert_eq!(actions[1].fields[0].1, "Int");
    }

    #[test]
    fn test_parse_redeemer_empty() {
        let result = parse_redeemer_actions("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_redeemer_lowercase_fails() {
        let result = parse_redeemer_actions("claim");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_timelock_needs_int() {
        let features = vec![Feature::TimeLock];
        let fields = vec![DatumField {
            name: "owner".to_string(),
            aiken_type: "ByteArray".to_string(),
        }];
        assert!(validate_features_against_types(&features, &fields, "spend").is_err());

        let fields_ok = vec![
            DatumField {
                name: "owner".to_string(),
                aiken_type: "ByteArray".to_string(),
            },
            DatumField {
                name: "deadline".to_string(),
                aiken_type: "Int".to_string(),
            },
        ];
        assert!(validate_features_against_types(&features, &fields_ok, "spend").is_ok());
    }

    #[test]
    fn test_validate_burn_needs_mint_purpose() {
        let features = vec![Feature::BurnVerification];
        assert!(validate_features_against_types(&features, &[], "spend").is_err());
        assert!(validate_features_against_types(&features, &[], "mint").is_ok());
    }
}
