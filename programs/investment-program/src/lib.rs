use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("Mh9eszZZsLKqd17YTTy1rKMSshKY4aqBaNPGBG8x5db");

#[program]
pub mod investment_program {
    use super::*;

    pub fn initialize_user(ctx: Context<InitializeUser>) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        user_account.owner = ctx.accounts.user.key();
        user_account.total_invested = 0;
        user_account.total_sol_received = 0;
        user_account.investment_count = 0;
        user_account.last_investment_time = Clock::get()?.unix_timestamp;
        user_account.is_active = true;
        user_account.reward_points = 0;
        
        msg!("User account initialized for: {}", ctx.accounts.user.key());
        Ok(())
    }

    pub fn invest(ctx: Context<Invest>, amount_in_lamports: u64) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        let clock = Clock::get()?;
        
        require!(amount_in_lamports > 0, InvestmentError::InvalidAmount);
        require!(user_account.is_active, InvestmentError::UserNotActive);
        
        // Update user investment data
        user_account.total_invested = user_account.total_invested
            .checked_add(amount_in_lamports)
            .ok_or(InvestmentError::MathOverflow)?;
        
        user_account.total_sol_received = user_account.total_sol_received
            .checked_add(amount_in_lamports)
            .ok_or(InvestmentError::MathOverflow)?;
        
        user_account.investment_count = user_account.investment_count
            .checked_add(1)
            .ok_or(InvestmentError::MathOverflow)?;
        
        user_account.last_investment_time = clock.unix_timestamp;
        
        // Calculate reward points (1 point per 0.01 SOL invested)
        let reward_points = amount_in_lamports / 10_000_000; // 0.01 SOL in lamports
        user_account.reward_points = user_account.reward_points
            .checked_add(reward_points)
            .ok_or(InvestmentError::MathOverflow)?;
        
        // Create investment record
        let investment_record = &mut ctx.accounts.investment_record;
        investment_record.user = ctx.accounts.user.key();
        investment_record.amount = amount_in_lamports;
        investment_record.timestamp = clock.unix_timestamp;
        investment_record.investment_type = InvestmentType::Manual;
        investment_record.transaction_signature = ctx.accounts.user.key().to_string(); // Placeholder
        
        msg!(
            "Investment recorded: {} lamports for user: {}",
            amount_in_lamports,
            ctx.accounts.user.key()
        );
        
        Ok(())
    }

    pub fn auto_invest(ctx: Context<Invest>, amount_in_lamports: u64) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        let clock = Clock::get()?;
        
        require!(amount_in_lamports > 0, InvestmentError::InvalidAmount);
        require!(user_account.is_active, InvestmentError::UserNotActive);
        
        // Update user investment data
        user_account.total_invested = user_account.total_invested
            .checked_add(amount_in_lamports)
            .ok_or(InvestmentError::MathOverflow)?;
        
        user_account.total_sol_received = user_account.total_sol_received
            .checked_add(amount_in_lamports)
            .ok_or(InvestmentError::MathOverflow)?;
        
        user_account.investment_count = user_account.investment_count
            .checked_add(1)
            .ok_or(InvestmentError::MathOverflow)?;
        
        user_account.last_investment_time = clock.unix_timestamp;
        
        // Auto investments get bonus reward points (1.5x multiplier)
        let reward_points = (amount_in_lamports / 10_000_000) * 3 / 2;
        user_account.reward_points = user_account.reward_points
            .checked_add(reward_points)
            .ok_or(InvestmentError::MathOverflow)?;
        
        // Create investment record
        let investment_record = &mut ctx.accounts.investment_record;
        investment_record.user = ctx.accounts.user.key();
        investment_record.amount = amount_in_lamports;
        investment_record.timestamp = clock.unix_timestamp;
        investment_record.investment_type = InvestmentType::Auto;
        investment_record.transaction_signature = ctx.accounts.user.key().to_string();
        
        msg!(
            "Auto investment recorded: {} lamports for user: {}",
            amount_in_lamports,
            ctx.accounts.user.key()
        );
        
        Ok(())
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        
        require!(user_account.reward_points > 0, InvestmentError::NoRewardsAvailable);
        
        let rewards_to_claim = user_account.reward_points;
        user_account.reward_points = 0;
        
        msg!(
            "Rewards claimed: {} points for user: {}",
            rewards_to_claim,
            ctx.accounts.user.key()
        );
        
        Ok(())
    }

    pub fn get_user_stats(ctx: Context<GetUserStats>) -> Result<UserStats> {
        let user_account = &ctx.accounts.user_account;
        
        Ok(UserStats {
            total_invested: user_account.total_invested,
            total_sol_received: user_account.total_sol_received,
            investment_count: user_account.investment_count,
            reward_points: user_account.reward_points,
            last_investment_time: user_account.last_investment_time,
            is_active: user_account.is_active,
        })
    }
}

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + UserAccount::INIT_SPACE,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Invest<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(
        init,
        payer = user,
        space = 8 + InvestmentRecord::INIT_SPACE,
        seeds = [
            b"investment",
            user.key().as_ref(),
            &user_account.investment_count.to_le_bytes()
        ],
        bump
    )]
    pub investment_record: Account<'info, InvestmentRecord>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct GetUserStats<'info> {
    #[account(
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    
    pub user: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct UserAccount {
    pub owner: Pubkey,
    pub total_invested: u64,
    pub total_sol_received: u64,
    pub investment_count: u64,
    pub last_investment_time: i64,
    pub is_active: bool,
    pub reward_points: u64,
}

#[account]
#[derive(InitSpace)]
pub struct InvestmentRecord {
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
    pub investment_type: InvestmentType,
    #[max_len(88)]
    pub transaction_signature: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum InvestmentType {
    Manual,
    Auto,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UserStats {
    pub total_invested: u64,
    pub total_sol_received: u64,
    pub investment_count: u64,
    pub reward_points: u64,
    pub last_investment_time: i64,
    pub is_active: bool,
}

#[error_code]
pub enum InvestmentError {
    #[msg("Invalid investment amount")]
    InvalidAmount,
    #[msg("User account is not active")]
    UserNotActive,
    #[msg("Math operation overflow")]
    MathOverflow,
    #[msg("No rewards available to claim")]
    NoRewardsAvailable,
}