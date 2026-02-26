#![allow(unexpected_cfgs)]
use pinocchio::{
    AccountView, Address, ProgramResult, address::declare_id, entrypoint, error::ProgramError,
};

use crate::instructions::EscrowInstrctions;

pub mod instructions;
pub mod state;
pub mod tests;

entrypoint!(process_instruction);

declare_id!("1cxuHMSWBe1gkX3pC19zFcahwPGBWMA9x4SvxhBiCn3");

pub fn process_instruction(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &ID);

    //get the DESCRIMINATOR from the Instruction Data
    let (descriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;
    match EscrowInstrctions::try_from(descriminator)? {
        EscrowInstrctions::Make => instructions::make::process_make_instruction(accounts, data)?,
        _ => return Err(ProgramError::InvalidInstructionData),
    };
    Ok(())
}
