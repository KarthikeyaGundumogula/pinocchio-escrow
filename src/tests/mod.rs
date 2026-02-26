#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use litesvm::LiteSVM;
    use litesvm_token::{
        spl_token::{self},
        CreateAssociatedTokenAccount, CreateMint, MintTo,
    };
    use solana_sdk::{
        message::{AccountMeta, Instruction, Message},
        native_token::LAMPORTS_PER_SOL,
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        transaction::Transaction,
    };

    const PROGRAM_ID: &str = "1cxuHMSWBe1gkX3pC19zFcahwPGBWMA9x4SvxhBiCn3";
    const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
    const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

    fn program_id() -> Pubkey {
        Pubkey::from(crate::ID)
    }

    struct TestContext {
        svm: LiteSVM,
        maker: Keypair,
        taker: Keypair,
        mint_a: Pubkey,
        mint_b: Pubkey,
        maker_ata_a: Pubkey,
        maker_ata_b: Pubkey,
        taker_ata_a: Pubkey,
        taker_ata_b: Pubkey,
        escrow: Pubkey,
        escrow_bump: u8,
        escrow_ata_a: Pubkey,
        associated_token_program: Pubkey,
        system_program: Pubkey,
    }

    fn setup() -> TestContext {
        let mut svm = LiteSVM::new();
        let maker = Keypair::new();
        let taker = Keypair::new();

        svm.airdrop(&maker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed for maker");
        svm.airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed for taker");

        // Load program SO file
        let so_path = PathBuf::from(
            "/Users/karthikeya/Documents/Security-Research/Learning/Turbin3 /Karthikeya_Q126Accel_Work/pinocchio-escrow/target/sbpf-solana-solana/release/escrow.so",
        );
        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");
        svm.add_program(program_id(), &program_data)
            .expect("Failed to add program");

        // Create mints
        let mint_a = CreateMint::new(&mut svm, &maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();

        let mint_b = CreateMint::new(&mut svm, &taker)
            .decimals(6)
            .authority(&taker.pubkey())
            .send()
            .unwrap();

        // Create ATAs
        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &maker, &mint_a)
            .owner(&maker.pubkey())
            .send()
            .unwrap();

        let maker_ata_b = CreateAssociatedTokenAccount::new(&mut svm, &maker, &mint_b)
            .owner(&maker.pubkey())
            .send()
            .unwrap();

        let taker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &taker, &mint_a)
            .owner(&taker.pubkey())
            .send()
            .unwrap();

        let taker_ata_b = CreateAssociatedTokenAccount::new(&mut svm, &taker, &mint_b)
            .owner(&taker.pubkey())
            .send()
            .unwrap();

        // Derive escrow PDA and vault
        let (escrow, escrow_bump) = Pubkey::find_program_address(
            &[b"escrow".as_ref(), maker.pubkey().as_ref()],
            &program_id(),
        );

        let escrow_ata_a =
            spl_associated_token_account::get_associated_token_address(&escrow, &mint_a);

        // Mint tokens
        MintTo::new(&mut svm, &maker, &mint_a, &maker_ata_a, 1_000_000_000)
            .send()
            .unwrap();

        MintTo::new(&mut svm, &taker, &mint_b, &taker_ata_b, 1_000_000_000)
            .send()
            .unwrap();

        let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
        let system_program = Pubkey::from(pinocchio_system::ID);

        TestContext {
            svm,
            maker,
            taker,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
            escrow,
            escrow_bump,
            escrow_ata_a,
            associated_token_program,
            system_program,
        }
    }

    fn send_transaction(svm: &mut LiteSVM, ix: Instruction, signers: &[&Keypair], payer: &Pubkey) {
        let message = Message::new(&[ix], Some(payer));
        let recent_blockhash = svm.latest_blockhash();
        let transaction = Transaction::new(signers, message, recent_blockhash);
        let tx = svm
            .send_transaction(transaction)
            .expect("Transaction should succeed");
        println!("CUs Consumed: {}", tx.compute_units_consumed);
    }

    fn make_instruction(ctx: &mut TestContext) {
        let amount_to_receive: u64 = 100_000_000;
        let amount_to_give: u64 = 500_000_000;

        let make_data = [
            vec![0u8], // Make discriminator
            ctx.escrow_bump.to_le_bytes().to_vec(),
            amount_to_receive.to_le_bytes().to_vec(),
            amount_to_give.to_le_bytes().to_vec(),
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
        // Keypair doesn't implement Copy so we need to use try_sign workaround
        let message = Message::new(&[make_ix], Some(&maker_pubkey));
        let recent_blockhash = ctx.svm.latest_blockhash();
        let transaction = Transaction::new(&[&ctx.maker], message, recent_blockhash);
        let tx = ctx
            .svm
            .send_transaction(transaction)
            .expect("Make transaction should succeed");
        println!(
            "\n✅ Make transaction successful | CUs: {}",
            tx.compute_units_consumed
        );
    }

    fn take_instruction(ctx: &mut TestContext) {
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
        let message = Message::new(&[take_ix], Some(&taker_pubkey));
        let recent_blockhash = ctx.svm.latest_blockhash();
        let transaction = Transaction::new(&[&ctx.taker], message, recent_blockhash);
        let tx = ctx
            .svm
            .send_transaction(transaction)
            .expect("Take transaction should succeed");
        println!(
            "\n✅ Take transaction successful | CUs: {}",
            tx.compute_units_consumed
        );
    }

    #[test]
    pub fn test_make_instruction() {
        let mut ctx = setup();
        make_instruction(&mut ctx);
    }

    #[test]
    pub fn test_take_instruction() {
        let mut ctx = setup();
        make_instruction(&mut ctx); 
        println!("escrow owner: {:?}", ctx.svm.get_account(&ctx.escrow));
        println!("taker_ata_a: {:?}", ctx.svm.get_account(&ctx.taker_ata_a));
        take_instruction(&mut ctx);
    }
}
