use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, Transfer};

declare_id!("GTav3wuJLZq9qRXC5iihFnuHk1MYYcetUyxtW6VWo5Ri");

#[program]
pub mod seer {
    use super::*;
    pub fn initialize(
        ctx: Context<Initialize>,
        program_name: String,
        bumps: PoolBumps
        ) -> ProgramResult {
        let main_account  = &mut ctx.accounts.main_account;
        
        let name_bytes = program_name.as_bytes();
        let mut name_data = [b' '; 10];
        name_data[..name_bytes.len()].copy_from_slice(name_bytes);
        
        main_account.program_name = name_data;
        main_account.bumps = bumps;

        main_account.authority = ctx.accounts.user.key();
        main_account.usdc_mint = ctx.accounts.usdc_token.key();
        main_account.redeemable_mint = ctx.accounts.redeemable_mint.key();
        main_account.pool_usdc = ctx.accounts.pool_usdc.key();

        let user_data = &mut ctx.accounts.user_data;
        user_data.bump = main_account.bumps.user_data;

        Ok(())
    }

    pub fn init_user_redeemable(ctx: Context<InitUserRedeemable>) -> ProgramResult{
        Ok(())
    }

    pub fn deposit_usdc(ctx: Context<DepositUsdc>, amount: u64) -> ProgramResult{
        let user_data = &mut ctx.accounts.user_data;
        let sender = & ctx.accounts.user;
        user_data.author = *sender.key;

        let cpi_accounts = Transfer{
            from: ctx.accounts.user_usdc.to_account_info(),
            to: ctx.accounts.pool_usdc.to_account_info(),
            authority: ctx.accounts.user.to_account_info()
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx  = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        user_data.total_deposited = user_data.total_deposited + amount;

        let program_name = ctx.accounts.main_account.program_name.as_ref();
        let seeds = &[
            program_name,
            &[ctx.accounts.main_account.bumps.main_account]
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = MintTo{
            mint: ctx.accounts.redeemable_mint.to_account_info(),
            to: ctx.accounts.user_redeemable.to_account_info(),
            authority: ctx.accounts.user.to_account_info()
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);   
        token::mint_to(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn init_user_escrow(ctx: Context<InitUserEscrow>) -> ProgramResult{
        Ok(())
    }

    pub fn withdraw_usdc(ctx: Context<WithdrawUsdc>, amount: u64) -> ProgramResult{
        let program_name = ctx.accounts.main_account.program_name.as_ref();
        let seeds = &[
            program_name,
            &[ctx.accounts.main_account.bumps.main_account]
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Burn{
            mint: ctx.accounts.redeemable_mint.to_account_info(),
            to: ctx.accounts.user_redeemable.to_account_info(),
            authority: ctx.accounts.main_account.to_account_info()
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::burn(cpi_ctx, amount)?;

        let cpi_accounts = Transfer{
            from: ctx.accounts.pool_usdc.to_account_info(),
            to: ctx.accounts.user_escrow.to_account_info(),
            authority: ctx.accounts.user.to_account_info()
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

}

#[derive(Accounts)]
pub struct WithdrawUsdc<'info>{
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [
            user.key().as_ref(),
            main_account.program_name.as_ref(),
            b"user_redeemable"
        ],
        bump
    )]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [
            main_account.program_name.as_ref(),
            b"redeemable_mint"
        ],
        bump = main_account.bumps.redeemable_mint
    )]
    pub redeemable_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [
            user.key().as_ref(),
            main_account.program_name.as_ref(),
            b"user_escrow"
        ],
        bump
    )]
    pub user_escrow: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [
            main_account.program_name.as_ref()
        ],
        bump = main_account.bumps.main_account,
        has_one = usdc_mint
    )]
    pub main_account: Box<Account<'info, MainAccount>>,
    pub usdc_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [
            main_account.program_name.as_ref(),
            b"pool_usdc"
        ],
        bump = main_account.bumps.pool_usdc
    )]
    pub pool_usdc: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    // pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct DepositUsdc<'info>{
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [
            user.key().as_ref(),
            main_account.program_name.as_ref(),
            b"user_redeemable"
        ],
        bump
    )]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [
            main_account.program_name.as_ref(),
            b"redeemable_mint"
        ],
        bump = main_account.bumps.redeemable_mint
    )]
    pub redeemable_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [
            user.key().as_ref()
        ],
        bump = user_data.bump
    )]
    pub user_data: Account<'info, UserData>,

    #[account(
        mut,
        constraint = user_usdc.owner == user.key(),
        constraint = user_usdc.mint == usdc_mint.key()
    )]
    pub user_usdc: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [
            main_account.program_name.as_ref()
        ],
        bump = main_account.bumps.main_account,
        has_one = usdc_mint
    )]
    pub main_account: Box<Account<'info, MainAccount>>,
    pub usdc_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [
            main_account.program_name.as_ref(),
            b"pool_usdc"
        ],
        bump = main_account.bumps.pool_usdc
    )]
    pub pool_usdc: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: AccountInfo<'info>,
}    

