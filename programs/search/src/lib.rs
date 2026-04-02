use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

declare_id!("BSMa3VZ7xjFrbVBzhv2bGd6PAsiaHYWzwiGnx3xwYfdb");

/// програма для реалізації механіки пошуку випадкових ресурсів
#[program]
pub mod search {
    use super::*;

    /// ініціалізація профілю гравця для відстеження часу останнього пошуку
    pub fn init_player(ctx: Context<InitPlayer>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        player.owner = ctx.accounts.owner.key();
        player.last_search_timestamp = 0;
        player.bump = ctx.bumps.player;
        
        msg!("Профіль гравця успішно створено.");
        Ok(())
    }

    /// генерація трьох випадкових ресурсів з перевіркою часового обмеження
    pub fn search_resources(ctx: Context<SearchResources>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        // перевірка чи минуло 60 секунд з моменту останнього виклику
        require!(
            current_time - player.last_search_timestamp >= 60,
            ErrorCode::CooldownNotPassed
        );

        player.last_search_timestamp = current_time;

        // створення псевдовипадкового значення на основі часу та слоту
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

        // нарахування знайдених ресурсів через cpi виклики до менеджера ресурсів
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

/// перелік акаунтів для створення профілю гравця
#[derive(Accounts)]
pub struct InitPlayer<'info> {
    /// власник профілю та платник за створення акаунта
    #[account(mut)]
    pub owner: Signer<'info>,

    /// pda акаунт гравця з прив'язкою до публічного ключа власника
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 8 + 1,
        seeds = [b"player", owner.key().as_ref()],
        bump
    )]
    pub player: Account<'info, Player>,

    /// системна програма solana
    pub system_program: Program<'info, System>,
}

/// перелік акаунтів для інструкції пошуку ресурсів
#[derive(Accounts)]
pub struct SearchResources<'info> {
    /// власник який ініціює пошук
    #[account(mut)]
    pub owner: Signer<'info>,

    /// pda акаунт гравця для оновлення мітки часу
    #[account(
        mut,
        seeds = [b"player", owner.key().as_ref()],
        bump = player.bump,
        has_one = owner
    )]
    pub player: Account<'info, Player>,

    /// акаунт глобальних налаштувань гри
    #[account(mut)]
    pub game_config: Account<'info, resource_manager::GameConfig>,

    /// посилання на програму менеджера ресурсів для випуску токенів
    pub resource_manager_program: Program<'info, resource_manager::program::ResourceManager>,

    #[account(mut)] pub mint_0: InterfaceAccount<'info, Mint>,
    #[account(mut)] pub mint_1: InterfaceAccount<'info, Mint>,
    #[account(mut)] pub mint_2: InterfaceAccount<'info, Mint>,
    #[account(mut)] pub mint_3: InterfaceAccount<'info, Mint>,
    #[account(mut)] pub mint_4: InterfaceAccount<'info, Mint>,
    #[account(mut)] pub mint_5: InterfaceAccount<'info, Mint>,

    /// CHECK: токенні акаунти перевіряються в cpi виклику на стороні менеджера ресурсів
    #[account(mut)] pub ata_0: UncheckedAccount<'info>,
    /// CHECK: акаунт перевіряється в програмі resource_manager
    #[account(mut)] pub ata_1: UncheckedAccount<'info>,
    /// CHECK: акаунт перевіряється в програмі resource_manager
    #[account(mut)] pub ata_2: UncheckedAccount<'info>,
    /// CHECK: акаунт перевіряється в програмі resource_manager
    #[account(mut)] pub ata_3: UncheckedAccount<'info>,
    /// CHECK: акаунт перевіряється в програмі resource_manager
    #[account(mut)] pub ata_4: UncheckedAccount<'info>,
    /// CHECK: акаунт перевіряється в програмі resource_manager
    #[account(mut)] pub ata_5: UncheckedAccount<'info>,

    /// посилання на програму токенів spl
    pub token_program: Interface<'info, TokenInterface>,
    /// програма асоційованих токенів
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    /// системна програма solana
    pub system_program: Program<'info, System>,
}

/// структура даних для зберігання стану гравця
#[account]
pub struct Player {
    /// публічний ключ власника акаунта
    pub owner: Pubkey,
    /// позначка часу останнього успішного пошуку ресурсів
    pub last_search_timestamp: i64,
    /// значення bump для pda
    pub bump: u8,
}

/// перелік помилок програми пошуку
#[error_code]
pub enum ErrorCode {
    /// помилка при спробі пошуку раніше ніж через 60 секунд
    #[msg("Ще не минуло 60 секунд з минулого пошуку! Зачекайте.")]
    CooldownNotPassed,
}