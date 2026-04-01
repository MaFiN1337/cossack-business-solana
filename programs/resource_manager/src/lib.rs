use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

declare_id!("4PFu4dsdPuDisbSjUCRjufg8xFysRWNADKViwW43Ch5R");

#[program]
pub mod resource_manager {
    use super::*;

    pub fn initialize_game(ctx: Context<InitializeGame>) -> Result<()> {
        let game_config = &mut ctx.accounts.game_config;
        game_config.admin = ctx.accounts.admin.key();
        game_config.bump = ctx.bumps.game_config;
        game_config.item_prices = [10, 50, 100, 200];
        msg!("Гру успішно ініціалізовано");
        Ok(())
    }

    pub fn create_resource_mint(ctx: Context<CreateResourceMint>, resource_id: u8) -> Result<()> {
        require!(resource_id < 6, ErrorCode::InvalidResourceId);

        let game_config = &mut ctx.accounts.game_config;
        game_config.resource_mints[resource_id as usize] = ctx.accounts.resource_mint.key();

        msg!("Друкарський верстат для ресурсу {} успішно створено", resource_id);
        Ok(())
    }

    pub fn mint_resource(ctx: Context<MintResource>, resource_id: u8, amount: u64) -> Result<()> {
        let game_config = &ctx.accounts.game_config;

        require!(
            game_config.resource_mints[resource_id as usize] == ctx.accounts.resource_mint.key(),
            ErrorCode::InvalidResourceMint
        );

        let seeds = &[
            b"game_config".as_ref(),
            &[game_config.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = anchor_spl::token_interface::MintTo {
            mint: ctx.accounts.resource_mint.to_account_info(),
            to: ctx.accounts.player_token_account.to_account_info(),
            authority: game_config.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        anchor_spl::token_interface::mint_to(cpi_ctx, amount)?;

        msg!("Успішно видано {} одиниць ресурсу ID: {}", amount, resource_id);
        Ok(())
    }

    pub fn burn_resource(ctx: Context<BurnResource>, resource_id: u8, amount: u64) -> Result<()> {
        let game_config = &ctx.accounts.game_config;

        require!(
            game_config.resource_mints[resource_id as usize] == ctx.accounts.resource_mint.key(),
            ErrorCode::InvalidResourceMint
        );

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = anchor_spl::token_interface::Burn {
            mint: ctx.accounts.resource_mint.to_account_info(),
            from: ctx.accounts.player_token_account.to_account_info(),
            authority: ctx.accounts.player.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        anchor_spl::token_interface::burn(cpi_ctx, amount)?;

        msg!("Успішно спалено {} одиниць ресурсу ID: {}", amount, resource_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeGame<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + 32 + (32 * 6) + 32 + (8 * 4) + 1,
        seeds = [b"game_config"],
        bump
    )]
    pub game_config: Account<'info, GameConfig>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct CreateResourceMint<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"game_config"],
        bump = game_config.bump
    )]
    pub game_config: Account<'info, GameConfig>,

    #[account(
        init_if_needed,
        payer = admin,
        mint::decimals = 0,
        mint::authority = game_config,
        mint::token_program = token_program
    )]
    pub resource_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct MintResource<'info> {
    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump
    )]
    pub game_config: Account<'info, GameConfig>,

    #[account(mut)]
    pub resource_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = player,
        associated_token::mint = resource_mint,
        associated_token::authority = player,
        associated_token::token_program = token_program 
    )]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub player: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct BurnResource<'info> {
    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump
    )]
    pub game_config: Account<'info, GameConfig>,

    #[account(mut)]
    pub resource_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = resource_mint,
        associated_token::authority = player,
        associated_token::token_program = token_program 
    )]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub player: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

#[account]
pub struct GameConfig {
    pub admin: Pubkey,              
    pub resource_mints: [Pubkey; 6],
    pub magic_token_mint: Pubkey,   
    pub item_prices: [u64; 4],      
    pub bump: u8,                   
}

#[error_code]
pub enum ErrorCode {
    #[msg("Невірний ID ресурсу. Має бути від 0 до 5.")]
    InvalidResourceId,
    #[msg("Переданий токен не співпадає з токеном у налаштуваннях гри.")]
    InvalidResourceMint,
}