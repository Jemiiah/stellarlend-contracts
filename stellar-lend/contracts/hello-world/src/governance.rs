#![allow(dead_code)]
use crate::storage_keys::StorageKey;
use soroban_sdk::{contracttype, Address, Env, Map};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub title: soroban_sdk::String,
    pub created: u64,
    pub voting_ends: u64,
    pub queued_until: u64,
    pub for_votes: i128,
    pub against_votes: i128,
    pub executed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct VoteReceipt {
    pub voter: Address,
    pub support: bool,
    pub weight: i128,
}

/// Governance storage helper using namespaced keys
pub struct GovStorage;

impl GovStorage {
    /// Get the next proposal ID and increment the counter
    pub fn next_id(env: &Env) -> u64 {
        let id: u64 = env
            .storage()
            .instance()
            .get(&StorageKey::gov_counter(env))
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&StorageKey::gov_counter(env), &(id + 1));
        id + 1
    }

    /// Save a proposal to storage
    pub fn save_proposal(env: &Env, p: &Proposal) {
        let mut map: Map<u64, Proposal> = env
            .storage()
            .instance()
            .get(&StorageKey::gov_proposals(env))
            .unwrap_or_else(|| Map::new(env));
        map.set(p.id, p.clone());
        env.storage()
            .instance()
            .set(&StorageKey::gov_proposals(env), &map);
    }

    /// Get a proposal from storage
    pub fn get_proposal(env: &Env, id: u64) -> Option<Proposal> {
        let map: Map<u64, Proposal> = env
            .storage()
            .instance()
            .get(&StorageKey::gov_proposals(env))
            .unwrap_or_else(|| Map::new(env));
        map.get(id)
    }

    /// Save a vote receipt for a proposal
    pub fn save_receipt(env: &Env, id: u64, r: &VoteReceipt) {
        let key = StorageKey::gov_receipts(env, id);
        let mut map: Map<Address, VoteReceipt> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Map::new(env));
        map.set(r.voter.clone(), r.clone());
        env.storage().instance().set(&key, &map);
    }

    /// Get the quorum threshold in basis points
    pub fn get_quorum_bps(env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&StorageKey::gov_quorum_bps(env))
            .unwrap_or(1000)
    }

    /// Set the quorum threshold in basis points
    pub fn set_quorum_bps(env: &Env, bps: i128) {
        env.storage()
            .instance()
            .set(&StorageKey::gov_quorum_bps(env), &bps);
    }

    /// Get the timelock duration in seconds
    pub fn get_timelock(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&StorageKey::gov_timelock(env))
            .unwrap_or(60)
    }

    /// Set the timelock duration in seconds
    pub fn set_timelock(env: &Env, secs: u64) {
        env.storage()
            .instance()
            .set(&StorageKey::gov_timelock(env), &secs);
    }
}

/// Governance module for proposal management
pub struct Governance;

impl Governance {
    /// Create a new proposal
    pub fn propose(
        env: &Env,
        proposer: &Address,
        title: soroban_sdk::String,
        voting_period_secs: u64,
    ) -> Proposal {
        let now = env.ledger().timestamp();
        let id = GovStorage::next_id(env);
        let p = Proposal {
            id,
            proposer: proposer.clone(),
            title,
            created: now,
            voting_ends: now + voting_period_secs,
            queued_until: 0,
            for_votes: 0,
            against_votes: 0,
            executed: false,
        };
        GovStorage::save_proposal(env, &p);
        p
    }

    /// Vote on a proposal
    pub fn vote(env: &Env, id: u64, voter: &Address, support: bool, weight: i128) -> Proposal {
        let mut p = GovStorage::get_proposal(env, id).unwrap();
        if env.ledger().timestamp() > p.voting_ends {
            return p;
        }
        if support {
            p.for_votes += weight;
        } else {
            p.against_votes += weight;
        }
        GovStorage::save_receipt(
            env,
            id,
            &VoteReceipt {
                voter: voter.clone(),
                support,
                weight,
            },
        );
        GovStorage::save_proposal(env, &p);
        p
    }

    /// Queue a proposal for execution after timelock
    pub fn queue(env: &Env, id: u64) -> Proposal {
        let mut p = GovStorage::get_proposal(env, id).unwrap();
        let now = env.ledger().timestamp();
        let quorum = GovStorage::get_quorum_bps(env);
        let total = p.for_votes + p.against_votes;
        let have_quorum = if total == 0 {
            false
        } else {
            (p.for_votes * 10000 / total) >= quorum
        };
        if have_quorum && now >= p.voting_ends {
            p.queued_until = now + GovStorage::get_timelock(env);
        }
        GovStorage::save_proposal(env, &p);
        p
    }

    /// Execute a queued proposal
    pub fn execute(env: &Env, id: u64) -> Proposal {
        let mut p = GovStorage::get_proposal(env, id).unwrap();
        let now = env.ledger().timestamp();
        if now >= p.queued_until && p.queued_until != 0 {
            p.executed = true;
        }
        GovStorage::save_proposal(env, &p);
        p
    }

    /// Delegate voting power to another address
    pub fn delegate(env: &Env, from: &Address, to: &Address) {
        let key = StorageKey::gov_delegation(env, from);
        env.storage().instance().set(&key, to);
    }

    /// Get the delegate for an address
    pub fn get_delegate(env: &Env, from: &Address) -> Option<Address> {
        let key = StorageKey::gov_delegation(env, from);
        env.storage().instance().get(&key)
    }
}
