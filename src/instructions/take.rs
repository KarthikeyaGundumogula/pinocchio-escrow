use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, ProgramResult,
};

use crate::state::Escrow;

pub fn process_take_instruction(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    // Account destructuring
    let [taker, maker, escrow_acc, mint_a, mint_b, taker_ata_a, taker_ata_b, escrow_ata_a, maker_ata_b, _token_program, _system_program, _associated_token_program @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Account validation
    // We are not validating the pda because we are checking the maker against the maker ata
    // so it will be fine if the maker in the escrow gets his stated tokens to him
    let (amount_to_receive, amount_to_give, bump) = {
        let escrow_state = Escrow::from_account_info(escrow_acc)?;
        let taker_ata_a_state =
            pinocchio_token::state::TokenAccount::from_account_view(taker_ata_a)?;
        let taker_ata_b_state =
            pinocchio_token::state::TokenAccount::from_account_view(taker_ata_b)?;
        let maker_ata_b_state =
            pinocchio_token::state::TokenAccount::from_account_view(maker_ata_b)?;

        if *maker_ata_b_state.owner() != escrow_state.maker()
            || taker_ata_a_state.mint() != mint_a.address()
            || taker_ata_b_state.mint() != mint_b.address()
            || *maker.address() != escrow_state.maker()
        {
            return Err(ProgramError::InvalidAccountData);
        }

        let amount_to_receive = escrow_state.amount_to_receive();
        let amount_to_give = escrow_state.amount_to_give();
        let bump = escrow_state.bump;

        (amount_to_receive, amount_to_give, bump)
    };

    // Build seeds for PDA signing
    let binding = [bump.to_le()];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.address().as_array()),
        Seed::from(&binding),
    ];
    let seeds = Signer::from(&seed);

    // Taker sends amount_to_receive to maker
    pinocchio_token::instructions::Transfer {
        from: taker_ata_b,
        to: maker_ata_b,
        authority: taker,
        amount: amount_to_receive,
    }
    .invoke()?;

    // Escrow sends amount_to_give to taker
    pinocchio_token::instructions::Transfer {
        from: escrow_ata_a,
        to: taker_ata_a,
        authority: escrow_acc,
        amount: amount_to_give,
    }
    .invoke_signed(&[seeds.clone()])?;

    // Close the vault token account, lamports go back to maker
    pinocchio_token::instructions::CloseAccount {
        account: escrow_ata_a,
        destination: maker,
        authority: escrow_acc,
    }
    .invoke_signed(&[seeds.clone()])?;

    Ok(())
}
