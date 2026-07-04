use anchor_lang::prelude::*;

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

        msg!("Subscription created for {}", ctx.accounts.subscriber.key());
        Ok(())
    }

    pub fn process_payment(ctx: Context<ProcessPayment>) -> Result<()> {
        let subscription = &mut ctx.accounts.subscription;
        let now = Clock::get()?.unix_timestamp;

        require!(
            now >= subscription.last_paid + subscription.period_seconds,
            SubSolError::TooEarly
        );

        subscription.last_paid = now;
        msg!("Payment processed at {}", now);
        Ok(())
    }

    pub fn cancel_subscription(ctx: Context<CancelSubscription>) -> Result<()> {
        msg!("Subscription cancelled for {}", ctx.accounts.subscriber.key());
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
