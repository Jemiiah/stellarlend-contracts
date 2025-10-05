#![cfg(test)]

use super::*;
use soroban_sdk::{
    contract, contractimpl, testutils, testutils::Address as TestAddress, Address, Env, Map,
    String, Symbol,
};

use crate::{FlashLoan, ProtocolError, ReentrancyGuard};

#[contract]
pub struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn initialize(env: Env, admin: Address) {
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "admin"), &admin);
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        Self::add_balance(&env, &to, amount);
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        Self::deduct_balance(&env, &from, amount);
        Self::add_balance(&env, &to, amount);
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        Self::get_balance(&env, &id)
    }
}

impl MockToken {
    fn balances_key(env: &Env) -> Symbol {
        Symbol::new(env, "balances")
    }

    fn get_balances(env: &Env) -> Map<Address, i128> {
        env.storage()
            .instance()
            .get(&Self::balances_key(env))
            .unwrap_or_else(|| Map::new(env))
    }

    fn save_balances(env: &Env, balances: &Map<Address, i128>) {
        env.storage()
            .instance()
            .set(&Self::balances_key(env), balances);
    }

    fn get_balance(env: &Env, addr: &Address) -> i128 {
        Self::get_balances(env).get(addr.clone()).unwrap_or(0)
    }

    fn add_balance(env: &Env, addr: &Address, amount: i128) {
        let mut balances = Self::get_balances(env);
        let current = balances.get(addr.clone()).unwrap_or(0);
        balances.set(addr.clone(), current.saturating_add(amount));
        Self::save_balances(env, &balances);
    }

    fn deduct_balance(env: &Env, addr: &Address, amount: i128) {
        let mut balances = Self::get_balances(env);
        let current = balances.get(addr.clone()).unwrap_or(0);
        if current < amount {
            panic!("insufficient balance");
        }
        balances.set(addr.clone(), current - amount);
        Self::save_balances(env, &balances);
    }
}

#[contract]
pub struct FlashLoanReceiver;

#[contractimpl]
impl FlashLoanReceiver {
    pub fn on_flash_loan(
        _env: Env,
        _asset: Address,
        _amount: i128,
        _fee: i128,
        _initiator: Address,
    ) {
    }
}

/// Test utilities for creating test environments and addresses
pub struct TestUtils;

impl TestUtils {
    /// Create a test environment
    pub fn create_test_env() -> Env {
        let env = Env::default();
        env.mock_all_auths();
        env
    }

    /// Create a test address from a string
    pub fn create_test_address(env: &Env, address_str: &str) -> Address {
        Address::from_string(&String::from_str(env, address_str))
    }

    /// Create a test admin address
    pub fn create_admin_address(env: &Env) -> Address {
        Self::create_test_address(
            env,
            "GCAZYE3EB54VKP3UQBX3H73VQO3SIWTZNR7NJQKJFZZ6XLADWA4C3SOC",
        )
    }

    /// Create a test user address
    pub fn create_user_address(env: &Env, user_id: u32) -> Address {
        match user_id {
            0 => Self::create_test_address(
                env,
                "GCXOTMMXRS24MYZI5FJPUCOEOFNWSR4XX7UXIK3NDGGE6A5QMJ5FF2FS",
            ),
            1 => Self::create_test_address(
                env,
                "GAUA7XL5K54CC2DDGP77FJ2YBHRJLT36CPZDXWPM6MP7MANOGG77PNJU",
            ),
            2 => Self::create_test_address(
                env,
                "GCUA7XL5K54CC2DDGP77FJ2YBHRJLT36CPZDXWPM6MP7MANOGG77PNJU",
            ),
            _ => Self::create_test_address(
                env,
                "GCUA7XL5K54CC2DDGP77FJ2YBHRJLT36CPZDXWPM6MP7MANOGG77PNJU",
            ),
        }
    }

