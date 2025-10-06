#![allow(dead_code)]
use crate::storage_keys::StorageKey;
use soroban_sdk::{contracttype, vec, Address, Env, IntoVal, Symbol, Vec};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct OracleSource {
    pub addr: Address,
    pub weight: i128, // relative weight (sum can be arbitrary)
    pub last_heartbeat: u64,
}

impl OracleSource {
    pub fn new(addr: Address, weight: i128, last_heartbeat: u64) -> Self {
        Self {
            addr,
            weight,
            last_heartbeat,
        }
    }
}

/// Oracle storage helper using namespaced keys
pub struct OracleStorage;

impl OracleStorage {
    /// Get oracle sources for a specific asset
    pub fn get_sources(env: &Env, asset: &Address) -> Vec<OracleSource> {
        let key = StorageKey::oracle_sources(env, asset);
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Save oracle sources for a specific asset
    pub fn put_sources(env: &Env, asset: &Address, sources: &Vec<OracleSource>) {
        let key = StorageKey::oracle_sources(env, asset);
        env.storage().instance().set(&key, sources);
    }

    /// Get the heartbeat TTL (time-to-live) in seconds
    pub fn get_heartbeat_ttl(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&StorageKey::oracle_heartbeat_ttl(env))
            .unwrap_or(300)
    }

    pub fn set_heartbeat_ttl(
        env: &Env,
        caller: &Address,
        ttl: u64,
    ) -> Result<(), crate::ProtocolError> {
        crate::UserManager::require_admin(env, caller)?;
        env.storage()
            .instance()
            .set(&StorageKey::oracle_heartbeat_ttl(env), &ttl);
        Ok(())
    }

    pub fn set_mode(env: &Env, caller: &Address, mode: i128) -> Result<(), crate::ProtocolError> {
        crate::UserManager::require_admin(env, caller)?;
        env.storage().instance().set(&StorageKey::oracle_mode(env), &mode);
        Ok(())
    }

    /// Get the oracle aggregation mode (0=median, 1=twap)
    pub fn get_mode(env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&StorageKey::oracle_mode(env))
            .unwrap_or(0)
    }

    /// Increment and return the performance counter
    pub fn inc_perf(env: &Env) -> i128 {
        let cur: i128 = env
            .storage()
            .instance()
            .get(&StorageKey::oracle_perf_count(env))
            .unwrap_or(0)
            + 1;
        env.storage()
            .instance()
            .set(&StorageKey::oracle_perf_count(env), &cur);
        cur
    }
}

/// Oracle module for price aggregation
pub struct Oracle;

impl Oracle {
    /// Register or update an oracle source for an asset
    pub fn set_source(
        env: &Env,
        caller: &Address,
        asset: &Address,
        source: OracleSource,
    ) -> Result<(), crate::ProtocolError> {
        crate::UserManager::require_admin(env, caller)?;
        let list = OracleStorage::get_sources(env, asset);
        // Replace if exists
        let mut replaced = false;
        let mut out: Vec<OracleSource> = Vec::new(env);
        for s in list.iter() {
            if s.addr == source.addr {
                out.push_back(source.clone());
                replaced = true;
            } else {
                out.push_back(s);
            }
        }
        if !replaced {
            out.push_back(source);
        }
        OracleStorage::put_sources(env, asset, &out);
        Ok(())
    }

    /// Remove a source
    pub fn remove_source(
        env: &Env,
        caller: &Address,
        asset: &Address,
        addr: &Address,
    ) -> Result<(), crate::ProtocolError> {
        crate::UserManager::require_admin(env, caller)?;
        let list = OracleStorage::get_sources(env, asset);
        let mut out: Vec<OracleSource> = Vec::new(env);
        for s in list.iter() {
            if s.addr != *addr {
                out.push_back(s);
            }
        }
        OracleStorage::put_sources(env, asset, &out);
        Ok(())
    }

    /// Fetch prices from all sources (stubbed as calling `get_price()` on source contracts)
    pub fn fetch_prices(env: &Env, asset: &Address) -> Vec<i128> {
        let list = OracleStorage::get_sources(env, asset);
        let ttl = OracleStorage::get_heartbeat_ttl(env);
        let now = env.ledger().timestamp();
        let mut prices: Vec<i128> = Vec::new(env);
        for s in list.iter() {
            if now.saturating_sub(s.last_heartbeat) > ttl {
                continue;
            }
            // Try calling a standard oracle interface: fn get_price(asset: Address) -> i128
            let args = vec![env, asset.clone().into_val(env)];
            let price: i128 = env.invoke_contract(&s.addr, &Symbol::new(env, "get_price"), args);
            if price > 0 {
                prices.push_back(price);
            }
        }
        prices
    }

    /// Aggregate prices using median; returns None if no healthy sources
    pub fn aggregate_price(env: &Env, asset: &Address) -> Option<i128> {
        let mut prices = Self::fetch_prices(env, asset);
        OracleStorage::inc_perf(env);
        let n = prices.len();
        if n == 0 {
            return None;
        }
        let mode = OracleStorage::get_mode(env);
        if mode == 1 {
            // TWAP approximation: simple average
            let mut sum: i128 = 0;
            for i in 0..n {
                sum += prices.get(i).unwrap_or(0);
            }
            Some(sum / (n as i128))
        } else {
            // Median with outlier trim (drop max and min if enough sources)
            for i in 0..n {
                for j in i + 1..n {
                    if prices.get(i).unwrap() > prices.get(j).unwrap() {
                        let a = prices.get(i).unwrap();
                        let b = prices.get(j).unwrap();
                        prices.set(i, b);
                        prices.set(j, a);
                    }
                }
            }
            let mut start = 0;
            let mut end = n;
            if n >= 3 {
                start = 1;
                end = n - 1;
            }
            let span = end - start;
            if span == 0 {
                return Some(prices.get(0).unwrap());
            }
            let mid = start + span / 2;
            let med = if span % 2 == 1 {
                prices.get(mid).unwrap()
            } else {
                (prices.get(mid - 1).unwrap() + prices.get(mid).unwrap()) / 2
            };
            Some(med)
        }
    }
}
