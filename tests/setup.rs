use pinocchio_escrow;
use std::path::PathBuf;

use litesvm::LiteSVM;
use litesvm_token::{
    spl_token::{self},
    CreateAssociatedTokenAccount, CreateMint, MintTo,
};
use solana_sdk::{
    message::{Instruction, Message},
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};

pub const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
pub const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

pub fn program_id() -> Pubkey {
    Pubkey::from(pinocchio_escrow::ID)
}

pub struct TestContext {
    pub svm: LiteSVM,
    pub maker: Keypair,
    pub taker: Keypair,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub maker_ata_a: Pubkey,
    pub maker_ata_b: Pubkey,
    pub taker_ata_a: Pubkey,
    pub taker_ata_b: Pubkey,
    pub escrow: Pubkey,
    pub escrow_bump: u8,
    pub escrow_ata_a: Pubkey,
    pub associated_token_program: Pubkey,
    pub system_program: Pubkey,
}

pub fn setup() -> TestContext {
    let mut svm = LiteSVM::new();
    let maker = Keypair::new();
    let taker = Keypair::new();

    svm.airdrop(&maker.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Airdrop failed for maker");
    svm.airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Airdrop failed for taker");

    // Load program SO file
    let so_path = PathBuf::from(
            "/Users/karthikeya/Documents/Security-Research/Learning/Turbin3 /Karthikeya_Q126Accel_Work/pinocchio-escrow/target/sbpf-solana-solana/release/pinocchio_escrow.so",
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

    let escrow_ata_a = spl_associated_token_account::get_associated_token_address(&escrow, &mint_a);

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

pub fn send_transaction(svm: &mut LiteSVM, ix: Instruction, signers: &[&Keypair], payer: &Pubkey) {
    let message = Message::new(&[ix], Some(payer));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(signers, message, recent_blockhash);
    let tx = svm
        .send_transaction(transaction)
        .expect("Transaction should succeed");
    println!("CUs Consumed: {}", tx.compute_units_consumed);
}
