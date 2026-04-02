use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

declare_id!("4PFu4dsdPuDisbSjUCRjufg8xFysRWNADKViwW43Ch5R");

/// головна програма для керування ігровими ресурсами та глобальною конфігурацією
#[program]
pub mod resource_manager {
    use super::*;

    /// ініціалізація початкових налаштувань гри та встановлення цін на предмети
    pub fn initialize_game(ctx: Context<InitializeGame>) -> Result<()> {
        let game_config = &mut ctx.accounts.game_config;
        game_config.admin = ctx.accounts.admin.key();
        game_config.bump = ctx.bumps.game_config;
        game_config.item_prices = [10, 50, 100, 200];
        msg!("Гру успішно ініціалізовано");
        Ok(())
    }

    /// створення та реєстрація нового мінту для конкретного типу ресурсу
    pub fn create_resource_mint(ctx: Context<CreateResourceMint>, resource_id: u8) -> Result<()> {
        require!(resource_id < 6, ErrorCode::InvalidResourceId);

        let game_config = &mut ctx.accounts.game_config;
        game_config.resource_mints[resource_id as usize] = ctx.accounts.resource_mint.key();

        msg!("Друкарський верстат для ресурсу {} успішно створено", resource_id);
        Ok(())
    }

    /// випуск певної кількості ресурсів на токенний акаунт гравця
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

    /// видалення ресурсів з акаунта гравця через операцію спалення
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

/// перелік акаунтів для початкової ініціалізації гри
#[derive(Accounts)]
pub struct InitializeGame<'info> {
    /// адміністратор що ініціалізує гру та сплачує за створення акаунта
    #[account(mut)]
    pub admin: Signer<'info>,

    /// акаунт pda для зберігання глобальних налаштувань
    #[account(
        init,
        payer = admin,
        space = 8 + 32 + (32 * 6) + 32 + (8 * 4) + 1,
        seeds = [b"game_config"],
        bump
    )]
    pub game_config: Account<'info, GameConfig>,

    /// системна програма solana
    pub system_program: Program<'info, System>,
}

/// перелік акаунтів для створення мінту ресурсу
#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct CreateResourceMint<'info> {
    /// адміністратор що ініціює створення мінту
    #[account(mut)]
    pub admin: Signer<'info>,

    /// акаунт налаштувань гри де реєструється мінт
    #[account(
        mut,
        seeds = [b"game_config"],
        bump = game_config.bump
    )]
    pub game_config: Account<'info, GameConfig>,

    /// акаунт мінту ресурсу що створюється
    #[account(
        init_if_needed,
        payer = admin,
        mint::decimals = 0,
        mint::authority = game_config,
        mint::token_program = token_program
    )]
    pub resource_mint: InterfaceAccount<'info, Mint>,

    /// посилання на програму токенів spl
    pub token_program: Interface<'info, TokenInterface>,
    /// системна програма solana
    pub system_program: Program<'info, System>,
}

/// акаунти необхідні для випуску ігрових ресурсів
#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct MintResource<'info> {
    /// акаунт налаштувань гри що виступає в ролі мінт-авториті
    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump
    )]
    pub game_config: Account<'info, GameConfig>,

    /// акаунт мінту ресурсу
    #[account(mut)]
    pub resource_mint: InterfaceAccount<'info, Mint>,

    /// токенний акаунт гравця для отримання ресурсів
    #[account(
        init_if_needed,
        payer = player,
        associated_token::mint = resource_mint,
        associated_token::authority = player,
        associated_token::token_program = token_program 
    )]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,

    /// гравець який отримує ресурси та сплачує за створення акаунта
    #[account(mut)]
    pub player: Signer<'info>,

    /// посилання на програму токенів spl
    pub token_program: Interface<'info, TokenInterface>,
    /// програма асоційованих токенів
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    /// системна програма solana
    pub system_program: Program<'info, System>,
}

/// акаунти необхідні для спалення ігрових ресурсів
#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct BurnResource<'info> {
    /// акаунт налаштувань гри
    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump
    )]
    pub game_config: Account<'info, GameConfig>,

    /// акаунт мінту ресурсу
    #[account(mut)]
    pub resource_mint: InterfaceAccount<'info, Mint>,

    /// токенний акаунт гравця з якого списуються ресурси
    #[account(
        mut,
        associated_token::mint = resource_mint,
        associated_token::authority = player,
        associated_token::token_program = token_program 
    )]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,

    /// гравець який підписує операцію спалення власних ресурсів
    #[account(mut)]
    pub player: Signer<'info>,

    /// посилання на програму токенів spl
    pub token_program: Interface<'info, TokenInterface>,
}

/// структура даних глобальної конфігурації гри
#[account]
pub struct GameConfig {
    /// публічний ключ адміністратора
    pub admin: Pubkey,               
    /// масив публічних ключів мінтів для всіх типів ресурсів
    pub resource_mints: [Pubkey; 6],
    /// публічний ключ мінту магічного токена
    pub magic_token_mint: Pubkey,   
    /// масив цін на ігрові предмети
    pub item_prices: [u64; 4],      
    /// значення bump для pda конфігурації
    pub bump: u8,                   
}

/// перелік помилок програми менеджера ресурсів
#[error_code]
pub enum ErrorCode {
    /// помилка при використанні невірного ідентифікатора ресурсу
    #[msg("Невірний ID ресурсу. Має бути від 0 до 5.")]
    InvalidResourceId,
    /// помилка при невідповідності мінту токена зареєстрованому в конфігурації
    #[msg("Переданий токен не співпадає з токеном у налаштуваннях гри.")]
    InvalidResourceMint,
}