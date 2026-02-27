pub mod setup;

pub use setup::*;

use solana_sdk::message::{AccountMeta, Instruction};
use solana_sdk::signer::Signer;

pub fn make_instruction(ctx: &mut TestContext, v2: bool) {
    let amount_to_receive: u64 = 100_000_000;
    let amount_to_give: u64 = 500_000_000;
    let amount_to_receive_bytes: [u8; 64] = {
        let mut arr = [0u8; 64];
        arr[..8].copy_from_slice(&amount_to_receive.to_le_bytes());
        arr
    };

    let amount_to_give_bytes: [u8; 64] = {
        let mut arr = [0u8; 64];
        arr[..8].copy_from_slice(&amount_to_give.to_le_bytes());
        arr
    };

    let make_data = [
        if v2 { vec![3u8] } else { vec![0u8] }, // Make discriminator
        ctx.escrow_bump.to_le_bytes().to_vec(),
        if v2 {
            amount_to_receive_bytes.to_vec()
        } else {
            amount_to_receive.to_le_bytes().to_vec()
        },
        if v2 {
            amount_to_give_bytes.to_vec()
        } else {
            amount_to_give.to_le_bytes().to_vec()
        },
    ]
    .concat();

    let make_ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(ctx.maker.pubkey(), true),
            AccountMeta::new(ctx.escrow, false),
            AccountMeta::new(ctx.mint_a, false),
            AccountMeta::new(ctx.mint_b, false),
            AccountMeta::new(ctx.maker_ata_a, false),
            AccountMeta::new(ctx.escrow_ata_a, false),
            AccountMeta::new(ctx.system_program, false),
            AccountMeta::new(TOKEN_PROGRAM_ID, false),
            AccountMeta::new(ctx.associated_token_program, false),
        ],
        data: make_data,
    };

    // need to extract pubkey before passing ctx.svm as mut borrow
    let maker_pubkey = ctx.maker.pubkey();

    send_transaction(&mut ctx.svm, make_ix, &[&ctx.maker], &maker_pubkey);
    println!("Make transaction Succeeded");
}

pub fn take_instruction(ctx: &mut TestContext) {
    let take_data = vec![1u8]; // Take discriminator

    let take_ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(ctx.taker.pubkey(), true),
            AccountMeta::new(ctx.maker.pubkey(), false),
            AccountMeta::new(ctx.escrow, false),
            AccountMeta::new(ctx.mint_a, false),
            AccountMeta::new(ctx.mint_b, false),
            AccountMeta::new(ctx.taker_ata_a, false),
            AccountMeta::new(ctx.taker_ata_b, false),
            AccountMeta::new(ctx.escrow_ata_a, false),
            AccountMeta::new(ctx.maker_ata_b, false),
            AccountMeta::new(TOKEN_PROGRAM_ID, false),
            AccountMeta::new(ctx.system_program, false),
            AccountMeta::new(ctx.associated_token_program, false),
        ],
        data: take_data,
    };

    let taker_pubkey = ctx.taker.pubkey();
    send_transaction(&mut ctx.svm, take_ix, &[&ctx.taker], &taker_pubkey);
    println!("Take transaction Succeeded");
}

pub fn refund_instruction(ctx: &mut TestContext) {
    let refund_data = vec![2u8];

    let refund_ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(ctx.maker.pubkey(), true),
            AccountMeta::new(ctx.escrow, false),
            AccountMeta::new(ctx.maker_ata_a, false),
            AccountMeta::new(ctx.escrow_ata_a, false),
            AccountMeta::new(TOKEN_PROGRAM_ID, false),
            AccountMeta::new(ctx.system_program, false),
            AccountMeta::new(ctx.associated_token_program, false),
        ],
        data: refund_data,
    };

    let maker_pubkey = ctx.maker.pubkey();

    send_transaction(&mut ctx.svm, refund_ix, &[&ctx.maker], &maker_pubkey);
    println!("Refund transaction Succeeded");
}

#[cfg(test)]
mod tests {
    use crate::{make_instruction, refund_instruction, setup, take_instruction};

    #[test]
    pub fn test_make_instruction() {
        let mut ctx = setup();
        make_instruction(&mut ctx,false);
    }

    #[test]
    pub fn test_take_instruction() {
        let mut ctx = setup();
        make_instruction(&mut ctx,true);
        take_instruction(&mut ctx);
    }

    #[test]
    pub fn test_refund_instruction() {
        let mut ctx = setup();
        make_instruction(&mut ctx,true);
        refund_instruction(&mut ctx);
    }
}