    pub fn setup_contract_with_token(env: &Env, users: &[Address]) -> (Address, Address, Address) {
        let admin = Self::create_admin_address(env);
        let contract_id = env.register(Contract, ());
        env.as_contract(&contract_id, || {
            Contract::initialize(env.clone(), admin.to_string()).unwrap();
        });

        let token_id = env.register_contract(None, MockToken);
        env.as_contract(&token_id, || {
            MockToken::initialize(env.clone(), admin.clone());
        });

        env.as_contract(&contract_id, || {
            Contract::set_primary_asset(env.clone(), admin.to_string(), token_id.clone()).unwrap();
        });

        env.as_contract(&token_id, || {
            MockToken::mint(env.clone(), contract_id.clone(), 1_000_000);
            for addr in users {
                MockToken::mint(env.clone(), addr.clone(), 1_000_000);
            }
        });

        (admin, contract_id, token_id)
    }

    /// Initialize the contract with test admin
    pub fn initialize_contract(env: &Env) -> Address {
        let admin = Self::create_admin_address(env);
        let contract_id = env.register(Contract, ());
        env.as_contract(&contract_id, || {
            Contract::initialize(env.clone(), admin.to_string()).unwrap();
        });
        admin
    }

    /// Mark a user as verified for testing convenience
    pub fn verify_user(env: &Env, admin: &Address, user: &Address) {
        Contract::set_user_verification(
            env.clone(),
            admin.to_string(),
            user.clone(),
            VerificationStatus::Verified,
        )
        .unwrap();
    }
}

#[test]
fn test_contract_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = TestUtils::create_admin_address(&env);

    let contract_id = env.register(Contract, ());
    env.as_contract(&contract_id, || {
        let result = Contract::initialize(env.clone(), admin.to_string());
        assert!(result.is_ok());
    });
}

#[test]
fn test_contract_already_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = TestUtils::create_admin_address(&env);

    let contract_id = env.register(Contract, ());
    env.as_contract(&contract_id, || {
        // First initialization should succeed
        let result = Contract::initialize(env.clone(), admin.to_string());
        assert!(result.is_ok());

        // Second initialization should fail
        let result = Contract::initialize(env.clone(), admin.to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProtocolError::AlreadyInitialized);
    });
}

#[test]
fn test_deposit_collateral() {
    let env = Env::default();
    env.mock_all_auths();

    let user = TestUtils::create_user_address(&env, 0);

    let (admin, contract_id, _token) = TestUtils::setup_contract_with_token(&env, &[user.clone()]);
    env.as_contract(&contract_id, || {
        TestUtils::verify_user(&env, &admin, &user);

        // Test successful deposit
        let result = Contract::deposit_collateral(env.clone(), user.to_string(), 1000);
        assert!(result.is_ok());

        // Verify position
        let position = Contract::get_position(env.clone(), user.to_string()).unwrap();
        assert_eq!(position.0, 1000); // collateral
        assert_eq!(position.1, 0); // debt
    });
}

#[test]
fn test_deposit_collateral_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let user = TestUtils::create_user_address(&env, 0);

    let (admin, contract_id, _token) = TestUtils::setup_contract_with_token(&env, &[user.clone()]);
    env.as_contract(&contract_id, || {
        TestUtils::verify_user(&env, &admin, &user);

        // Test deposit with zero amount
        let result = Contract::deposit_collateral(env.clone(), user.to_string(), 0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProtocolError::InvalidAmount);

        // Test deposit with negative amount
        let result = Contract::deposit_collateral(env.clone(), user.to_string(), -100);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProtocolError::InvalidAmount);
    });
}

#[test]
fn test_deposit_collateral_invalid_address() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = TestUtils::create_admin_address(&env);

    let contract_id = env.register(Contract, ());
    env.as_contract(&contract_id, || {
        // Initialize contract
        Contract::initialize(env.clone(), admin.to_string()).unwrap();

        // Test deposit with empty address
        let result = Contract::deposit_collateral(env.clone(), String::from_str(&env, ""), 1000);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProtocolError::InvalidAddress);
    });
}

