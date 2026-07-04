use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

#[test]
fn test_create_subscription() {
    let mut svm = LiteSVM::new();

    svm.add_program_from_file(subsol_program::ID, "/workspaces/subsol/subsol_program/target/deploy/subsol_program.so")
        .unwrap();

    let merchant = Keypair::new();
    let subscriber = Keypair::new();

    svm.airdrop(&subscriber.pubkey(), 1_000_000_000).unwrap();

    let (subscription_pda, _bump) = Pubkey::find_program_address(
        &[
            b"subscription",
            merchant.pubkey().as_ref(),
            subscriber.pubkey().as_ref(),
        ],
        &subsol_program::ID,
    );

    let accounts = subsol_program::accounts::CreateSubscription {
        subscription: subscription_pda,
        merchant: merchant.pubkey(),
        subscriber: subscriber.pubkey(),
        system_program: anchor_lang::system_program::ID,
    }
    .to_account_metas(None);

    let data = subsol_program::instruction::CreateSubscription {
        amount: 10_000_000,
        period_seconds: 2_592_000,
    }
    .data();

    let ix = Instruction {
        program_id: subsol_program::ID,
        accounts,
        data,
    };

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&subscriber.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&subscriber]).unwrap();

    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "transaction failed: {:?}", result.err());

    let account = svm
        .get_account(&subscription_pda)
        .expect("subscription account should exist");
    let subscription =
        subsol_program::Subscription::try_deserialize(&mut account.data.as_slice()).unwrap();

    assert_eq!(subscription.merchant, merchant.pubkey());
    assert_eq!(subscription.subscriber, subscriber.pubkey());
    assert_eq!(subscription.amount, 10_000_000);
    assert_eq!(subscription.period_seconds, 2_592_000);

    println!("Subscription created and verified!");
}
