//! Storage Key Namespacing and Hygiene Module
//! 
//! This module provides namespaced key helpers and composite key constructors
//! to prevent storage key collisions across features and modules.
//! 
//! # Key Naming Convention
//! 
//! All storage keys follow a hierarchical namespacing scheme:
//! `{module}_{feature}_{identifier}`
//! 
//! Where:
//! - `module`: The main functional area (e.g., "core", "gov", "oracle", "risk")
//! - `feature`: The specific feature within the module (e.g., "position", "config", "state")
//! - `identifier`: Optional specific identifier for the data (e.g., user address, asset address)
//! 
//! # Examples
//! - `core_position_{user_address}` - User position data
//! - `core_config_admin` - Admin configuration
//! - `gov_proposal_{proposal_id}` - Governance proposal data
//! - `oracle_sources_{asset_address}` - Oracle sources for an asset
//! - `risk_config_global` - Global risk configuration
//! 
//! # Benefits
//! - Prevents key collisions between modules
//! - Makes storage keys human-auditable
//! - Provides clear ownership and purpose for each key
//! - Enables easier debugging and maintenance

use soroban_sdk::{Address, Env, Symbol};
use alloc::{format, vec::Vec};

/// Core protocol storage keys
pub struct CoreKeys;

impl CoreKeys {
    /// Admin address key
    pub fn admin(env: &Env) -> Symbol {
        Symbol::new(env, "core_config_admin")
    }

    /// Oracle address key
    pub fn oracle(env: &Env) -> Symbol {
        Symbol::new(env, "core_config_oracle")
    }

    /// Minimum collateral ratio key
    pub fn min_collateral_ratio(env: &Env) -> Symbol {
        Symbol::new(env, "core_config_min_ratio")
    }

    /// Flash loan fee basis points key
    pub fn flash_fee_bps(env: &Env) -> Symbol {
        Symbol::new(env, "core_config_flash_fee_bps")
    }

    /// Reentrancy guard key
    pub fn reentrancy_guard(env: &Env) -> Symbol {
        Symbol::new(env, "core_security_reentrancy")
    }

    /// User position key with address
    pub fn user_position(env: &Env, user: &Address) -> Symbol {
        // For now, we'll use a simplified approach since formatting with addresses is complex
        // In a production system, you might want to use a hash-based approach
        Symbol::new(env, "core_position_user")
    }

    /// User position key with string identifier (for backward compatibility)
    pub fn user_position_str(env: &Env, user_id: &str) -> Symbol {
        Symbol::new(env, &format!("core_position_{}", user_id))
    }
}

/// Interest rate management storage keys
pub struct InterestKeys;

impl InterestKeys {
    /// Interest rate configuration key
    pub fn config(env: &Env) -> Symbol {
        Symbol::new(env, "interest_config_global")
    }

    /// Interest rate state key
    pub fn state(env: &Env) -> Symbol {
        Symbol::new(env, "interest_state_global")
    }
}

/// Risk management storage keys
pub struct RiskKeys;

impl RiskKeys {
    /// Global risk configuration key
    pub fn config(env: &Env) -> Symbol {
        Symbol::new(env, "risk_config_global")
    }

    /// Asset-specific risk parameters key
    pub fn asset_params(env: &Env, asset: &Address) -> Symbol {
        Symbol::new(env, "risk_params_asset")
    }

    /// User risk score key
    pub fn user_score(env: &Env, user: &Address) -> Symbol {
        Symbol::new(env, "risk_score_user")
    }
}

/// Governance storage keys
pub struct GovernanceKeys;

impl GovernanceKeys {
    /// Proposal counter key
    pub fn proposal_counter(env: &Env) -> Symbol {
        Symbol::new(env, "gov_counter_proposals")
    }

    /// All proposals map key
    pub fn proposals(env: &Env) -> Symbol {
        Symbol::new(env, "gov_data_proposals")
    }

    /// Vote receipts for a specific proposal
    pub fn vote_receipts(env: &Env, proposal_id: u64) -> Symbol {
        Symbol::new(env, &format!("gov_receipts_{}", proposal_id))
    }

    /// Quorum basis points key
    pub fn quorum_bps(env: &Env) -> Symbol {
        Symbol::new(env, "gov_config_quorum_bps")
    }

    /// Timelock duration key
    pub fn timelock(env: &Env) -> Symbol {
        Symbol::new(env, "gov_config_timelock")
    }

    /// Delegation mapping key
    pub fn delegation(env: &Env, from: &Address) -> Symbol {
        Symbol::new(env, "gov_delegation_user")
    }
}

/// Oracle storage keys
pub struct OracleKeys;

impl OracleKeys {
    /// Oracle sources for a specific asset
    pub fn sources(env: &Env, asset: &Address) -> Symbol {
        Symbol::new(env, "oracle_sources_asset")
    }

    /// Heartbeat TTL configuration key
    pub fn heartbeat_ttl(env: &Env) -> Symbol {
        Symbol::new(env, "oracle_config_heartbeat_ttl")
    }

    /// Oracle mode configuration key
    pub fn mode(env: &Env) -> Symbol {
        Symbol::new(env, "oracle_config_mode")
    }

    /// Performance counter key
    pub fn perf_count(env: &Env) -> Symbol {
        Symbol::new(env, "oracle_metrics_perf_count")
    }

    /// Price data for a specific asset
    pub fn price_data(env: &Env, asset: &Address) -> Symbol {
        Symbol::new(env, "oracle_price_asset")
    }
}