#[test]
fn test_borrow_success() {
    let env = Env::default();
    env.mock_all_auths();

    let user = TestUtils::create_user_address(&env, 0);

    let (admin, contract_id, _token) = TestUtils::setup_contract_with_token(&env, &[user.clone()]);
    env.as_contract(&contract_id, || {
        TestUtils::verify_user(&env, &admin, &user);

        // Deposit collateral first
        Contract::deposit_collateral(env.clone(), user.to_string(), 2000).unwrap();

        // Test successful borrow
        let result = Contract::borrow(env.clone(), user.to_string(), 1000);
        assert!(result.is_ok());

        // Verify position
        let position = Contract::get_position(env.clone(), user.to_string()).unwrap();
        assert_eq!(position.0, 2000); // collateral
        assert_eq!(position.1, 1000); // debt
    });
}

#[test]
fn test_borrow_insufficient_collateral_ratio() {
    let env = Env::default();
    env.mock_all_auths();

    let user = TestUtils::create_user_address(&env, 0);

    let (admin, contract_id, _token) = TestUtils::setup_contract_with_token(&env, &[user.clone()]);
    env.as_contract(&contract_id, || {
        TestUtils::verify_user(&env, &admin, &user);

        // Deposit small amount of collateral
        Contract::deposit_collateral(env.clone(), user.to_string(), 100).unwrap();

        // Try to borrow too much (should fail due to insufficient collateral ratio)
        let result = Contract::borrow(env.clone(), user.to_string(), 1000);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ProtocolError::InsufficientCollateralRatio
        );
    });
}

#[test]
fn test_emergency_pause_blocks_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let user = TestUtils::create_user_address(&env, 0);

    let (admin, contract_id, _token) = TestUtils::setup_contract_with_token(&env, &[user.clone()]);
    env.as_contract(&contract_id, || {
        TestUtils::verify_user(&env, &admin, &user);

        let reason = Some(String::from_str(&env, "halt"));
        Contract::trigger_emergency_pause(env.clone(), admin.to_string(), reason).unwrap();

        let result = Contract::deposit_collateral(env.clone(), user.to_string(), 1000);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProtocolError::ProtocolPaused);

        Contract::resume_operations(env.clone(), admin.to_string()).unwrap();
        let result = Contract::deposit_collateral(env.clone(), user.to_string(), 1000);
        assert!(result.is_ok());
    });
}

#[test]
fn test_recovery_mode_allows_repay_blocks_borrow() {
    let env = Env::default();
    env.mock_all_auths();

    let user = TestUtils::create_user_address(&env, 0);

    let (admin, contract_id, _token) = TestUtils::setup_contract_with_token(&env, &[user.clone()]);
    env.as_contract(&contract_id, || {
        TestUtils::verify_user(&env, &admin, &user);

        Contract::deposit_collateral(env.clone(), user.to_string(), 2000).unwrap();
        Contract::borrow(env.clone(), user.to_string(), 500).unwrap();

        let plan = Some(String::from_str(&env, "staged restart"));
        Contract::enter_recovery_mode(env.clone(), admin.to_string(), plan).unwrap();

        Contract::record_recovery_step(
            env.clone(),
            admin.to_string(),
            String::from_str(&env, "notified stakeholders"),
        )
        .unwrap();

        // Repay should be allowed in recovery mode
        let repay_result = Contract::repay(env.clone(), user.to_string(), 200);
        assert!(repay_result.is_ok());

        // Borrow should be restricted while in recovery
        let borrow_result = Contract::borrow(env.clone(), user.to_string(), 100);
        assert!(borrow_result.is_err());
        assert_eq!(
            borrow_result.unwrap_err(),
            ProtocolError::RecoveryModeRestricted
        );

        let state = Contract::get_emergency_state(env.clone()).unwrap();
        assert_eq!(state.status, EmergencyStatus::Recovery);
        assert_eq!(state.recovery_steps.len(), 1u32);
    });
}