#[derive(Accounts)]
pub struct InitUserRedeemable<'info>{
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        token::mint = redeemable_mint,
        token::authority = main_account,
        seeds = [
            user.key().as_ref(),
            main_account.program_name.as_ref(),
            b"user_redeemable"
        ],
        bump,
        payer=user
    )]
    pub user_redeemable: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [
            main_account.program_name.as_ref(),
        ],
        bump = main_account.bumps.main_account
    )]
    pub main_account: Box<Account<'info, MainAccount>>,

    #[account(
        seeds = [
            main_account.program_name.as_ref(),
            b"redeemable_mint"
        ],
        bump = main_account.bumps.redeemable_mint
    )]
    pub redeemable_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitUserEscrow<'info>{
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        token::mint = usdc_mint,
        token::authority = main_account,
        seeds = [
            user.key().as_ref(),
            main_account.program_name.as_ref(),
            b"user_escrow"
        ],
        bump,
        payer=user
    )]
    pub user_escrow: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [
            main_account.program_name.as_ref(),
        ],
        bump = main_account.bumps.main_account,
        has_one = usdc_mint
    )]
    pub main_account: Box<Account<'info, MainAccount>>,
    pub usdc_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}



#[derive(Accounts)]
#[instruction(program_name: String, bumps: PoolBumps)]
pub struct Initialize<'info> {
    #[account()]
    pub user: Signer<'info>,

    #[account(
        init,
        seeds = [user.key().as_ref()],
        bump,
        payer=user
    )]
    pub user_data: Account<'info, UserData>,

    #[account(
        init,
        seeds = [program_name.as_bytes()],
        bump,
        payer=user
    )]
    pub main_account: Box<Account<'info,MainAccount>>,

    #[account(constraint = usdc_token.decimals == 6)]
    pub usdc_token: Box<Account<'info, Mint>>,

    #[account(
        init,
        token::mint = usdc_token,
        token::authority = user,
        seeds = [program_name.as_bytes(), b"pool_usdc".as_ref()],
        bump,
        payer=user
    )]
    pub pool_usdc:Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        mint::decimals = 6,
        mint::authority = user,
        seeds = [program_name.as_bytes(), b"redeemable_mint".as_ref()],
        bump,
        payer=user

    )]
    pub redeemable_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}

#[account]
#[derive( Default)]
pub struct MainAccount {
    pub program_name: [u8; 10],
    pub bumps: PoolBumps,

    pub authority: Pubkey,
    pub usdc_mint: Pubkey,
    pub redeemable_mint: Pubkey,
    pub pool_usdc: Pubkey
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct PoolBumps{
    pub main_account: u8,
    pub redeemable_mint: u8,
    pub pool_usdc: u8,
    pub user_data: u8
}

#[account]
#[derive(Default)]
pub struct UserData {
    pub author: Pubkey,
    pub total_deposited: u64,
    pub total_debt: u64,
    pub total_credit: u64,
    pub last_deposit: u64,
    pub bump: u8
}