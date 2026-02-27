use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, ProgramResult,
};
use pinocchio_log::log;

use crate::state::Escrow;

pub fn process_refund_instruction(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    log!("enterd refund");
    let [maker, escrow_acc, maker_ata, escrow_ata, _token_program, _system_program, _associated_token_program @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::IncorrectAuthority);
    }

    let (amount_to_refund, bump) = {
        let escrow_state = Escrow::from_account_info(escrow_acc)?;
        let maker_ata_state = pinocchio_token::state::TokenAccount::from_account_view(maker_ata)?;

        if *maker_ata_state.owner() != escrow_state.maker()
            || *maker.address() != escrow_state.maker()
        {
            return Err(ProgramError::IllegalOwner);
        }

        let amount_to_refund = escrow_state.amount_to_give();
        let bump = escrow_state.bump;

        (amount_to_refund, bump)
    };

    // Build seeds for PDA signing
    let binding = [bump.to_le()];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.address().as_array()),
        Seed::from(&binding),
    ];
    let seeds = Signer::from(&seed);

    pinocchio_token::instructions::Transfer {
        from: escrow_ata,
        to: maker_ata,
        authority: escrow_acc,
        amount: amount_to_refund,
    }
    .invoke_signed(&[seeds.clone()])?;

    // Close the vault token account, lamports go back to maker
    pinocchio_token::instructions::CloseAccount {
        account: escrow_ata,
        destination: maker,
        authority: escrow_acc,
    }
    .invoke_signed(&[seeds.clone()])?;

    Ok(())
}