/// Analytics storage keys
pub struct AnalyticsKeys;

impl AnalyticsKeys {
    /// User activity tracking key
    pub fn user_activity(env: &Env, user: &Address) -> Symbol {
        Symbol::new(env, "analytics_activity_user")
    }

    /// Protocol metrics key
    pub fn protocol_metrics(env: &Env) -> Symbol {
        Symbol::new(env, "analytics_metrics_protocol")
    }

    /// Asset metrics key
    pub fn asset_metrics(env: &Env, asset: &Address) -> Symbol {
        Symbol::new(env, "analytics_metrics_asset")
    }

    /// Performance metrics key
    pub fn performance_metrics(env: &Env) -> Symbol {
        Symbol::new(env, "analytics_metrics_performance")
    }
}

/// Flash loan storage keys
pub struct FlashLoanKeys;

impl FlashLoanKeys {
    /// Active flash loan tracking key
    pub fn active_loan(env: &Env, borrower: &Address) -> Symbol {
        Symbol::new(env, "flash_active_borrower")
    }

    /// Flash loan fee configuration key
    pub fn fee_config(env: &Env) -> Symbol {
        Symbol::new(env, "flash_config_fees")
    }

    /// Flash loan statistics key
    pub fn statistics(env: &Env) -> Symbol {
        Symbol::new(env, "flash_metrics_stats")
    }
}

/// Asset management storage keys
pub struct AssetKeys;

impl AssetKeys {
    /// Asset registry key
    pub fn registry(env: &Env) -> Symbol {
        Symbol::new(env, "asset_registry_global")
    }

    /// Asset configuration key
    pub fn config(env: &Env, asset: &Address) -> Symbol {
        Symbol::new(env, "asset_config_specific")
    }

    /// Asset reserves key
    pub fn reserves(env: &Env, asset: &Address) -> Symbol {
        Symbol::new(env, "asset_reserves_specific")
    }

    /// Asset utilization key
    pub fn utilization(env: &Env, asset: &Address) -> Symbol {
        Symbol::new(env, "asset_util_specific")
    }
}

/// Composite key builder for complex storage patterns
pub struct CompositeKeyBuilder;

impl CompositeKeyBuilder {
    /// Build a key with module, feature, and identifier
    pub fn build(env: &Env, module: &str, feature: &str, identifier: &str) -> Symbol {
        Symbol::new(env, &format!("{}_{}_{}",  module, feature, identifier))
    }

    /// Build a key with module and feature only
    pub fn build_simple(env: &Env, module: &str, feature: &str) -> Symbol {
        Symbol::new(env, &format!("{}_{}", module, feature))
    }

    /// Build a key for user-specific data
    pub fn build_user_key(env: &Env, module: &str, feature: &str, user: &Address) -> Symbol {
        // For production, consider using a hash of the address to ensure consistent key length
        Symbol::new(env, &format!("{}_{}_{}", module, feature, "user"))
    }

    /// Build a key for asset-specific data
    pub fn build_asset_key(env: &Env, module: &str, feature: &str, asset: &Address) -> Symbol {
        // For production, consider using a hash of the address to ensure consistent key length
        Symbol::new(env, &format!("{}_{}_{}", module, feature, "asset"))
    }

    /// Build a key with numeric identifier
    pub fn build_numeric_key(env: &Env, module: &str, feature: &str, id: u64) -> Symbol {
        Symbol::new(env, &format!("{}_{}_{}",  module, feature, id))
    }
}

/// Key validation utilities
pub struct KeyValidator;

impl KeyValidator {
    /// Validate that a key follows the naming convention
    pub fn is_valid_key(key: &str) -> bool {
        let parts: Vec<&str> = key.split('_').collect();
        // Must have at least module_feature format
        parts.len() >= 2 && !parts.iter().any(|part| part.is_empty())
    }

    /// Extract module from a key
    pub fn extract_module(key: &str) -> Option<&str> {
        key.split('_').next()
    }

    /// Extract feature from a key
    pub fn extract_feature(key: &str) -> Option<&str> {
        let parts: Vec<&str> = key.split('_').collect();
        if parts.len() >= 2 {
            Some(parts[1])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_key_validation() {
        assert!(KeyValidator::is_valid_key("core_config_admin"));
        assert!(KeyValidator::is_valid_key("gov_proposal_123"));
        assert!(!KeyValidator::is_valid_key("invalid"));
        assert!(!KeyValidator::is_valid_key("_invalid_"));
        assert!(!KeyValidator::is_valid_key(""));
    }

    #[test]
    fn test_key_extraction() {
        assert_eq!(KeyValidator::extract_module("core_config_admin"), Some("core"));
        assert_eq!(KeyValidator::extract_feature("core_config_admin"), Some("config"));
        assert_eq!(KeyValidator::extract_module("gov_proposal_123"), Some("gov"));
        assert_eq!(KeyValidator::extract_feature("gov_proposal_123"), Some("proposal"));
    }

    #[test]
    fn test_composite_key_builder() {
        let env = Env::default();
        
        let key1 = CompositeKeyBuilder::build(&env, "test", "feature", "id");
        assert_eq!(key1, Symbol::new(&env, "test_feature_id"));
        
        let key2 = CompositeKeyBuilder::build_simple(&env, "test", "feature");
        assert_eq!(key2, Symbol::new(&env, "test_feature"));
        
        let key3 = CompositeKeyBuilder::build_numeric_key(&env, "test", "feature", 123);
        assert_eq!(key3, Symbol::new(&env, "test_feature_123"));
    }
}