#[test]
fn test_emergency_param_updates_apply() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = TestUtils::create_admin_address(&env);

    let contract_id = env.register(Contract, ());
    env.as_contract(&contract_id, || {
        Contract::initialize(env.clone(), admin.to_string()).unwrap();

        let base_rate_symbol = Symbol::new(&env, "base_rate");
        Contract::queue_emergency_param_update(
            env.clone(),
            admin.to_string(),
            base_rate_symbol,
            5000000,
        )
        .unwrap();
        Contract::apply_emergency_param_updates(env.clone(), admin.to_string()).unwrap();

        let config = InterestRateStorage::get_config(&env);
        assert_eq!(config.base_rate, 5000000);

        let state = Contract::get_emergency_state(env.clone()).unwrap();
        assert_eq!(state.pending_param_updates.len(), 0u32);
    });
}

#[test]
fn test_emergency_fund_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = TestUtils::create_admin_address(&env);
    let recipient = TestUtils::create_user_address(&env, 1);

    let contract_id = env.register(Contract, ());
    env.as_contract(&contract_id, || {
        Contract::initialize(env.clone(), admin.to_string()).unwrap();

        let token = Some(recipient.clone());
        Contract::adjust_emergency_fund(
            env.clone(),
            admin.to_string(),
            token.clone(),
            1_000_000,
            500_000,
        )
        .unwrap();

        let state = Contract::get_emergency_state(env.clone()).unwrap();
        assert_eq!(state.fund.balance, 1_000_000);
        assert_eq!(state.fund.reserved, 500_000);
        assert_eq!(state.fund.token, token);

        let err =
            Contract::adjust_emergency_fund(env.clone(), admin.to_string(), None, -2_000_000, 0)
                .unwrap_err();
        assert_eq!(err, ProtocolError::EmergencyFundInsufficient);
    });
}

#[test]
fn test_repay_success() {
    let env = Env::default();
    env.mock_all_auths();

    let user = TestUtils::create_user_address(&env, 0);

    let (admin, contract_id, _token) = TestUtils::setup_contract_with_token(&env, &[user.clone()]);
    env.as_contract(&contract_id, || {
        TestUtils::verify_user(&env, &admin, &user);

        // Deposit and borrow
        Contract::deposit_collateral(env.clone(), user.to_string(), 2000).unwrap();
        Contract::borrow(env.clone(), user.to_string(), 1000).unwrap();

        // Test successful repayment
        let result = Contract::repay(env.clone(), user.to_string(), 500);
        assert!(result.is_ok());

        // Verify position
        let position = Contract::get_position(env.clone(), user.to_string()).unwrap();
        assert_eq!(position.0, 2000); // collateral
        assert_eq!(position.1, 500); // debt
    });
}

#[test]
fn test_repay_full_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let user = TestUtils::create_user_address(&env, 0);

    let (admin, contract_id, _token) = TestUtils::setup_contract_with_token(&env, &[user.clone()]);
    env.as_contract(&contract_id, || {
        TestUtils::verify_user(&env, &admin, &user);

        // Deposit and borrow
        Contract::deposit_collateral(env.clone(), user.to_string(), 2000).unwrap();
        Contract::borrow(env.clone(), user.to_string(), 1000).unwrap();

        // Test full repayment
        let result = Contract::repay(env.clone(), user.to_string(), 1000);
        assert!(result.is_ok());

        // Verify position
        let position = Contract::get_position(env.clone(), user.to_string()).unwrap();
        assert_eq!(position.0, 2000); // collateral
        assert_eq!(position.1, 0); // debt
    });
}

