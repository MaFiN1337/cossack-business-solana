use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface, MintTo, mint_to};

declare_id!("DusrqDQ7sK5mkSfztr4ZQaagCQz3v1XNSzCYH9uVWuvR");

/// програма для керування випуском ігрової валюти magictoken
#[program]
pub mod magic_token {
    use super::*;

    /// випуск магічних токенів на акаунт гравця через cpi виклик
    pub fn mint_to_player(ctx: Context<MintMagicToken>, amount: u64) -> Result<()> {
        
        // підготовка підписів pda для авторизації випуску токенів
        let seeds = &[b"mint_authority".as_ref(), &[ctx.bumps.mint_authority]];
        let signer = &[&seeds[..]];

        // виконання cpi виклику до програми токенів для мінту
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

/// перелік акаунтів для інструкції випуску магічних токенів
#[derive(Accounts)]
pub struct MintMagicToken<'info> {
    /// мінт токена магічної валюти
    #[account(mut)]
    pub token_mint: InterfaceAccount<'info, Mint>,
    
    /// токенний акаунт гравця куди зараховуються кошти
    #[account(mut)]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,
    
    /// CHECK: pda акаунт з правами мінт-авториті
    #[account(seeds = [b"mint_authority"], bump)]
    pub mint_authority: AccountInfo<'info>,
    
    /// посилання на програму токенів spl
    pub token_program: Interface<'info, TokenInterface>,
}