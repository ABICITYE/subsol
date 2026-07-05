# SubSol

**Recurring payments protocol on Solana.** One customer approval becomes automatic USDC subscription payments — every cycle, on time, on-chain.

🔗 **[Live landing page](https://abicitye.github.io/subsol/)** · 🔍 **[View on Solana Explorer](https://explorer.solana.com/address/HmYLKgRmZDAUG1tDPJ5xf55ut664EioACQQSyBAbA8MB?cluster=devnet)**

---

## The problem

Subscription billing is the engine of every SaaS business — and it's gated behind payment processors that won't onboard founders in much of the world. *"Stripe is not currently available in your country"* is a message thousands of African founders know by heart. The alternatives freeze funds without warning, settle in banking days, and take ~3% off the top.

**SubSol removes the gatekeeper.** Subscriptions live directly on Solana as on-chain accounts. A merchant needs a wallet, not an approval letter. A customer approves once, and USDC moves on schedule for as long as the subscription runs. Settlement is measured in seconds, and the protocol takes a flat 0.5% — nothing else.

## How it works

SubSol is three instructions:

1. **`create_subscription`** — Writes the subscription terms (amount, interval, merchant) to a PDA and approves an SPL Token delegate in a single signature. SubSol never takes custody of anyone's funds.
2. **`process_payment`** — Checks the elapsed time via Solana's `Clock` sysvar, transfers USDC to the merchant via CPI, takes the 0.5% fee, and sets the next due date.
3. **`cancel_subscription`** — Revokes the delegate, closes the account, and refunds rent. The customer can walk away anytime.

The design uses **classic SPL Token** (not Token-2022) because real USDC on Solana uses the classic standard on both mainnet and devnet.

## Status

| | |
|---|---|
| **Network** | Devnet (mainnet planned) |
| **Program ID** | `HmYLKgRmZDAUG1tDPJ5xf55ut664EioACQQSyBAbA8MB` |
| **Tests** | 3/3 passing (LiteSVM) |
| **Framework** | Anchor |

The core protocol is built, tested, and deployed to devnet. Mainnet deployment is the next milestone.

## Tech stack

- **Rust + Anchor** — on-chain program
- **SPL Token** — delegate authority for recurring transfers
- **USDC** — settlement asset
- **LiteSVM** — test runtime

## Roadmap

- [x] Core protocol: create, process, cancel
- [x] LiteSVM test suite
- [x] Devnet deployment
- [x] Landing page
- [ ] Wallet-connect demo
- [ ] Mainnet deployment

---

Built by **Oyewole Emmanuel** · University of Lagos
Solana Summer School 2026 — Capstone