#[test]
fn test_withdraw_success() {
    let env = Env::default();
    env.mock_all_auths();

    let user = TestUtils::create_user_address(&env, 0);

    let (admin, contract_id, _token) = TestUtils::setup_contract_with_token(&env, &[user.clone()]);
    env.as_contract(&contract_id, || {
        TestUtils::verify_user(&env, &admin, &user);

        // Deposit collateral
        Contract::deposit_collateral(env.clone(), user.to_string(), 2000).unwrap();

        // Test successful withdrawal
        let result = Contract::withdraw(env.clone(), user.to_string(), 1000);
        assert!(result.is_ok());

        // Verify position
        let position = Contract::get_position(env.clone(), user.to_string()).unwrap();
        assert_eq!(position.0, 1000); // collateral
        assert_eq!(position.1, 0); // debt
    });
}

#[test]
fn test_event_summary_updates() {
    let env = Env::default();
    env.mock_all_auths();

    let user = TestUtils::create_user_address(&env, 0);

    let (admin, contract_id, _token) = TestUtils::setup_contract_with_token(&env, &[user.clone()]);
    env.as_contract(&contract_id, || {
        TestUtils::verify_user(&env, &admin, &user);

        Contract::deposit_collateral(env.clone(), user.to_string(), 1200).unwrap();
        Contract::withdraw(env.clone(), user.to_string(), 200).unwrap();

        let summary = Contract::get_event_summary(env.clone()).unwrap();
        let totals = summary.totals;
        let key = Symbol::new(&env, "position_updated");
        let aggregate = totals.get(key).unwrap();
        assert!(aggregate.count > 0);

        let recent_types = Contract::get_recent_event_types(env.clone()).unwrap();
        assert!(recent_types.len() > 0);

        let events =
            Contract::get_events_for_type(env.clone(), Symbol::new(&env, "position_updated"), 5)
                .unwrap();
        assert!(events.len() > 0);

        let aggregates = Contract::get_event_aggregates(env.clone()).unwrap();
        assert!(aggregates.len() >= totals.len());
    });
}

#[test]
fn test_deposit_reentrancy_blocked() {
    let env = Env::default();
    env.mock_all_auths();

    let user = TestUtils::create_user_address(&env, 0);
    let (_admin, contract_id, _token) = TestUtils::setup_contract_with_token(&env, &[user.clone()]);

    env.as_contract(&contract_id, || {
        ReentrancyGuard::enter(&env).unwrap();
        let result = Contract::deposit_collateral(env.clone(), user.to_string(), 100);
        ReentrancyGuard::exit(&env);
        assert_eq!(Err(ProtocolError::ReentrancyDetected), result);
    });
}

#[test]
fn test_withdraw_insufficient_collateral() {
    let env = Env::default();
    env.mock_all_auths();

    let user = TestUtils::create_user_address(&env, 0);

    let (admin, contract_id, _token) = TestUtils::setup_contract_with_token(&env, &[user.clone()]);
    env.as_contract(&contract_id, || {
        TestUtils::verify_user(&env, &admin, &user);

        // Deposit small amount
        Contract::deposit_collateral(env.clone(), user.to_string(), 100).unwrap();

        // Try to withdraw more than deposited
        let result = Contract::withdraw(env.clone(), user.to_string(), 200);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProtocolError::InsufficientCollateral);
    });
}

#[test]
fn test_withdraw_insufficient_collateral_ratio() {
    let env = Env::default();
    env.mock_all_auths();

    let user = TestUtils::create_user_address(&env, 0);

    let (admin, contract_id, _token) = TestUtils::setup_contract_with_token(&env, &[user.clone()]);
    env.as_contract(&contract_id, || {
        TestUtils::verify_user(&env, &admin, &user);

        // Deposit and borrow
        Contract::deposit_collateral(env.clone(), user.to_string(), 2000).unwrap();
        Contract::borrow(env.clone(), user.to_string(), 1000).unwrap();

        // Try to withdraw too much (would make collateral ratio too low)
        let result = Contract::withdraw(env.clone(), user.to_string(), 1500);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ProtocolError::InsufficientCollateralRatio
        );
    });
}

