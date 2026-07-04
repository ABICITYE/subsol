use anchor_lang::prelude::*;
use anchor_spl::token::{self, Approve, Revoke, Token, TokenAccount, Transfer};

declare_id!("HmYLKgRmZDAUG1tDPJ5xf55ut664EioACQQSyBAbA8MB");

#[program]
pub mod subsol {
    use super::*;

    pub fn create_subscription(
        ctx: Context<CreateSubscription>,
        amount: u64,
        period_seconds: i64,
    ) -> Result<()> {
        let subscription = &mut ctx.accounts.subscription;
        subscription.merchant = ctx.accounts.merchant.key();
        subscription.subscriber = ctx.accounts.subscriber.key();
        subscription.amount = amount;
        subscription.period_seconds = period_seconds;
        subscription.last_paid = Clock::get()?.unix_timestamp;
        subscription.bump = ctx.bumps.subscription;

        let cpi_accounts = Approve {
            to: ctx.accounts.subscriber_token_account.to_account_info(),
            delegate: ctx.accounts.subscription.to_account_info(),
            authority: ctx.accounts.subscriber.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.key(), cpi_accounts);
        token::approve(cpi_ctx, amount * 12)?;

        msg!("Subscription created and delegate approved for {}", ctx.accounts.subscriber.key());
        Ok(())
    }

    pub fn process_payment(ctx: Context<ProcessPayment>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;

        require!(
            now >= ctx.accounts.subscription.last_paid + ctx.accounts.subscription.period_seconds,
            SubSolError::TooEarly
        );

        let merchant_key = ctx.accounts.subscription.merchant;
        let subscriber_key = ctx.accounts.subscription.subscriber;
        let bump = ctx.accounts.subscription.bump;
        let amount = ctx.accounts.subscription.amount;

        let seeds = &[
            b"subscription",
            merchant_key.as_ref(),
            subscriber_key.as_ref(),
            &[bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.subscriber_token_account.to_account_info(),
            to: ctx.accounts.merchant_token_account.to_account_info(),
            authority: ctx.accounts.subscription.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            cpi_accounts,
            signer,
        );
        token::transfer(cpi_ctx, amount)?;

        let subscription = &mut ctx.accounts.subscription;
        subscription.last_paid = now;
        msg!("Payment of {} processed at {}", amount, now);
        Ok(())
    }

    pub fn cancel_subscription(ctx: Context<CancelSubscription>) -> Result<()> {
        let cpi_accounts = Revoke {
            source: ctx.accounts.subscriber_token_account.to_account_info(),
            authority: ctx.accounts.subscriber.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.key(), cpi_accounts);
        token::revoke(cpi_ctx)?;

        msg!("Subscription cancelled and delegate revoked for {}", ctx.accounts.subscriber.key());
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateSubscription<'info> {
    #[account(
        init,
        payer = subscriber,
        space = 8 + 32 + 32 + 8 + 8 + 8 + 1,
        seeds = [b"subscription", merchant.key().as_ref(), subscriber.key().as_ref()],
        bump
    )]
    pub subscription: Account<'info, Subscription>,

    /// CHECK: just recording this wallet's address
    pub merchant: UncheckedAccount<'info>,

    #[account(mut)]
    pub subscriber: Signer<'info>,

    #[account(mut)]
    pub subscriber_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ProcessPayment<'info> {
    #[account(
        mut,
        seeds = [b"subscription", subscription.merchant.as_ref(), subscription.subscriber.as_ref()],
        bump = subscription.bump
    )]
    pub subscription: Account<'info, Subscription>,

    #[account(mut)]
    pub subscriber_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub merchant_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CancelSubscription<'info> {
    #[account(
        mut,
        close = subscriber,
        seeds = [b"subscription", subscription.merchant.as_ref(), subscription.subscriber.as_ref()],
        bump = subscription.bump,
        has_one = subscriber
    )]
    pub subscription: Account<'info, Subscription>,

    #[account(mut)]
    pub subscriber: Signer<'info>,

    #[account(mut)]
    pub subscriber_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Subscription {
    pub merchant: Pubkey,
    pub subscriber: Pubkey,
    pub amount: u64,
    pub period_seconds: i64,
    pub last_paid: i64,
    pub bump: u8,
}

#[error_code]
pub enum SubSolError {
    #[msg("Payment period hasn't elapsed yet")]
    TooEarly,
}
