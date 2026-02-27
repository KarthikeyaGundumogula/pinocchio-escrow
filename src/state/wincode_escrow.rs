use pinocchio::{error::ProgramError, AccountView};
use wincode::SchemaRead;

#[repr(C)]
#[derive(Clone, Copy, SchemaRead)]
pub struct WincodeEscrow {
    pub maker: [u8; 32],
    pub mint_a: [u8; 32],
    pub mint_b: [u8; 32],
    pub amount_to_receive: [u8; 64],
    pub amount_to_give: [u8; 64],
    pub bump: u8,
}
impl WincodeEscrow {
    pub const LEN: usize = 32 + 32 + 32 + 8 + 8 + 1;
    pub fn from_account_info(account_info: &AccountView) -> Result<&mut Self, ProgramError> {
        let mut data = account_info.try_borrow_mut()?;
        if data.len() != WincodeEscrow::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if (data.as_ptr() as usize) % core::mem::align_of::<Self>() != 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }
}
