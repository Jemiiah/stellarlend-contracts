# Storage Key Namespacing and Hygiene

## Overview

This document describes the storage key namespacing scheme used in the StellarLend protocol to prevent collisions across features and modules and ensure maintainability.

## Key Conventions

All storage keys follow a hierarchical namespacing scheme with the following structure:

```
<module_prefix>:<entity_type>[:<identifier>]
```

### Components

1. **Module Prefix**: A short identifier for the module (e.g., `gov`, `oracle`, `user`)
2. **Entity Type**: Describes what is being stored (e.g., `proposal`, `source`, `profile`)
3. **Identifier** (optional): For entity-specific data, includes the entity identifier

## Module Namespaces

### Core Protocol (`core`)
- `admin` - Protocol admin address
- `oracle` - Oracle contract address
- `min_ratio` - Minimum collateral ratio
- `flash_fee_bps` - Flash loan fee in basis points
- `reentrancy` - Reentrancy guard flag

### Governance (`gov`)
- `gov:proposals` - Map of all governance proposals
- `gov:receipts:<proposal_id>` - Vote receipts for a specific proposal
- `gov:counter` - Proposal ID counter
- `gov:quorum_bps` - Quorum threshold in basis points
- `gov:timelock` - Timelock duration in seconds
- `gov:delegation:<delegator>` - Delegation mapping

### Oracle (`oracle`)
- `oracle:sources:<asset>` - Oracle sources for a specific asset
- `oracle:heartbeat_ttl` - Heartbeat time-to-live in seconds
- `oracle:mode` - Aggregation mode (0=median, 1=twap)
- `oracle:perf_count` - Performance counter

### Interest Rates (`interest`)
- `interest_config` - Interest rate configuration parameters
- `interest_state` - Current interest rate state

### Risk Management (`risk`)
- `risk_config` - Risk configuration parameters

### Emergency (`emergency`)
- `emergency_state` - Emergency state and controls

### Events (`event`)
- `event_aggregates` - Event aggregates by type
- `event_logs` - Event logs by type
- `event_summary` - Event summary

### Token Registry (`token`)
- `token_registry` - Token asset registry
- `primary_asset` - Primary asset address

### AMM (`amm`)
- `amm:pairs` - AMM pair registry
- `amm:swap_history` - AMM swap history
- `amm:pair_count` - AMM pair counter

### Analytics (`analytics`)
- `analytics:protocol` - Protocol-level metrics
- `analytics:user:<address>` - User-specific metrics
- `analytics:asset:<address>` - Asset-specific metrics
- `analytics:performance` - Performance metrics

## Usage Examples

### Simple Keys

```rust
use crate::storage_keys::StorageKey;

// Get admin key
let admin_key = StorageKey::admin(&env);

// Get interest config key
let config_key = StorageKey::interest_config(&env);
```

### Composite Keys

```rust
use crate::storage_keys::StorageKey;

// Get governance proposal receipts for proposal #5
let receipts_key = StorageKey::gov_receipts(&env, 5);

// Get oracle sources for a specific asset
let sources_key = StorageKey::oracle_sources(&env, &asset_address);

// Get user delegation
let delegation_key = StorageKey::gov_delegation(&env, &delegator_address);
```

## Benefits

### 1. Collision Prevention
Namespaced keys prevent accidental overwrites between different modules:
```rust
// These are guaranteed to be different
let gov_counter = StorageKey::gov_counter(&env);      // "gov:counter"
let oracle_perf = StorageKey::oracle_perf_count(&env); // "oracle:perf_count"
```

### 2. Auditability
Clear naming makes storage usage traceable:
```rust
// Easy to understand what this stores
let user_profile = StorageKey::user_profile(&user_address);
// vs raw: Symbol::new(env, "profile_GADDRESS...")
```

### 3. Maintainability
Centralized key management simplifies refactoring:
```rust
// Change the key format in one place
impl StorageKey {
    pub fn gov_proposals(env: &Env) -> Symbol {
        Symbol::new(env, "gov:proposals") // Easy to update
    }
}
```

### 4. Type Safety
Strongly-typed key constructors reduce errors:
```rust
// Compiler ensures correct parameters
let key = StorageKey::gov_receipts(&env, proposal_id); // ✓
// let key = StorageKey::gov_receipts(&env, "wrong"); // ✗ Compile error
```

## Migration Guide

### Before (Raw Keys)
```rust
fn proposals_key(env: &Env) -> Symbol {
    Symbol::new(env, "gov_proposals")
}

fn receipts_key(env: &Env) -> Symbol {
    Symbol::new(env, "gov_receipts")
}
```

### After (Namespaced Keys)
```rust
use crate::storage_keys::StorageKey;

// Use centralized key constructors
let proposals = StorageKey::gov_proposals(&env);
let receipts = StorageKey::gov_receipts(&env, proposal_id);
```

## Adding New Keys

When adding new storage keys:

1. **Choose appropriate namespace**: Select or create a module prefix
2. **Add to StorageKey enum**: Add variant to the enum in `storage_keys.rs`
3. **Implement constructor**: Add a static method to construct the key
4. **Document**: Add entry to this document

Example:
```rust
// 1. Add to enum
pub enum StorageKey {
    // ... existing variants
    /// New feature data: feature:data
    FeatureData,
}

// 2. Add constructor
impl StorageKey {
    pub fn feature_data(env: &Env) -> Symbol {
        Symbol::new(env, "feature:data")
    }
}

// 3. Use in code
let key = StorageKey::feature_data(&env);
env.storage().instance().set(&key, &data);
```

## Best Practices

1. **Always use StorageKey helpers**: Never create raw Symbol keys directly
2. **Keep prefixes short**: Use 3-6 character prefixes (e.g., `gov`, `oracle`)
3. **Use descriptive names**: Entity types should be clear (e.g., `proposals`, not `props`)
4. **Document new keys**: Update this file when adding new storage keys
5. **Test key uniqueness**: Ensure new keys don't collide with existing ones

## Testing

The `storage_keys.rs` module includes tests to verify key uniqueness:

```rust
#[test]
fn test_key_uniqueness() {
    let env = Env::default();
    
    let admin = StorageKey::admin(&env);
    let oracle = StorageKey::oracle(&env);
    let gov_counter = StorageKey::gov_counter(&env);
    
    // All keys must be unique
    assert_ne!(admin, oracle);
    assert_ne!(admin, gov_counter);
    assert_ne!(oracle, gov_counter);
}
```

## Future Enhancements

Potential improvements to the storage key system:

1. **Versioning**: Add version prefixes for migration support
2. **Compression**: Use shorter keys for frequently accessed data
3. **Indexing**: Add secondary indices for complex queries
4. **Expiration**: Implement TTL for temporary data

## References

- [Soroban Storage Documentation](https://soroban.stellar.org/docs/fundamentals-and-concepts/storage)
- [Storage Best Practices](https://soroban.stellar.org/docs/learn/best-practices)
