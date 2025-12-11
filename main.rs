use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("tmk3y8hqZnix8yWBZxY41AN13HTYGZNrSEVL3PNEDYh");

#[program]
pub mod stake{
    use super::*;

    pub fn initialize(ctx : Context<Initialize>) -> Result<()>{
        let pool = &mut ctx.accounts.pool;
        pool.admin = ctx.accounts.admin.key();
        Ok(())
    }

    pub fn stake(ctx : Context<Stake>, amount : u64) -> Result<()>{
        // require!(amount > 0, "amount cannot be less than zero");

        let cpi_account = system_program::Transfer{
            from: ctx.accounts.user.to_account_info(),
            to: ctx.accounts.pool.to_account_info()
        };

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            cpi_account
        );

        system_program::transfer(cpi_context, amount)?;

        let user_stake = &mut ctx.accounts.user_stake;
        let now = Clock::get()?.unix_timestamp;

        if user_stake.amount > 0 {
            let seconds = now - user_stake.last_update;
            // let days = seconds/86400;
            let days = seconds;
            let points = (user_stake.amount / 1_000_000_000) * days as u64;
            user_stake.points += points;
        }else{
            user_stake.user = ctx.accounts.user.key();
        }

        user_stake.amount += amount;
        user_stake.last_update = now;

        Ok(())
    }

    pub fn unstake(ctx : Context<UnStake>, amount : u64) -> Result<()>{
        let user_stake = &mut ctx.accounts.user_stake;
        let now = Clock::get()?.unix_timestamp;
        let seconds = now - user_stake.last_update;
        let days = seconds;
        let points = (user_stake.amount / 1_000_000_000) * days as u64;
        user_stake.points += points;
        user_stake.last_update = now;

        user_stake.amount -= amount;
        user_stake.last_update = now;

        let pool = &ctx.accounts.pool;
        **pool.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += amount;
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info>{
    #[account(mut)]
    pub admin : Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + 32,
        seeds = [b"pool"],
        bump
    )]
    pub pool : Account<'info, StakingPool>,

    pub system_program : Program<'info ,System>
}

#[derive(Accounts)]
pub struct Stake<'info>{
    #[account(
        mut,
        seeds = [b"pool"],
        bump
    )]
    pub pool : Account<'info, StakingPool>,

    #[account(mut)]
    pub user : Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 8 + 8 + 8,
        seeds = [b"user_stake", user.key().as_ref()],
        bump
    )]
    pub user_stake : Account<'info, UserStake>,

    pub system_program : Program<'info, System>
}

#[derive(Accounts)]
pub struct UnStake<'info>{
    #[account(
        mut,
        seeds = [b"pool"],
        bump
    )]
    pub pool : Account<'info, StakingPool>,

    #[account(mut)]
    pub user : Signer<'info>,

    #[account(
        mut,
        seeds = [b"user_stake", user.key().as_ref()],
        bump
    )]
    pub user_stake : Account<'info, UserStake>,

    pub system_program : Program<'info, System>
}

#[account]
pub struct StakingPool{
    pub admin : Pubkey
}

#[account]
pub struct UserStake{
    pub user : Pubkey,
    pub amount : u64,
    pub points : u64,
    pub last_update : i64
}