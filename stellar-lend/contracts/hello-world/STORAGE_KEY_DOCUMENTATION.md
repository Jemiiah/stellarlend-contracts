# Storage Key Namespacing and Hygiene Documentation

## Overview

This document describes the storage key namespacing scheme implemented for the StellarLend protocol to prevent storage key collisions across features and modules and ensure human-auditable storage keys.

## Problem Statement

Previously, the protocol used raw symbol keys directly, which created risks of:
- Storage key collisions between different modules
- Difficulty in auditing and debugging storage usage
- Unclear ownership and purpose of storage keys
- Maintenance challenges when adding new features

## Solution: Hierarchical Namespacing

All storage keys now follow a hierarchical namespacing scheme:

```
{module}_{feature}_{identifier}
```

Where:
- **module**: The main functional area (e.g., "core", "gov", "oracle", "risk")
- **feature**: The specific feature within the module (e.g., "position", "config", "state")
- **identifier**: Optional specific identifier for the data (e.g., user address, asset address)

## Key Modules and Their Namespaces

### 1. Core Protocol (`core_`)

**Purpose**: Core protocol functionality including admin settings, user positions, and security features.

**Keys**:
- `core_config_admin` - Admin address
- `core_config_oracle` - Oracle contract address
- `core_config_min_ratio` - Minimum collateral ratio
- `core_config_flash_fee_bps` - Flash loan fee in basis points
- `core_security_reentrancy` - Reentrancy guard state
- `core_position_user` - User position data (simplified for address handling)

### 2. Interest Rate Management (`interest_`)

**Purpose**: Interest rate calculations and configurations.

**Keys**:
- `interest_config_global` - Global interest rate configuration
- `interest_state_global` - Current interest rate state

### 3. Risk Management (`risk_`)

**Purpose**: Risk parameters and configurations.

**Keys**:
- `risk_config_global` - Global risk configuration
- `risk_params_asset` - Asset-specific risk parameters
- `risk_score_user` - User risk scores

### 4. Governance (`gov_`)

**Purpose**: Governance proposals, voting, and delegation.

**Keys**:
- `gov_counter_proposals` - Proposal ID counter
- `gov_data_proposals` - All proposals map
- `gov_receipts_{proposal_id}` - Vote receipts for specific proposal
- `gov_config_quorum_bps` - Quorum threshold in basis points
- `gov_config_timelock` - Timelock duration
- `gov_delegation_user` - User delegation mappings

### 5. Oracle (`oracle_`)

**Purpose**: Price oracle management and aggregation.

**Keys**:
- `oracle_sources_asset` - Oracle sources for specific asset
- `oracle_config_heartbeat_ttl` - Heartbeat TTL configuration
- `oracle_config_mode` - Oracle aggregation mode
- `oracle_metrics_perf_count` - Performance counter
- `oracle_price_asset` - Price data for specific asset

### 6. Analytics (`analytics_`)

**Purpose**: Protocol analytics and reporting.

**Keys**:
- `analytics_activity_user` - User activity tracking
- `analytics_metrics_protocol` - Protocol-wide metrics
- `analytics_metrics_asset` - Asset-specific metrics
- `analytics_metrics_performance` - Performance metrics

### 7. Flash Loans (`flash_`)

**Purpose**: Flash loan functionality.

**Keys**:
- `flash_active_borrower` - Active flash loan tracking
- `flash_config_fees` - Flash loan fee configuration
- `flash_metrics_stats` - Flash loan statistics

### 8. Asset Management (`asset_`)

**Purpose**: Asset registry and management.

**Keys**:
- `asset_registry_global` - Global asset registry
- `asset_config_specific` - Asset-specific configuration
- `asset_reserves_specific` - Asset reserves
- `asset_util_specific` - Asset utilization

## Implementation

### Key Helper Structs

The implementation provides dedicated helper structs for each module:

```rust
// Core protocol keys
pub struct CoreKeys;
impl CoreKeys {
    pub fn admin(env: &Env) -> Symbol { ... }
    pub fn user_position(env: &Env, user: &Address) -> Symbol { ... }
    // ... other methods
}

// Governance keys
pub struct GovernanceKeys;
impl GovernanceKeys {
    pub fn proposals(env: &Env) -> Symbol { ... }
    pub fn vote_receipts(env: &Env, proposal_id: u64) -> Symbol { ... }
    // ... other methods
}

// Oracle keys
pub struct OracleKeys;
impl OracleKeys {
    pub fn sources(env: &Env, asset: &Address) -> Symbol { ... }
    // ... other methods
}
```

### Composite Key Builder

For dynamic key construction:

```rust
pub struct CompositeKeyBuilder;
impl CompositeKeyBuilder {
    pub fn build(env: &Env, module: &str, feature: &str, identifier: &str) -> Symbol { ... }
    pub fn build_user_key(env: &Env, module: &str, feature: &str, user: &Address) -> Symbol { ... }
    pub fn build_asset_key(env: &Env, module: &str, feature: &str, asset: &Address) -> Symbol { ... }
    pub fn build_numeric_key(env: &Env, module: &str, feature: &str, id: u64) -> Symbol { ... }
}
```

### Key Validation

Utilities for validating key format:

```rust
pub struct KeyValidator;
impl KeyValidator {
    pub fn is_valid_key(key: &str) -> bool { ... }
    pub fn extract_module(key: &str) -> Option<&str> { ... }
    pub fn extract_feature(key: &str) -> Option<&str> { ... }
}
```

## Migration Process

### Before (Raw Keys)
```rust
fn admin_key(env: &Env) -> Symbol {
    Symbol::new(env, "admin")
}

fn position_key(env: &Env, user: &Address) -> Symbol {
    Symbol::new(env, &format!("position_{}", "user"))
}
```

### After (Namespaced Keys)
```rust
pub fn set_admin(env: &Env, admin: &Address) {
    let key = CoreKeys::admin(env);
    env.storage().instance().set(&key, admin);
}

pub fn save_position(env: &Env, position: &Position) {
    let key = CoreKeys::user_position(env, &position.user);
    env.storage().instance().set(&key, position);
}
```

## Benefits

### 1. Collision Prevention
- Each module has its own namespace
- Clear separation between different functional areas
- Reduced risk of accidental key overwrites

### 2. Human Auditability
- Keys are self-documenting
- Easy to understand purpose and ownership
- Simplified debugging and maintenance

### 3. Scalability
- Easy to add new modules without conflicts
- Consistent patterns across the codebase
- Clear guidelines for future development

### 4. Type Safety
- Compile-time validation of key usage
- Centralized key management
- Reduced string literal errors

## Best Practices

### 1. Naming Conventions
- Use lowercase with underscores
- Keep module names short but descriptive
- Use consistent feature names across modules

### 2. Key Organization
- Group related keys in the same module
- Use specific identifiers when needed
- Avoid overly long key names

### 3. Documentation
- Document all key purposes
- Maintain this documentation when adding new keys
- Include examples in code comments

### 4. Testing
- Test key uniqueness across modules
- Validate key format compliance
- Test migration from old to new keys

## Future Considerations

### 1. Key Versioning
Consider adding version prefixes for major protocol upgrades:
```
v2_core_config_admin
```

### 2. Compression
For very long keys, consider using hash-based approaches:
```rust
pub fn user_position_hash(env: &Env, user: &Address) -> Symbol {
    let hash = hash_address(user);
    Symbol::new(env, &format!("core_position_{}", hash))
}
```

### 3. Migration Tools
Develop tools to:
- Migrate existing storage to new key format
- Validate key compliance across the codebase
- Generate key documentation automatically

## Conclusion

The storage key namespacing system provides a robust foundation for the StellarLend protocol's storage management. It ensures collision-free storage, improves auditability, and establishes clear patterns for future development.

All new storage keys should follow this namespacing scheme, and existing keys should be migrated during major protocol updates.