#[test]
fn test_liquidate_success() {
    let env = Env::default();
    env.mock_all_auths();

    let user = TestUtils::create_user_address(&env, 0);
    let liquidator = TestUtils::create_user_address(&env, 1);

    let (admin, contract_id, _token) =
        TestUtils::setup_contract_with_token(&env, &[user.clone(), liquidator.clone()]);
    env.as_contract(&contract_id, || {
        TestUtils::verify_user(&env, &admin, &user);
        TestUtils::verify_user(&env, &admin, &liquidator);

        // Set a very low minimum collateral ratio for testing
        Contract::set_min_collateral_ratio(env.clone(), admin.to_string(), 50).unwrap();

        // Deposit collateral and borrow to create undercollateralized position
        Contract::deposit_collateral(env.clone(), user.to_string(), 1000).unwrap();
        Contract::borrow(env.clone(), user.to_string(), 1000).unwrap();

        // Now set the minimum ratio back to a higher value to make the position undercollateralized
        Contract::set_min_collateral_ratio(env.clone(), admin.to_string(), 150).unwrap();

        // Test successful liquidation
        let result =
            Contract::liquidate(env.clone(), liquidator.to_string(), user.to_string(), 500);
        assert!(result.is_ok());
    });
}

#[test]
fn test_liquidate_not_eligible() {
    let env = Env::default();
    env.mock_all_auths();

    let user = TestUtils::create_user_address(&env, 0);
    let liquidator = TestUtils::create_user_address(&env, 1);

    let (admin, contract_id, _token) =
        TestUtils::setup_contract_with_token(&env, &[user.clone(), liquidator.clone()]);
    env.as_contract(&contract_id, || {
        TestUtils::verify_user(&env, &admin, &user);
        TestUtils::verify_user(&env, &admin, &liquidator);

        // Deposit large amount and borrow small amount (healthy position)
        Contract::deposit_collateral(env.clone(), user.to_string(), 2000).unwrap();
        Contract::borrow(env.clone(), user.to_string(), 1000).unwrap();

        // Try to liquidate (should fail)
        let result =
            Contract::liquidate(env.clone(), liquidator.to_string(), user.to_string(), 500);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ProtocolError::NotEligibleForLiquidation
        );
    });
}

#[test]
fn test_flash_loan_reentrancy_blocked() {
    let env = Env::default();
    env.mock_all_auths();

    let initiator = TestUtils::create_user_address(&env, 0);
    let (_admin, contract_id, token_id) =
        TestUtils::setup_contract_with_token(&env, &[initiator.clone()]);
    let receiver = env.register_contract(None, FlashLoanReceiver);

    env.as_contract(&contract_id, || {
        ReentrancyGuard::enter(&env).unwrap();
        let result = FlashLoan::execute(&env, &initiator, &token_id, 100, 10, &receiver);
        ReentrancyGuard::exit(&env);
        assert_eq!(Err(ProtocolError::ReentrancyDetected), result);
    });
}

#[test]
fn test_set_risk_params() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = TestUtils::create_admin_address(&env);

    let contract_id = env.register(Contract, ());
    env.as_contract(&contract_id, || {
        // Initialize contract
        Contract::initialize(env.clone(), admin.to_string()).unwrap();

        // Test setting risk parameters
        let result = Contract::set_risk_params(env.clone(), admin.to_string(), 60000000, 15000000);
        assert!(result.is_ok());

        // Verify the parameters were set
        let risk_config = Contract::get_risk_config(env.clone()).unwrap();
        assert_eq!(risk_config.0, 60000000); // close_factor
        assert_eq!(risk_config.1, 15000000); // liquidation_incentive
    });
}

