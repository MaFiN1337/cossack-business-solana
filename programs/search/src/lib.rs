use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

declare_id!("BSMa3VZ7xjFrbVBzhv2bGd6PAsiaHYWzwiGnx3xwYfdb");

#[program]
pub mod search {
    use super::*;

    pub fn init_player(ctx: Context<InitPlayer>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        player.owner = ctx.accounts.owner.key();
        player.last_search_timestamp = 0;
        player.bump = ctx.bumps.player;
        
        msg!("Профіль гравця успішно створено.");
        Ok(())
    }

    pub fn search_resources(ctx: Context<SearchResources>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        require!(
            current_time - player.last_search_timestamp >= 60,
            ErrorCode::CooldownNotPassed
        );

        player.last_search_timestamp = current_time;

        let random_seed = current_time.wrapping_add(clock.slot as i64);
        
        let res1 = (random_seed % 6) as u8;
        let res2 = ((random_seed / 2) % 6) as u8;
        let res3 = ((random_seed / 3) % 6) as u8;

        msg!("Генерація успішна. Знайдені ресурси: {}, {}, {}", res1, res2, res3);

        let mints = [
            ctx.accounts.mint_0.to_account_info(), ctx.accounts.mint_1.to_account_info(),
            ctx.accounts.mint_2.to_account_info(), ctx.accounts.mint_3.to_account_info(),
            ctx.accounts.mint_4.to_account_info(), ctx.accounts.mint_5.to_account_info(),
        ];
        
        let atas = [
            ctx.accounts.ata_0.to_account_info(), ctx.accounts.ata_1.to_account_info(),
            ctx.accounts.ata_2.to_account_info(), ctx.accounts.ata_3.to_account_info(),
            ctx.accounts.ata_4.to_account_info(), ctx.accounts.ata_5.to_account_info(),
        ];

        let results = [res1, res2, res3];

        for res_id in results {
            let cpi_program = ctx.accounts.resource_manager_program.to_account_info();
            
            let cpi_accounts = resource_manager::cpi::accounts::MintResource {
                game_config: ctx.accounts.game_config.to_account_info(),
                resource_mint: mints[res_id as usize].clone(),
                player_token_account: atas[res_id as usize].clone(),
                player: ctx.accounts.owner.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
            };
            
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            resource_manager::cpi::mint_resource(cpi_ctx, res_id, 1)?;
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitPlayer<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 8 + 1,
        seeds = [b"player", owner.key().as_ref()],
        bump
    )]
    pub player: Account<'info, Player>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SearchResources<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"player", owner.key().as_ref()],
        bump = player.bump,
        has_one = owner
    )]
    pub player: Account<'info, Player>,

    #[account(mut)]
    pub game_config: Account<'info, resource_manager::GameConfig>,

    pub resource_manager_program: Program<'info, resource_manager::program::ResourceManager>,

    #[account(mut)] pub mint_0: InterfaceAccount<'info, Mint>,
    #[account(mut)] pub mint_1: InterfaceAccount<'info, Mint>,
    #[account(mut)] pub mint_2: InterfaceAccount<'info, Mint>,
    #[account(mut)] pub mint_3: InterfaceAccount<'info, Mint>,
    #[account(mut)] pub mint_4: InterfaceAccount<'info, Mint>,
    #[account(mut)] pub mint_5: InterfaceAccount<'info, Mint>,

    /// CHECK: Я довіряю CPI-виклику до resource_manager, який сам перевірить цей гаманець
    #[account(mut)] pub ata_0: UncheckedAccount<'info>,
    /// CHECK: Перевіряється на стороні програми resource_manager
    #[account(mut)] pub ata_1: UncheckedAccount<'info>,
    /// CHECK: Перевіряється на стороні програми resource_manager
    #[account(mut)] pub ata_2: UncheckedAccount<'info>,
    /// CHECK: Перевіряється на стороні програми resource_manager
    #[account(mut)] pub ata_3: UncheckedAccount<'info>,
    /// CHECK: Перевіряється на стороні програми resource_manager
    #[account(mut)] pub ata_4: UncheckedAccount<'info>,
    /// CHECK: Перевіряється на стороні програми resource_manager
    #[account(mut)] pub ata_5: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Player {
    pub owner: Pubkey,
    pub last_search_timestamp: i64,
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Ще не минуло 60 секунд з минулого пошуку! Зачекайте.")]
    CooldownNotPassed,
}