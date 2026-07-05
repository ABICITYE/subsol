use anchor_lang::AccountDeserialize;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::token::TokenAccount as SplTokenAccount;
use litesvm::LiteSVM;
use litesvm_token::{CreateAssociatedTokenAccount, CreateMint, MintTo};
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

#[test]
fn test_cancel_subscription() {
    let mut svm = LiteSVM::new();
    svm.add_program_from_file(
        subsol_program::ID,
        "/workspaces/subsol/subsol_program/target/deploy/subsol_program.so",
    )
    .unwrap();

    let merchant = Keypair::new();
    let subscriber = Keypair::new();

    svm.airdrop(&subscriber.pubkey(), 2_000_000_000).unwrap();

    let mint = CreateMint::new(&mut svm, &subscriber).decimals(6).send().unwrap();

    let subscriber_token_account = CreateAssociatedTokenAccount::new(&mut svm, &subscriber, &mint)
        .owner(&subscriber.pubkey())
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
        period_seconds: 2_592_000,
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

    let subscriber_balance_before = svm.get_balance(&subscriber.pubkey()).unwrap();

    let cancel_accounts = subsol_program::accounts::CancelSubscription {
        subscription: subscription_pda,
        subscriber: subscriber.pubkey(),
        subscriber_token_account,
        token_program: anchor_spl::token::ID,
    }
    .to_account_metas(None);

    let cancel_data = subsol_program::instruction::CancelSubscription {}.data();

    let cancel_ix = Instruction {
        program_id: subsol_program::ID,
        accounts: cancel_accounts,
        data: cancel_data,
    };

    let blockhash2 = svm.latest_blockhash();
    let msg2 = Message::new_with_blockhash(&[cancel_ix], Some(&subscriber.pubkey()), &blockhash2);
    let tx2 = VersionedTransaction::try_new(VersionedMessage::Legacy(msg2), &[&subscriber]).unwrap();
    let result2 = svm.send_transaction(tx2);
    assert!(result2.is_ok(), "cancel_subscription failed: {:?}", result2.err());

    let closed_account = svm.get_account(&subscription_pda);
    assert!(
        closed_account.is_none() || closed_account.unwrap().data.iter().all(|&b| b == 0),
        "subscription account should be closed"
    );

    let subscriber_balance_after = svm.get_balance(&subscriber.pubkey()).unwrap();
    assert!(
        subscriber_balance_after > subscriber_balance_before,
        "subscriber should get rent refund back"
    );

    let token_account = svm.get_account(&subscriber_token_account).unwrap();
    let token_data = SplTokenAccount::try_deserialize(&mut token_account.data.as_slice()).unwrap();
    assert!(token_data.delegate.is_none(), "delegate should be revoked");

    println!("Subscription cancelled — delegate revoked and rent refunded!");
}