#[test]
fn test_set_risk_params_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = TestUtils::create_admin_address(&env);
    let user = TestUtils::create_user_address(&env, 0);

    let contract_id = env.register(Contract, ());
    env.as_contract(&contract_id, || {
        // Initialize contract
        Contract::initialize(env.clone(), admin.to_string()).unwrap();

        // Test setting risk parameters with non-admin (should fail)
        let result = Contract::set_risk_params(env.clone(), user.to_string(), 60000000, 15000000);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProtocolError::Unauthorized);
    });
}

#[test]
fn test_set_pause_switches() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = TestUtils::create_admin_address(&env);

    let contract_id = env.register(Contract, ());
    env.as_contract(&contract_id, || {
        // Initialize contract
        Contract::initialize(env.clone(), admin.to_string()).unwrap();

        // Test setting pause switches
        let result = Contract::set_pause_switches(
            env.clone(),
            admin.to_string(),
            true,  // pause_borrow
            false, // pause_deposit
            true,  // pause_withdraw
            false, // pause_liquidate
        );
        assert!(result.is_ok());

        // Verify the switches were set
        let risk_config = Contract::get_risk_config(env.clone()).unwrap();
        assert_eq!(risk_config.2, true); // pause_borrow
        assert_eq!(risk_config.3, false); // pause_deposit
        assert_eq!(risk_config.4, true); // pause_withdraw
        assert_eq!(risk_config.5, false); // pause_liquidate
    });
}

#[test]
fn test_get_protocol_params() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = TestUtils::create_admin_address(&env);

    let contract_id = env.register(Contract, ());
    env.as_contract(&contract_id, || {
        // Initialize contract
        Contract::initialize(env.clone(), admin.to_string()).unwrap();

        // Test getting protocol parameters
        let params = Contract::get_protocol_params(env.clone()).unwrap();
        assert_eq!(params.0, 2000000); // base_rate
        assert_eq!(params.1, 80000000); // kink_utilization
        assert_eq!(params.2, 10000000); // multiplier
        assert_eq!(params.3, 10000000); // reserve_factor
        assert_eq!(params.4, 50000000); // close_factor
        assert_eq!(params.5, 10000000); // liquidation_incentive
    });
}

#[test]
fn test_get_system_stats() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = TestUtils::create_admin_address(&env);

    let contract_id = env.register(Contract, ());
    env.as_contract(&contract_id, || {
        // Initialize contract
        Contract::initialize(env.clone(), admin.to_string()).unwrap();

        // Test getting system stats
        let stats = Contract::get_system_stats(env.clone()).unwrap();
        assert_eq!(stats.0, 0); // total_supplied
        assert_eq!(stats.1, 0); // total_borrowed
        assert_eq!(stats.2, 0); // current_borrow_rate
        assert_eq!(stats.3, 0); // current_supply_rate
    });
}

#[test]
fn test_get_position_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = TestUtils::create_admin_address(&env);
    let user = TestUtils::create_user_address(&env, 0);

    let contract_id = env.register(Contract, ());
    env.as_contract(&contract_id, || {
        // Initialize contract
        Contract::initialize(env.clone(), admin.to_string()).unwrap();

        // Test getting position for user who hasn't deposited
        let result = Contract::get_position(env.clone(), user.to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProtocolError::PositionNotFound);
    });
}

#[test]
fn test_oracle_set_heartbeat_ttl_admin_only() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = TestUtils::create_admin_address(&env);
    let user = TestUtils::create_user_address(&env, 0);

    let contract_id = env.register(Contract, ());
    env.as_contract(&contract_id, || {
        // Initialize contract
        Contract::initialize(env.clone(), admin.to_string()).unwrap();

        // Test admin can set heartbeat_ttl (this would need oracle functions in main contract)
        // This test would require Oracle functions to be exposed through Contract interface
    });
}

#[test]
fn test_oracle_set_mode_admin_only() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = TestUtils::create_admin_address(&env);
    let user = TestUtils::create_user_address(&env, 0);

    let contract_id = env.register(Contract, ());
    env.as_contract(&contract_id, || {
        // Initialize contract
        Contract::initialize(env.clone(), admin.to_string()).unwrap();

        // Test admin can set mode (this would need oracle functions in main contract)
        // This test would require Oracle functions to be exposed through Contract interface
    });
}

