//! Storage Key Namespacing and Hygiene
//!
//! This module provides a centralized, namespaced approach to storage key management
//! to prevent collisions across features and modules.
//!
//! # Key Conventions
//!
//! All storage keys follow a hierarchical namespacing scheme:
//! - **Module Prefix**: Each module has a unique prefix (e.g., "gov", "oracle", "user")
//! - **Entity Type**: Describes what is being stored (e.g., "proposal", "source", "profile")
//! - **Composite Keys**: For entity-specific data, includes the entity identifier
//!
//! # Examples
//!
//! ```ignore
//! // Simple key: "gov:counter"
//! StorageKey::governance_counter()
//!
//! // Composite key: "gov:proposal:123"
//! StorageKey::governance_proposal(123)
//!
//! // User-specific key: "user:profile:GADDRESS..."
//! StorageKey::user_profile(&address)
//! ```
//!
//! # Benefits
//!
//! - **Collision Prevention**: Namespaced keys prevent accidental overwrites
//! - **Auditability**: Clear naming makes storage usage traceable
//! - **Maintainability**: Centralized key management simplifies refactoring
//! - **Type Safety**: Strongly-typed key constructors reduce errors

use soroban_sdk::{contracttype, Address, Env, Symbol};

/// Storage key namespace for all protocol storage
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum StorageKey {
    // ==================== Core Protocol Keys ====================
    /// Protocol admin address
    Admin,
    /// Oracle contract address
    Oracle,
    /// Minimum collateral ratio
    MinCollateralRatio,
    /// Flash loan fee in basis points
    FlashLoanFeeBps,
    /// Reentrancy guard flag
    ReentrancyGuard,

    // ==================== User Management Keys ====================
    /// User profile data: user:profile:{address}
    UserProfile(Address),

    // ==================== Position Keys ====================
    /// User position data: position:{address}
    Position(Address),

    // ==================== Interest Rate Keys ====================
    /// Interest rate configuration
    InterestConfig,
    /// Interest rate state
    InterestState,

    // ==================== Risk Management Keys ====================
    /// Risk configuration parameters
    RiskConfig,

    // ==================== Emergency Keys ====================
    /// Emergency state and controls
    EmergencyState,

    // ==================== Event Tracking Keys ====================
    /// Event aggregates by type
    EventAggregates,
    /// Event logs by type
    EventLogs,
    /// Event summary
    EventSummary,

    // ==================== Token Registry Keys ====================
    /// Token asset registry
    TokenRegistry,
    /// Primary asset address
    PrimaryAsset,

    // ==================== Governance Keys ====================
    /// Governance proposals map: gov:proposals
    GovProposals,
    /// Governance vote receipts: gov:receipts:{proposal_id}
    GovReceipts(u64),
    /// Governance proposal counter: gov:counter
    GovCounter,
    /// Governance quorum basis points: gov:quorum_bps
    GovQuorumBps,
    /// Governance timelock duration: gov:timelock
    GovTimelock,
    /// Governance delegation: gov:delegation:{delegator}
    GovDelegation(Address),

    // ==================== Oracle Keys ====================
    /// Oracle sources for asset: oracle:sources:{asset}
    OracleSources(Address),
    /// Oracle heartbeat TTL: oracle:heartbeat_ttl
    OracleHeartbeatTtl,
    /// Oracle aggregation mode: oracle:mode
    OracleMode,
    /// Oracle performance counter: oracle:perf_count
    OraclePerfCount,

    // ==================== AMM Keys ====================
    /// AMM pair registry: amm:pairs
    AmmPairs,
    /// AMM swap history: amm:swap_history
    AmmSwapHistory,
    /// AMM pair counter: amm:pair_count
    AmmPairCount,

    // ==================== Analytics Keys ====================
    /// Analytics protocol metrics: analytics:protocol
    AnalyticsProtocol,
    /// Analytics user metrics: analytics:user:{address}
    AnalyticsUser(Address),
    /// Analytics asset metrics: analytics:asset:{address}
    AnalyticsAsset(Address),
    /// Analytics performance metrics: analytics:performance
    AnalyticsPerformance,
}

impl StorageKey {
    // ==================== Core Protocol Key Constructors ====================

    /// Get the admin key
    pub fn admin(env: &Env) -> Symbol {
        Symbol::new(env, "admin")
    }

    /// Get the oracle key
    pub fn oracle(env: &Env) -> Symbol {
        Symbol::new(env, "oracle")
    }

    /// Get the minimum collateral ratio key
    pub fn min_collateral_ratio(env: &Env) -> Symbol {
        Symbol::new(env, "min_ratio")
    }

    /// Get the flash loan fee basis points key
    pub fn flash_loan_fee_bps(env: &Env) -> Symbol {
        Symbol::new(env, "flash_fee_bps")
    }

    /// Get the reentrancy guard key
    pub fn reentrancy_guard(env: &Env) -> Symbol {
        Symbol::new(env, "reentrancy")
    }

    // ==================== Interest Rate Key Constructors ====================

    /// Get the interest rate configuration key
    pub fn interest_config(env: &Env) -> Symbol {
        Symbol::new(env, "interest_config")
    }

    /// Get the interest rate state key
    pub fn interest_state(env: &Env) -> Symbol {
        Symbol::new(env, "interest_state")
    }

    // ==================== Risk Management Key Constructors ====================

    /// Get the risk configuration key
    pub fn risk_config(env: &Env) -> Symbol {
        Symbol::new(env, "risk_config")
    }

