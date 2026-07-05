use anchor_lang::solana_program::clock::Clock;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use anchor_spl::token::TokenAccount as SplTokenAccount;
use litesvm::LiteSVM;
use litesvm_token::{CreateAssociatedTokenAccount, CreateMint, MintTo};
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

#[test]
fn test_process_payment() {
    let mut svm = LiteSVM::new();
    svm.add_program_from_file(
        subsol_program::ID,
        "/workspaces/subsol/subsol_program/target/deploy/subsol_program.so",
    )
    .unwrap();

    let merchant = Keypair::new();
    let subscriber = Keypair::new();

    svm.airdrop(&subscriber.pubkey(), 2_000_000_000).unwrap();
    svm.airdrop(&merchant.pubkey(), 1_000_000_000).unwrap();

    let mint = CreateMint::new(&mut svm, &subscriber).decimals(6).send().unwrap();

    let subscriber_token_account = CreateAssociatedTokenAccount::new(&mut svm, &subscriber, &mint)
        .owner(&subscriber.pubkey())
        .send()
        .unwrap();

    let merchant_token_account = CreateAssociatedTokenAccount::new(&mut svm, &merchant, &mint)
        .owner(&merchant.pubkey())
        .send()
        .unwrap();

    MintTo::new(&mut svm, &subscriber, &mint, &subscriber_token_account, 100_000_000)
        .send()
        .unwrap();

    let (subscription_pda, _bump) = Pubkey::find_program_address(
        &[b"subscription", merchant.pubkey().as_ref(), subscriber.pubkey().as_ref()],
        &subsol_program::ID,
    );

    let create_accounts = subsol_program::accounts::CreateSubscription {
        subscription: subscription_pda,
        merchant: merchant.pubkey(),
        subscriber: subscriber.pubkey(),
        subscriber_token_account,
        token_program: anchor_spl::token::ID,
        system_program: anchor_lang::system_program::ID,
    }
    .to_account_metas(None);

    let create_data = subsol_program::instruction::CreateSubscription {
        amount: 10_000_000,
        period_seconds: 1,
    }
    .data();

    let create_ix = Instruction {
        program_id: subsol_program::ID,
        accounts: create_accounts,
        data: create_data,
    };

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[create_ix], Some(&subscriber.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&subscriber]).unwrap();
    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "create_subscription failed: {:?}", result.err());

    let mut clock: Clock = svm.get_sysvar();
    clock.unix_timestamp += 10;
    svm.set_sysvar(&clock);

    let payment_accounts = subsol_program::accounts::ProcessPayment {
        subscription: subscription_pda,
        subscriber_token_account,
        merchant_token_account,
        token_program: anchor_spl::token::ID,
    }
    .to_account_metas(None);

    let payment_data = subsol_program::instruction::ProcessPayment {}.data();

    let payment_ix = Instruction {
        program_id: subsol_program::ID,
        accounts: payment_accounts,
        data: payment_data,
    };

    let blockhash2 = svm.latest_blockhash();
    let msg2 = Message::new_with_blockhash(&[payment_ix], Some(&merchant.pubkey()), &blockhash2);
    let tx2 = VersionedTransaction::try_new(VersionedMessage::Legacy(msg2), &[&merchant]).unwrap();
    let result2 = svm.send_transaction(tx2);
    assert!(result2.is_ok(), "process_payment failed: {:?}", result2.err());

    let merchant_account = svm.get_account(&merchant_token_account).unwrap();
    let merchant_token_data =
        SplTokenAccount::try_deserialize(&mut merchant_account.data.as_slice()).unwrap();
    assert_eq!(merchant_token_data.amount, 10_000_000);

    let subscriber_account = svm.get_account(&subscriber_token_account).unwrap();
    let subscriber_token_data =
        SplTokenAccount::try_deserialize(&mut subscriber_account.data.as_slice()).unwrap();
    assert_eq!(subscriber_token_data.amount, 90_000_000);

    println!("Payment processed — USDC actually moved from subscriber to merchant!");
}
