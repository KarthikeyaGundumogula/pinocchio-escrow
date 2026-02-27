use ::wincode::SchemaRead;
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, ProgramResult,
};
use pinocchio_associated_token_account::instructions::Create;
use pinocchio_log::log;
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::Transfer;

use crate::{state::WincodeEscrow, ID};

#[derive(SchemaRead)]
pub struct MakeInstructionData {
    pub bump: u8,
    pub amount_to_receive: [u8; 64],
    pub amount_to_give: [u8; 64],
}

pub fn process_make_v2_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [maker, escrow_acc, mint_a, mint_b, maker_ata, escrow_ata, system_program, token_program, _assoociated_token_program @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::IncorrectAuthority);
    }

    log!("entered v2");

    let ix_data = ::wincode::deserialize::<MakeInstructionData>(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let bump = ix_data.bump;
    let amount_to_receive = ix_data.amount_to_receive;
    let amount_to_give = ix_data.amount_to_give;

    {
        let maker_ata_state = pinocchio_token::state::TokenAccount::from_account_view(maker_ata)?;
        if maker_ata_state.owner() != maker.address() {
            return Err(ProgramError::IllegalOwner);
        }
        if maker_ata_state.mint() != mint_a.address() {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    let seeds: [&[u8]; 3] = [b"escrow", maker.address().as_array(), &[bump]];
    let expected_escrow = derive_address(&seeds, None, ID.as_array());

    if escrow_acc.address().as_array() != &expected_escrow {
        return Err(ProgramError::InvalidAccountData);
    }

    let bump_seed = [bump];
    let signer_seeds = [
        Seed::from(b"escrow"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump_seed),
    ];
    let signer = Signer::from(&signer_seeds[..]);

    CreateAccount {
        from: maker,
        to: escrow_acc,
        lamports: Rent::get()?.minimum_balance_unchecked(WincodeEscrow::LEN),
        space: WincodeEscrow::LEN as u64,
        owner: &ID,
    }
    .invoke_signed(&[signer])?;

    let escrow_state = WincodeEscrow::from_account_info(escrow_acc)?;

    escrow_state.maker = *maker.address().as_array();
    escrow_state.mint_a = *mint_a.address().as_array();
    escrow_state.mint_b = *mint_b.address().as_array();
    escrow_state.amount_to_receive = amount_to_receive;
    escrow_state.amount_to_give = amount_to_give;
    escrow_state.bump = bump;

    Create {
        funding_account: maker,
        account: escrow_ata,
        wallet: escrow_acc,
        mint: mint_a,
        token_program,
        system_program,
    }
    .invoke()?;
    let amount: u64 = u64::from_le_bytes(amount_to_give[..8].try_into().unwrap());
    Transfer {
        from: maker_ata,
        to: escrow_ata,
        authority: maker,
        amount: amount,
    }
    .invoke()?;

    Ok(())
}