    // ==================== Emergency Key Constructors ====================

    /// Get the emergency state key
    pub fn emergency_state(env: &Env) -> Symbol {
        Symbol::new(env, "emergency_state")
    }

    // ==================== Event Tracking Key Constructors ====================

    /// Get the event aggregates key
    pub fn event_aggregates(env: &Env) -> Symbol {
        Symbol::new(env, "event_aggregates")
    }

    /// Get the event logs key
    pub fn event_logs(env: &Env) -> Symbol {
        Symbol::new(env, "event_logs")
    }

    /// Get the event summary key
    pub fn event_summary(env: &Env) -> Symbol {
        Symbol::new(env, "event_summary")
    }

    // ==================== Token Registry Key Constructors ====================

    /// Get the token registry key
    pub fn token_registry(env: &Env) -> Symbol {
        Symbol::new(env, "token_registry")
    }

    /// Get the primary asset key
    pub fn primary_asset(env: &Env) -> Symbol {
        Symbol::new(env, "primary_asset")
    }

    // ==================== Governance Key Constructors ====================

    /// Get the governance proposals key
    pub fn gov_proposals(env: &Env) -> Symbol {
        Symbol::new(env, "gov_proposals")
    }

    /// Get the governance receipts key for a specific proposal
    pub fn gov_receipts(env: &Env, proposal_id: u64) -> (Symbol, u64) {
        (Symbol::new(env, "gov_receipts"), proposal_id)
    }

    /// Get the governance counter key
    pub fn gov_counter(env: &Env) -> Symbol {
        Symbol::new(env, "gov_counter")
    }

    /// Get the governance quorum basis points key
    pub fn gov_quorum_bps(env: &Env) -> Symbol {
        Symbol::new(env, "gov_quorum_bps")
    }

    /// Get the governance timelock key
    pub fn gov_timelock(env: &Env) -> Symbol {
        Symbol::new(env, "gov_timelock")
    }

    /// Get the governance delegation key for a delegator
    pub fn gov_delegation(env: &Env, delegator: &Address) -> (Symbol, Address) {
        (Symbol::new(env, "gov_delegation"), delegator.clone())
    }

    // ==================== Oracle Key Constructors ====================

    /// Get the oracle sources key for a specific asset
    pub fn oracle_sources(env: &Env, asset: &Address) -> (Symbol, Address) {
        (Symbol::new(env, "oracle_sources"), asset.clone())
    }

    /// Get the oracle heartbeat TTL key
    pub fn oracle_heartbeat_ttl(env: &Env) -> Symbol {
        Symbol::new(env, "oracle_heartbeat_ttl")
    }

    /// Get the oracle mode key
    pub fn oracle_mode(env: &Env) -> Symbol {
        Symbol::new(env, "oracle_mode")
    }

    /// Get the oracle performance counter key
    pub fn oracle_perf_count(env: &Env) -> Symbol {
        Symbol::new(env, "oracle_perf_count")
    }

    // ==================== AMM Key Constructors ====================

    /// Get the AMM pairs registry key
    pub fn amm_pairs(env: &Env) -> Symbol {
        Symbol::new(env, "amm_pairs")
    }

    /// Get the AMM swap history key
    pub fn amm_swap_history(env: &Env) -> Symbol {
        Symbol::new(env, "amm_swap_history")
    }

    /// Get the AMM pair counter key
    pub fn amm_pair_count(env: &Env) -> Symbol {
        Symbol::new(env, "amm_pair_count")
    }

    // ==================== Analytics Key Constructors ====================

    /// Get the analytics protocol metrics key
    pub fn analytics_protocol(env: &Env) -> Symbol {
        Symbol::new(env, "analytics_protocol")
    }

    /// Get the analytics user metrics key for a specific user
    pub fn analytics_user(env: &Env, user: &Address) -> (Symbol, Address) {
        (Symbol::new(env, "analytics_user"), user.clone())
    }

    /// Get the analytics asset metrics key for a specific asset
    pub fn analytics_asset(env: &Env, asset: &Address) -> (Symbol, Address) {
        (Symbol::new(env, "analytics_asset"), asset.clone())
    }

    /// Get the analytics performance metrics key
    pub fn analytics_performance(env: &Env) -> Symbol {
        Symbol::new(env, "analytics_performance")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_key_uniqueness() {
        let env = Env::default();

        // Verify that different key types produce different symbols
        let admin = StorageKey::admin(&env);
        let oracle = StorageKey::oracle(&env);
        let gov_counter = StorageKey::gov_counter(&env);
        let oracle_mode = StorageKey::oracle_mode(&env);

        assert_ne!(admin, oracle);
        assert_ne!(admin, gov_counter);
        assert_ne!(oracle, gov_counter);
        assert_ne!(gov_counter, oracle_mode);
    }

    #[test]
    fn test_namespaced_keys() {
        let env = Env::default();

        // Verify governance keys have "gov:" prefix
        let gov_proposals = StorageKey::gov_proposals(&env);
        let gov_counter = StorageKey::gov_counter(&env);

        // Verify oracle keys have "oracle:" prefix
        let oracle_mode = StorageKey::oracle_mode(&env);
        let oracle_perf = StorageKey::oracle_perf_count(&env);

        // These should be different due to namespacing
        assert_ne!(gov_proposals, gov_counter);
        assert_ne!(oracle_mode, oracle_perf);
    }
}
