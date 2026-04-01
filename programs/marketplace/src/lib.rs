use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface, Burn, burn};
use resource_manager::GameConfig;

declare_id!("BicWhdttM2dX1ENj7GC6kP4JJhbMoynAvjVmfmHzGfwN");

#[program]
pub mod marketplace {
    use super::*;

    pub fn sell_item(ctx: Context<SellItem>, item_type: u8) -> Result<()> {
        let config = &ctx.accounts.game_config;
        let price = config.item_prices[item_type as usize];

        let cpi_burn_accounts = Burn {
            mint: ctx.accounts.nft_mint.to_account_info(),
            from: ctx.accounts.player_nft_account.to_account_info(),
            authority: ctx.accounts.player.to_account_info(),
        };
        let cpi_burn_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            cpi_burn_accounts
        );
        burn(cpi_burn_ctx, 1)?;

        let cpi_mint_program = ctx.accounts.magic_token_program.to_account_info();
        
        let cpi_mint_accounts = magic_token::cpi::accounts::MintMagicToken {
            token_mint: ctx.accounts.magic_token_mint.to_account_info(),
            player_token_account: ctx.accounts.player_magic_token_account.to_account_info(),
            mint_authority: ctx.accounts.magic_token_authority.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };
        
        let cpi_mint_ctx = CpiContext::new(cpi_mint_program, cpi_mint_accounts);
        
        magic_token::cpi::mint_to_player(cpi_mint_ctx, price)?;

        msg!("Предмет продано за {} MagicTokens", price);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SellItem<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    
    pub game_config: Account<'info, GameConfig>,

    #[account(mut)]
    pub nft_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub player_nft_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub magic_token_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub player_magic_token_account: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: Authority з програми magic_token
    pub magic_token_authority: AccountInfo<'info>,

    pub magic_token_program: Program<'info, magic_token::program::MagicToken>,
    pub token_program: Interface<'info, TokenInterface>,
}