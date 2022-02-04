use anchor_lang::prelude::*;
use anchor_lang::solana_program::{clock, program_option::COption, sysvar};
use anchor_spl::token::{self, Mint, Token, TokenAccount};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod dice {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>, nonce: u8) -> ProgramResult {
        let dice = &mut ctx.accounts.dice;
        dice.win_returns = 90;
        dice.token_mint = ctx.accounts.token_mint.key();
        dice.token_vault = ctx.accounts.token_vault.key();
        dice.nonce = nonce;

        Ok(())
    }

    pub fn roll(ctx: Context<Roll>, amount: u64, side: u8) -> ProgramResult {
        if side > 6 {
            return Err(ErrorCode::DiceNumberShouldBeBelowSeven.into());
        }
        if amount == 0 {
            return Err(ErrorCode::AmountMustBeGreaterThanZero.into());
        }

        let dice = &mut ctx.accounts.dice;
        let c = clock::Clock::get().unwrap();

        // Transfer tokens into the token vault.
        {
            let cpi_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.stake_from_account.to_account_info(),
                    to: ctx.accounts.token_vault.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            );
            token::transfer(cpi_ctx, amount)?;
        }

        if (c.unix_timestamp % 6) == side {
            if ctx.accounts.token_vault.amount < ((amount * (dice.win_returns as u64))/100) {
                msg!("Congratulations, You won! Sry, we didn't have enough reward to gib you. So, we'll gib you all the remaining reward in the vault");

                // Transfer tokens from the vault to user vault.
                {
                    let seeds = &[dice.to_account_info().key.as_ref(), &[dice.nonce]];
                    let pool_signer = &[&seeds[..]];

                    let cpi_ctx = CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        token::Transfer {
                            from: ctx.accounts.token_vault.to_account_info(),
                            to: ctx.accounts.stake_from_account.to_account_info(),
                            authority: ctx.accounts.pool_signer.to_account_info(),
                        },
                        pool_signer,
                    );
                    token::transfer(cpi_ctx, ctx.accounts.token_vault.amount)?;
                }
            } else {
                // Transfer tokens from the vault to user vault.
                {
                    let seeds = &[dice.to_account_info().key.as_ref(), &[dice.nonce]];
                    let pool_signer = &[&seeds[..]];

                    let cpi_ctx = CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        token::Transfer {
                            from: ctx.accounts.token_vault.to_account_info(),
                            to: ctx.accounts.stake_from_account.to_account_info(),
                            authority: ctx.accounts.pool_signer.to_account_info(),
                        },
                        pool_signer,
                    );
                    token::transfer(cpi_ctx, amount * (100 + dice.win_returns as u64)/100)?;
                }

                msg!("Congratulations, You won!");
            }
        } else {
            msg!("Sorry, You lost!");
        }

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct Initialize<'info> {
    #[account(
        zero
    )]
    pub dice: Account<'info, Dice>,

    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,

    pub token_mint: Account<'info, Mint>,
    #[account(
        constraint = token_vault.mint == token_mint.key(),
        constraint = token_vault.owner == pool_signer.key()
    )]
    pub token_vault: Account<'info, TokenAccount>,

    #[account(
        seeds = [
            dice.to_account_info().key.as_ref()
        ],
        bump = nonce,
    )]
    pub pool_signer: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct Roll<'info> {
    #[account(
        mut,
        has_one = token_vault
    )]
    pub dice: Account<'info, Dice>,

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        constraint = token_vault.owner == pool_signer.key()
    )]
    pub token_vault: Account<'info, TokenAccount>,

    // the token account of the user
    #[account(mut)]
    pub stake_from_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [
            dice.to_account_info().key.as_ref()
        ],
        bump = nonce,
    )]
    pub pool_signer: UncheckedAccount<'info>,

    // Misc.
    pub token_program: Program<'info, Token>,
}

#[account]
#[derive(Default)]
pub struct Dice {
    pub win_returns: u8,
    pub token_mint: Pubkey,
    pub token_vault: Pubkey,
    pub nonce: u8,
}

#[error]
pub enum ErrorCode {
    #[msg("Amount must be greater than zero.")]
    AmountMustBeGreaterThanZero,
    #[msg("The dice number should be below 7")]
    DiceNumberShouldBeBelowSeven,
}
