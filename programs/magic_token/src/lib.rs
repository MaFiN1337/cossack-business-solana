use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface, MintTo, mint_to};

declare_id!("DusrqDQ7sK5mkSfztr4ZQaagCQz3v1XNSzCYH9uVWuvR");

#[program]
pub mod magic_token {
    use super::*;

    pub fn mint_to_player(ctx: Context<MintMagicToken>, amount: u64) -> Result<()> {
        
        let seeds = &[b"mint_authority".as_ref(), &[ctx.bumps.mint_authority]];
        let signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.player_token_account.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer,
        );

        mint_to(cpi_ctx, amount)?;
        msg!("Нараховано {} MagicTokens гравцею", amount);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct MintMagicToken<'info> {
    #[account(mut)]
    pub token_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: PDA, що має право мінтити токени
    #[account(seeds = [b"mint_authority"], bump)]
    pub mint_authority: AccountInfo<'info>,
    pub token_program: Interface<'info, TokenInterface>,
}