#[test]
fn test_admin_role_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = TestUtils::create_admin_address(&env);
    let user = TestUtils::create_user_address(&env, 0);

    let contract_id = env.register(Contract, ());
    env.as_contract(&contract_id, || {
        // Initialize contract
        Contract::initialize(env.clone(), admin.to_string()).unwrap();

        // Bootstrap users with different roles
        UserManager::bootstrap_admin(&env, &admin);

        // Test admin can perform admin-only operations
        let result = Contract::set_min_collateral_ratio(env.clone(), admin.to_string(), 150);
        assert!(result.is_ok());

        // Test non-admin cannot perform admin-only operations
        let result = Contract::set_min_collateral_ratio(env.clone(), user.to_string(), 200);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProtocolError::Unauthorized);
    });
}

#[test]
fn test_manager_role_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = TestUtils::create_admin_address(&env);
    let manager = TestUtils::create_user_address(&env, 0);

    let contract_id = env.register(Contract, ());
    env.as_contract(&contract_id, || {
        // Initialize contract
        Contract::initialize(env.clone(), admin.to_string()).unwrap();

        // Bootstrap users with different roles
        UserManager::bootstrap_admin(&env, &admin);

        // Set manager role for manager user
        UserManager::set_role(&env, &admin, &manager, UserRole::Manager).unwrap();

        // Test manager can perform manager-level operations (user management)
        let result = Contract::set_user_role(
            env.clone(),
            manager.to_string(),
            manager.clone(),
            UserRole::Standard,
        );
        assert!(result.is_ok());

        // Test manager cannot escalate to admin role
        let result = Contract::set_user_role(
            env.clone(),
            manager.to_string(),
            manager.clone(),
            UserRole::Admin,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProtocolError::Unauthorized);
    });
}

#[test]
fn test_analyst_role_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = TestUtils::create_admin_address(&env);
    let analyst = TestUtils::create_user_address(&env, 0);

    let contract_id = env.register(Contract, ());
    env.as_contract(&contract_id, || {
        // Initialize contract
        Contract::initialize(env.clone(), admin.to_string()).unwrap();

        // Bootstrap users with different roles
        UserManager::bootstrap_admin(&env, &admin);

        // Set analyst role for analyst user
        UserManager::set_role(&env, &admin, &analyst, UserRole::Analyst).unwrap();

        // Test analyst can perform verification operations
        let result = Contract::set_user_verification(
            env.clone(),
            analyst.to_string(),
            analyst.clone(),
            VerificationStatus::Verified,
        );
        assert!(result.is_ok());

        // Test analyst cannot perform admin operations
        let result = Contract::set_min_collateral_ratio(env.clone(), analyst.to_string(), 200);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProtocolError::Unauthorized);
    });
}

#[test]
fn test_role_escalation_prevention() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = TestUtils::create_admin_address(&env);
    let manager = TestUtils::create_user_address(&env, 0);

    let contract_id = env.register(Contract, ());
    env.as_contract(&contract_id, || {
        // Initialize contract
        Contract::initialize(env.clone(), admin.to_string()).unwrap();

        // Bootstrap users with different roles
        UserManager::bootstrap_admin(&env, &admin);

        // Set manager role for manager user
        UserManager::set_role(&env, &admin, &manager, UserRole::Manager).unwrap();

        // Test manager cannot escalate user to admin role (only admin can set admin)
        let result = Contract::set_user_role(
            env.clone(),
            manager.to_string(),
            manager.clone(),
            UserRole::Admin,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ProtocolError::Unauthorized);

        // Only admin can set admin role
        let result = Contract::set_user_role(
            env.clone(),
            admin.to_string(),
            manager.clone(),
            UserRole::Admin,
        );
        assert!(result.is_ok());
    });
}
