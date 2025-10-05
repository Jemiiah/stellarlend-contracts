use crate::{ProtocolError, ProtocolEvent, ReentrancyGuard};
use soroban_sdk::{vec, Address, Env, IntoVal, Symbol};

#[allow(dead_code)]
pub struct FlashLoan;

impl FlashLoan {
    pub fn _execute(
        env: &Env,
        initiator: &Address,
        asset: &Address,
        amount: i128,
        fee_bps: i128,
        receiver_contract: &Address,
    ) -> Result<(), ProtocolError> {
        if amount <= 0 {
            return Err(ProtocolError::InvalidAmount);
        }
        ReentrancyGuard::enter(env)?;
        let result = {
            let fee = (amount * fee_bps) / 10000;
            ProtocolEvent::FlashLoanInitiated(initiator.clone(), asset.clone(), amount, fee)
                .emit(env);
            let args = vec![
                env,
                asset.clone().into_val(env),
                amount.into_val(env),
                fee.into_val(env),
                initiator.clone().into_val(env),
            ];
            let _: () =
                env.invoke_contract(receiver_contract, &Symbol::new(env, "on_flash_loan"), args);
            ProtocolEvent::FlashLoanCompleted(initiator.clone(), asset.clone(), amount, fee)
                .emit(env);
            Ok(())
        };
        ReentrancyGuard::exit(env);
        result
    }
}
