pub mod make;
pub mod take;
pub mod refund;

pub use make::*;
use pinocchio::error::ProgramError;

pub enum EscrowInstrctions {
    Make = 0,
    Take = 1,
    Refund = 2,
}

impl TryFrom<&u8> for EscrowInstrctions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EscrowInstrctions::Make),
            1 => Ok(EscrowInstrctions::Take),
            2 => Ok(EscrowInstrctions::Refund),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}