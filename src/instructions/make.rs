use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, rent::Rent},
};
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;

use crate::state::Escrow;

pub fn process_make_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [
        maker,
        escrow_acc,
        mint_a,
        mint_b,
        maker_ata,
        escrow_ata,
        system_program,
        token_program,
        _assoociated_token_program @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let maker_ata_state = pinocchio_token::state::TokenAccount::from_account_view(maker_ata)?;
    // PDA Derivation
    let bump = data[0];
    let seed = [b"escrow".as_ref(), maker.address().as_ref(), &[bump]];
    let escrow_acc_pda = derive_address(&seed, None, &crate::ID.to_bytes());

    // Account validation
    if maker_ata_state.owner() != maker.address() {
        return Err(ProgramError::IllegalOwner);
    }
    if maker_ata_state.mint() != mint_a.address() {
        return Err(ProgramError::InvalidAccountData);
    }
    assert_eq!(escrow_acc_pda, *escrow_acc.address().as_array());

    // -- layout --//
    // 0 - descriminator but stripped at the entry point's process instruction
    // 1 - bump
    // 1-8 amount to recive
    // 9-16 amount to give 
    // 

    let amount_to_receive = unsafe { *(data.as_ptr().add(1) as *const u64) }; // here we are starting from first byte because first byte is actually the bump 
    let amount_to_give = unsafe { *(data.as_ptr().add(9) as *const u64) };

    let bump = [bump.to_le()];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    // state change
    unsafe {
        if escrow_acc.owner() != &crate::ID {
            CreateAccount {
                from: maker,
                to: escrow_acc,
                lamports: Rent::get()?.try_minimum_balance(Escrow::LEN)?,
                space: Escrow::LEN as u64,
                owner: &crate::ID,
            }
            .invoke_signed(&[seeds.clone()])?;

            {
                let escrow_state = Escrow::from_account_info(escrow_acc)?;

                escrow_state.set_maker(maker.address());
                escrow_state.set_mint_a(mint_a.address());
                escrow_state.set_mint_b(mint_b.address());
                escrow_state.set_amount_to_receive(amount_to_receive);
                escrow_state.set_amount_to_give(amount_to_give);
                escrow_state.bump = data[0];
            }
        } else {
            return Err(ProgramError::IllegalOwner);
        }
    }

    pinocchio_associated_token_account::instructions::Create {
      funding_account: maker,
      account:escrow_ata,
      wallet: escrow_acc,
      mint: mint_a,
      system_program: system_program,
      token_program: token_program
    }.invoke()?;

    pinocchio_token::instructions::Transfer{
      from:maker_ata,
      to: escrow_ata,
      authority:maker,
      amount:amount_to_give
    }.invoke()?;

    Ok(())
}
