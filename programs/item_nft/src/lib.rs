use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface, MintTo, mint_to};
use resource_manager::program::ResourceManager;
use resource_manager::cpi::accounts::BurnResource;

declare_id!("AmdVbUTd8VV6XTd6udq3ZgoyqdTAf63iCZythgohzaLG");

/// програма для створення nft предметів через механіку крафту
#[program]
pub mod item_nft {
    use super::*;

    /// обмін базових ресурсів на ігровий nft предмет згідно з рецептом
    pub fn craft_item(ctx: Context<CraftItem>, item_id: u8) -> Result<()> {
        // визначення кількості необхідних ресурсів для кожного типу предмета
        let (wood_req, iron_req, leather_req) = match item_id {
            1 => (1, 3, 1), // Шабля: 1 дерево, 3 заліза, 1 шкіра
            2 => (2, 0, 0), // Посох: 2 дерева
            _ => return err!(ErrorCode::InvalidRecipe),
        };

        let cpi_program = ctx.accounts.resource_manager_program.to_account_info();

        // спалення дерева через виклик cpi до менеджера ресурсів
        if wood_req > 0 {
            let cpi_accounts = BurnResource {
                game_config: ctx.accounts.game_config.to_account_info(),
                resource_mint: ctx.accounts.wood_mint.to_account_info(),
                player_token_account: ctx.accounts.player_wood_account.to_account_info(),
                player: ctx.accounts.player.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            };
            let cpi_ctx = CpiContext::new(cpi_program.clone(), cpi_accounts);
            resource_manager::cpi::burn_resource(cpi_ctx, 0, wood_req)?;
        }

        // спалення заліза через виклик cpi до менеджера ресурсів
        if iron_req > 0 {
            let cpi_accounts = BurnResource {
                game_config: ctx.accounts.game_config.to_account_info(),
                resource_mint: ctx.accounts.iron_mint.to_account_info(),
                player_token_account: ctx.accounts.player_iron_account.to_account_info(),
                player: ctx.accounts.player.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            };
            let cpi_ctx = CpiContext::new(cpi_program.clone(), cpi_accounts);
            resource_manager::cpi::burn_resource(cpi_ctx, 1, iron_req)?;
        }

        // спалення шкіри через виклик cpi до менеджера ресурсів
        if leather_req > 0 {
            let cpi_accounts = BurnResource {
                game_config: ctx.accounts.game_config.to_account_info(),
                resource_mint: ctx.accounts.leather_mint.to_account_info(),
                player_token_account: ctx.accounts.player_leather_account.to_account_info(),
                player: ctx.accounts.player.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            };
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            resource_manager::cpi::burn_resource(cpi_ctx, 3, leather_req)?;
        }

        // підготовка підписів pda для мінту предмета
        let seeds = &[b"nft_authority".as_ref(), &[ctx.bumps.nft_authority]];
        let signer = &[&seeds[..]];

        // випуск nft предмета на акаунт гравця
        let cpi_ctx_mint = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            MintTo {
                mint: ctx.accounts.item_mint.to_account_info(),
                to: ctx.accounts.player_item_account.to_account_info(),
                authority: ctx.accounts.nft_authority.to_account_info(),
            }, 
            signer
        );
        mint_to(cpi_ctx_mint, 1)?;

        msg!("Козак успішно скрафтив предмет ID: {}", item_id);
        Ok(())
    }
}

/// перелік акаунтів для виконання інструкції крафту предмета
#[derive(Accounts)]
pub struct CraftItem<'info> {
    /// гравець який здійснює крафт та підписує транзакцію
    #[account(mut)]
    pub player: Signer<'info>,

    /// акаунт з налаштуваннями гри
    #[account(mut)]
    pub game_config: Account<'info, resource_manager::GameConfig>,

    /// мінт токена дерева
    #[account(mut)]
    pub wood_mint: InterfaceAccount<'info, Mint>,
    /// токенний акаунт гравця для дерева
    #[account(mut)]
    pub player_wood_account: InterfaceAccount<'info, TokenAccount>,

    /// мінт токена заліза
    #[account(mut)]
    pub iron_mint: InterfaceAccount<'info, Mint>,
    /// токенний акаунт гравця для заліза
    #[account(mut)]
    pub player_iron_account: InterfaceAccount<'info, TokenAccount>,

    /// мінт токена шкіри
    #[account(mut)]
    pub leather_mint: InterfaceAccount<'info, Mint>,
    /// токенний акаунт гравця для шкіри
    #[account(mut)]
    pub player_leather_account: InterfaceAccount<'info, TokenAccount>,

    /// мінт nft предмета що створюється
    #[account(mut)]
    pub item_mint: InterfaceAccount<'info, Mint>,
    /// токенний акаунт гравця для отримання предмета
    #[account(mut)]
    pub player_item_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: pda акаунт для підпису випуску токенів
    #[account(seeds = [b"nft_authority"], bump)]
    pub nft_authority: AccountInfo<'info>,

    /// посилання на програму менеджера ресурсів
    pub resource_manager_program: Program<'info, ResourceManager>,
    /// посилання на програму токенів spl
    pub token_program: Interface<'info, TokenInterface>,
}

/// коди помилок програми крафту предметів
#[error_code]
pub enum ErrorCode {
    /// помилка що виникає при відсутності рецепта для вказаного id
    #[msg("Такого рецепту не існує!")]
    InvalidRecipe,
}