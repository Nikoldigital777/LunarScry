use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use std::convert::TryFrom;

declare_id!("LunarScryV3111111111111111111111111111111111");

pub mod constants {
    pub const PROGRAM_VERSION: u8 = 3; // Updated version
    pub const MAX_EMERGENCY_ADMINS: usize = 10;
    pub const MAX_CONTENT_HASH_LENGTH: usize = 64;
    pub const MIN_VOTING_PERIOD: i64 = 86400; // 1 day
    pub const MAX_VOTING_PERIOD: i64 = 2592000; // 30 days
    pub const MIN_QUORUM_PERCENTAGE: u8 = 10;
    pub const MAX_QUORUM_PERCENTAGE: u8 = 90;
    pub const STAKE_LOCKUP_PERIOD: i64 = 86400; // 1 day
    pub const EARLY_VOTER_BONUS: u8 = 30; // 30% bonus
    pub const MAX_DAILY_SUBMISSIONS: u32 = 10000;
    pub const MAX_DAILY_VOTES: u32 = 100000;
    pub const MAX_STAKE_PER_USER: u64 = 10_000_000_000; // 10,000 tokens with 6 decimals
    pub const MIN_AI_CONFIDENCE: u8 = 50;
    pub const VOTE_COOLDOWN_PERIOD: i64 = 10; // 10 seconds between votes
    pub const REWARD_DISTRIBUTION_PERIOD: i64 = 86400; // 1 day
}

#[program]
pub mod lunar_scry {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        config: ProtocolConfig,
    ) -> Result<()> {
        let protocol = &mut ctx.accounts.protocol_state;

        require!(
            config.quorum_percentage >= constants::MIN_QUORUM_PERCENTAGE
                && config.quorum_percentage <= constants::MAX_QUORUM_PERCENTAGE,
            ErrorCode::InvalidQuorumPercentage
        );
        require!(
            config.voting_period >= constants::MIN_VOTING_PERIOD
                && config.voting_period <= constants::MAX_VOTING_PERIOD,
            ErrorCode::InvalidVotingPeriod
        );
        require!(
            config.reward_per_vote > 0,
            ErrorCode::InvalidRewardPerVote
        );

        protocol.admin = ctx.accounts.admin.key();
        protocol.stake_required = config.stake_required;
        protocol.voting_period = config.voting_period;
        protocol.quorum_percentage = config.quorum_percentage;
        protocol.reward_per_vote = config.reward_per_vote;
        protocol.treasury = ctx.accounts.treasury.key();
        protocol.is_paused = false;
        protocol.version = constants::PROGRAM_VERSION;
        protocol.bump = *ctx.bumps.get("protocol_state").unwrap();
        protocol.emergency_admins = vec![ctx.accounts.admin.key()];

        emit!(ProtocolInitialized {
            admin: protocol.admin,
            treasury: protocol.treasury,
            stake_required: protocol.stake_required,
            voting_period: protocol.voting_period,
            quorum_percentage: protocol.quorum_percentage,
            reward_per_vote: protocol.reward_per_vote,
            version: protocol.version,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn submit_content(
        ctx: Context<SubmitContent>,
        content_data: ContentData,
    ) -> Result<()> {
        let protocol = &mut ctx.accounts.protocol_state;
        let content = &mut ctx.accounts.content;
        let clock = Clock::get()?;

        protocol.check_active_status()?;
        protocol.check_and_update_daily_limits(clock.unix_timestamp)?;

        require!(
            content_data.content_type != ContentType::Video
                && content_data.content_type != ContentType::DeFi,
            ErrorCode::UnsupportedContentType
        );
        require!(
            content_data.content_hash.len() <= constants::MAX_CONTENT_HASH_LENGTH,
            ErrorCode::ContentHashTooLong
        );
        require!(
            content_data.ai_score >= constants::MIN_AI_CONFIDENCE,
            ErrorCode::LowAIConfidence
        );

        content.initialize(
            ctx.accounts.submitter.key(),
            content_data,
            protocol,
            clock.unix_timestamp,
            *ctx.bumps.get("content").unwrap(),
        )?;

        protocol.increment_submission_count()?;

        emit!(ContentSubmitted {
            content_id: content.key(),
            submitter: content.submitter,
            content_hash: content.content_hash,
            content_type: content.content_type,
            ai_score: content.ai_score,
            timestamp: content.submission_time,
        });

        Ok(())
    }

    pub fn cast_vote(
        ctx: Context<CastVote>,
        vote_type: VoteType,
        stake_amount: u64,
    ) -> Result<()> {
        let protocol = &mut ctx.accounts.protocol_state;
        let content = &mut ctx.accounts.content;
        let vote_account = &mut ctx.accounts.vote_account;
        let clock = Clock::get()?;

        protocol.validate_vote_transaction(content, stake_amount, clock.unix_timestamp)?;
        transfer_stake_tokens(ctx, stake_amount)?;
        content.process_vote(vote_type, stake_amount)?;
        vote_account.initialize(
            ctx.accounts.voter.key(),
            content.key(),
            vote_type,
            stake_amount,
            clock.unix_timestamp,
        )?;
        protocol.increment_vote_count()?;

        emit!(VoteCast {
            content_id: content.key(),
            voter: ctx.accounts.voter.key(),
            vote_type,
            stake_amount,
            timestamp: clock.unix_timestamp,
            vote_number: content.vote_count,
        });

        Ok(())
    }

    pub fn finalize_decision(
        ctx: Context<FinalizeDecision>,
    ) -> Result<()> {
        let protocol = &mut ctx.accounts.protocol_state;
        let content = &mut ctx.accounts.content;
        let clock = Clock::get()?;

        protocol.check_active_status()?;
        require!(
            clock.unix_timestamp > content.submission_time + content.voting_period,
            ErrorCode::VotingPeriodActive
        );

        let total_stake = content.approve_votes + content.reject_votes;
        require!(
            total_stake >= (content.total_stake * content.quorum_percentage as u64) / 100,
            ErrorCode::QuorumNotReached
        );

        let final_status = if content.approve_votes > content.reject_votes {
            ContentStatus::Approved
        } else {
            ContentStatus::Rejected
        };

        content.status = final_status;

        emit!(DecisionFinalized {
            content_id: content.key(),
            final_status,
            approve_votes: content.approve_votes,
            reject_votes: content.reject_votes,
            total_stake,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    pub fn claim_rewards(
        ctx: Context<ClaimRewards>,
    ) -> Result<()> {
        let protocol = &mut ctx.accounts.protocol_state;
        let content = &mut ctx.accounts.content;
        let vote_account = &mut ctx.accounts.vote_account;
        let voter = &mut ctx.accounts.voter;
        let vote_vault = &mut ctx.accounts.vote_vault;
        let reward_vault = &mut ctx.accounts.reward_vault;
        let clock = Clock::get()?;

        protocol.check_active_status()?;
        require!(
            clock.unix_timestamp >= vote_account.vote_timestamp + constants::STAKE_LOCKUP_PERIOD,
            ErrorCode::StakeStillLocked
        );
        require!(
            vote_account.voter == *voter.key,
            ErrorCode::Unauthorized
        );
        require!(
            vote_account.status == VoteStatus::Active,
            ErrorCode::RewardsAlreadyClaimed
        );

        let reward_amount = (vote_account.stake_amount * protocol.reward_per_vote) / content.total_stake;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: vote_vault.to_account_info(),
                    to: reward_vault.to_account_info(),
                    authority: protocol.to_account_info(),
                },
            ),
            reward_amount,
        )?;

        vote_account.status = VoteStatus::Rewarded;

        emit!(RewardsClaimed {
            voter: *voter.key,
            content_id: content.key(),
            reward_amount,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    pub fn pause_protocol(
        ctx: Context<PauseProtocol>,
    ) -> Result<()> {
        let protocol = &mut ctx.accounts.protocol_state;

        require!(
            protocol.emergency_admins.contains(&ctx.accounts.admin.key()),
            ErrorCode::Unauthorized
        );

        protocol.is_paused = true;

        emit!(ProtocolPaused {
            paused_by: ctx.accounts.admin.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn unpause_protocol(
        ctx: Context<UnpauseProtocol>,
    ) -> Result<()> {
        let protocol = &mut ctx.accounts.protocol_state;

        require!(
            protocol.emergency_admins.contains(&ctx.accounts.admin.key()),
            ErrorCode::Unauthorized
        );

        protocol.is_paused = false;

        emit!(ProtocolUnpaused {
            unpaused_by: ctx.accounts.admin.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn add_emergency_admin(
        ctx: Context<AddEmergencyAdmin>,
    ) -> Result<()> {
        let protocol = &mut ctx.accounts.protocol_state;

        require!(
            protocol.emergency_admins.len() < constants::MAX_EMERGENCY_ADMINS,
            ErrorCode::MaxEmergencyAdminsReached
        );
        require!(
            protocol.emergency_admins.contains(&ctx.accounts.admin.key()),
            ErrorCode::Unauthorized
        );

        protocol.emergency_admins.push(ctx.accounts.new_admin.key());

        emit!(EmergencyAdminAdded {
            new_admin: ctx.accounts.new_admin.key(),
            added_by: ctx.accounts.admin.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn remove_emergency_admin(
        ctx: Context<RemoveEmergencyAdmin>,
    ) -> Result<()> {
        let protocol = &mut ctx.accounts.protocol_state;

        require!(
            protocol.emergency_admins.len() > 1,
            ErrorCode::CannotRemoveLastAdmin
        );
        require!(
            protocol.emergency_admins.contains(&ctx.accounts.admin.key()),
            ErrorCode::Unauthorized
        );

        protocol.emergency_admins.retain(|&admin| admin != ctx.accounts.admin_to_remove.key());

        emit!(EmergencyAdminRemoved {
            removed_admin: ctx.accounts.admin_to_remove.key(),
            removed_by: ctx.accounts.admin.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn distribute_rewards(
    ctx: Context<DistributeRewards>,
) -> Result<()> {
    let protocol = &mut ctx.accounts.protocol_state;
    let clock = Clock::get()?;

    // Validation checks
    require!(
        !protocol.is_paused,
        ErrorCode::ProtocolPaused
    );

    require!(
        clock.unix_timestamp >= protocol.last_reward_distribution_timestamp + constants::REWARD_DISTRIBUTION_PERIOD,
        ErrorCode::RewardDistributionNotDue
    );

    // Calculate total rewards for the period
    let total_stake = ctx.accounts.stake_vault.amount;
    let reward_pool = ctx.accounts.reward_vault.amount;
    
    require!(reward_pool > 0, ErrorCode::InsufficientRewardPool);

    // Process each eligible voter
    let mut total_distributed: u64 = 0;
    for vote_account in &ctx.remaining_accounts {
        let vote = Account::<Vote>::try_from(vote_account)?;
        
        // Verify vote eligibility
        if !is_vote_eligible(&vote, clock.unix_timestamp)? {
            continue;
        }

        // Calculate voter's reward share
        let reward_amount = calculate_voter_reward(
            vote.stake_amount,
            total_stake,
            reward_pool,
            vote.timestamp,
            clock.unix_timestamp
        )?;

        // Apply early voter bonus if applicable
        let final_reward = if is_early_voter(&vote, protocol) {
            reward_amount
                .checked_mul(120)
                .ok_or(ErrorCode::CalculationError)?
                .checked_div(100)
                .ok_or(ErrorCode::CalculationError)?
        } else {
            reward_amount
        };

        // Transfer rewards
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.reward_vault.to_account_info(),
                    to: ctx.accounts.voter_token_account.to_account_info(),
                    authority: protocol.to_account_info(),
                },
                &[&[b"protocol", &[protocol.bump]]],
            ),
            final_reward,
        )?;

        total_distributed = total_distributed
            .checked_add(final_reward)
            .ok_or(ErrorCode::CalculationError)?;

        emit!(RewardDistributed {
            voter: vote.voter,
            amount: final_reward,
            timestamp: clock.unix_timestamp,
        });
    }

    // Update protocol state
    protocol.last_reward_distribution_timestamp = clock.unix_timestamp;
    protocol.total_rewards_distributed = protocol.total_rewards_distributed
        .checked_add(total_distributed)
        .ok_or(ErrorCode::CalculationError)?;

    emit!(RewardsDistributed {
        total_amount: total_distributed,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

// Helper functions
fn is_vote_eligible(vote: &Vote, current_timestamp: i64) -> Result<bool> {
    // Check if vote is within eligible period
    Ok(
        !vote.claimed &&
        vote.timestamp + constants::REWARD_ELIGIBILITY_PERIOD > current_timestamp &&
        vote.stake_amount >= constants::MIN_STAKE_FOR_REWARDS
    )
}

fn calculate_voter_reward(
    stake_amount: u64,
    total_stake: u64,
    reward_pool: u64,
    vote_timestamp: i64,
    current_timestamp: i64,
) -> Result<u64> {
    // Calculate base reward share
    let base_share = (stake_amount as u128)
        .checked_mul(reward_pool as u128)
        .ok_or(ErrorCode::CalculationError)?
        .checked_div(total_stake as u128)
        .ok_or(ErrorCode::CalculationError)?;

    // Apply time-weighted multiplier
    let time_weight = calculate_time_weight(vote_timestamp, current_timestamp)?;
    
    let final_reward = base_share
        .checked_mul(time_weight as u128)
        .ok_or(ErrorCode::CalculationError)?
        .checked_div(100)
        .ok_or(ErrorCode::CalculationError)?;

    Ok(u64::try_from(final_reward).unwrap_or(0))
}

fn calculate_time_weight(vote_timestamp: i64, current_timestamp: i64) -> Result<u64> {
    let time_diff = current_timestamp
        .checked_sub(vote_timestamp)
        .ok_or(ErrorCode::CalculationError)?;

    // Implement exponential decay for rewards based on time
    if time_diff < 86400 { // Within 24 hours
        Ok(100)
    } else if time_diff < 259200 { // Within 3 days
        Ok(75)
    } else if time_diff < 604800 { // Within 7 days
        Ok(50)
    } else {
        Ok(25)
    }
}

#[derive(Accounts)]
pub struct DistributeRewards<'info> {
    #[account(mut)]
    pub protocol_state: Account<'info, ProtocolState>,
    
    #[account(mut)]
    pub reward_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub stake_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub voter_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[event]
pub struct RewardDistributed {
    pub voter: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct RewardsDistributed {
    pub total_amount: u64,
    pub timestamp: i64,
}

#[derive(Accounts)]
pub struct PauseProtocol<'info> {
    #[account(mut, seeds = [b"protocol"], bump = protocol_state.bump)]
    pub protocol_state: Account<'info, ProtocolState>,
    #[account(signer)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct UnpauseProtocol<'info> {
    #[account(mut, seeds = [b"protocol"], bump = protocol_state.bump)]
    pub protocol_state: Account<'info, ProtocolState>,
    #[account(signer)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct AddEmergencyAdmin<'info> {
    #[account(mut, seeds = [b"protocol"], bump = protocol_state.bump)]
    pub protocol_state: Account<'info, ProtocolState>,
    #[account(signer)]
    pub admin: Signer<'info>,
    pub new_admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveEmergencyAdmin<'info> {
    #[account(mut, seeds = [b"protocol"], bump = protocol_state.bump)]
    pub protocol_state: Account<'info, ProtocolState>,
    #[account(signer)]
    pub admin: Signer<'info>,
    pub admin_to_remove: Signer<'info>,
}

#[derive(Accounts)]
pub struct DistributeRewards<'info> {
    #[account(mut, seeds = [b"protocol"], bump = protocol_state.bump)]
    pub protocol_state: Account<'info, ProtocolState>,
    #[account(signer)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub reward_vault: Account<'info, TokenAccount>,
}

#[error_code]
pub enum ErrorCode {
    // Previous errors
    #[msg("Unsupported content type")]
    UnsupportedContentType,
    #[msg("Content hash too long")]
    ContentHashTooLong,
    #[msg("Voting period is still active")]
    VotingPeriodActive,
    #[msg("Rewards already claimed")]
    RewardsAlreadyClaimed,
    #[msg("Invalid quorum percentage")]
    InvalidQuorumPercentage,
    #[msg("Invalid voting period")]
    InvalidVotingPeriod,
    #[msg("Invalid reward per vote")]
    InvalidRewardPerVote,
    #[msg("Maximum emergency admins reached")]
    MaxEmergencyAdminsReached,
    #[msg("Cannot remove the last emergency admin")]
    CannotRemoveLastAdmin,
    #[msg("Reward distribution not due yet")]
    RewardDistributionNotDue,
}

#[event]
pub struct ProtocolInitialized {
    pub admin: Pubkey,
    pub treasury: Pubkey,
    pub stake_required: u64,
    pub voting_period: i64,
    pub quorum_percentage: u8,
    pub reward_per_vote: u64,
    pub version: u8,
    pub timestamp: i64,
}

#[event]
pub struct ProtocolPaused {
    pub paused_by: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ProtocolUnpaused {
    pub unpaused_by: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyAdminAdded {
    pub new_admin: Pubkey,
    pub added_by: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyAdminRemoved {
    pub removed_admin: Pubkey,
    pub removed_by: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct RewardsDistributed {
    pub timestamp: i64,
}

#[account]
pub struct ProtocolState {
    pub admin: Pubkey,
    pub treasury: Pubkey,
    pub stake_required: u64,
    pub voting_period: i64,
    pub quorum_percentage: u8,
    pub reward_per_vote: u64,
    pub is_paused: bool,
    pub daily_submission_count: u32,
    pub daily_vote_count: u32,
    pub last_reset_timestamp: i64,
    pub last_reward_distribution_timestamp: i64,
    pub version: u8,
    pub bump: u8,
    pub emergency_admins: Vec<Pubkey>,
}

impl ProtocolState {
    pub const SIZE: usize = 8 + // discriminator
        32 + // admin
        32 + // treasury
        8 + // stake_required
        8 + // voting_period
        1 + // quorum_percentage
        8 + // reward_per_vote
        1 + // is_paused
        4 + // daily_submission_count
        4 + // daily_vote_count
        8 + // last_reset_timestamp
        8 + // last_reward_distribution_timestamp
        1 + // version
        1 + // bump
        (4 + (32 * constants::MAX_EMERGENCY_ADMINS)); // emergency_admins vector

    pub fn check_active_status(&self) -> Result<()> {
        require!(!self.is_paused, ErrorCode::ProtocolPaused);
        Ok(())
    }

    pub fn check_and_update_daily_limits(&mut self, current_timestamp: i64) -> Result<()> {
        if current_timestamp - self.last_reset_timestamp >= 86400 {
            self.daily_submission_count = 0;
            self.daily_vote_count = 0;
            self.last_reset_timestamp = current_timestamp;
        }
        Ok(())
    }

    pub fn validate_vote_transaction(
        &self,
        content: &Content,
        stake_amount: u64,
        current_timestamp: i64,
    ) -> Result<()> {
        self.check_active_status()?;
        require!(
            stake_amount >= self.stake_required && stake_amount <= constants::MAX_STAKE_PER_USER,
            ErrorCode::InvalidStakeAmount
        );
        require!(
            current_timestamp <= content.submission_time + content.voting_period,
            ErrorCode::VotingPeriodEnded
        );
        require!(
            current_timestamp >= content.last_vote_timestamp + constants::VOTE_COOLDOWN_PERIOD,
            ErrorCode::VotingTooFrequent
        );
        Ok(())
    }

    pub fn increment_submission_count(&mut self) -> Result<()> {
        self.daily_submission_count = self
            .daily_submission_count
            .checked_add(1)
            .ok_or(ErrorCode::CalculationError)?;
        Ok(())
    }

    pub fn increment_vote_count(&mut self) -> Result<()> {
        self.daily_vote_count = self
            .daily_vote_count
            .checked_add(1)
            .ok_or(ErrorCode::CalculationError)?;
        Ok(())
    }
}

#[account]
pub struct Content {
    pub submitter: Pubkey,
    pub content_hash: [u8; 32],
    pub content_type: ContentType,
    pub ai_score: u8,
    pub submission_time: i64,
    pub status: ContentStatus,
    pub approve_votes: u64,
    pub reject_votes: u64,
    pub total_stake: u64,
    pub voting_period: i64,
    pub quorum_percentage: u8,
    pub vote_count: u32,
    pub last_vote_timestamp: i64,
    pub version: u8,
    pub bump: u8,
    pub moderation_flags: u8,
}

impl Content {
    pub const SIZE: usize = 8 + // discriminator
        32 + // submitter
        32 + // content_hash
        1 + // content_type
        1 + // ai_score
        8 + // submission_time
        1 + // status
        8 + // approve_votes
        8 + // reject_votes
        8 + // total_stake
        8 + // voting_period
        1 + // quorum_percentage
        4 + // vote_count
        8 + // last_vote_timestamp
        1 + // version
        1 + // bump
        1; // moderation_flags

    pub fn initialize(
        &mut self,
        submitter: Pubkey,
        content_data: ContentData,
        protocol: &ProtocolState,
        current_timestamp: i64,
        bump: u8,
    ) -> Result<()> {
        self.submitter = submitter;
        self.content_hash = content_data.content_hash;
        self.content_type = content_data.content_type;
        self.ai_score = content_data.ai_score;
        self.submission_time = current_timestamp;
        self.status = ContentStatus::Pending;
        self.voting_period = protocol.voting_period;
        self.quorum_percentage = protocol.quorum_percentage;
        self.version = constants::PROGRAM_VERSION;
        self.bump = bump;
        Ok(())
    }

    pub fn process_vote(
        &mut self,
        vote_type: VoteType,
        stake_amount: u64,
    ) -> Result<()> {
        match vote_type {
            VoteType::Approve => {
                self.approve_votes = self
                    .approve_votes
                    .checked_add(stake_amount)
                    .ok_or(ErrorCode::CalculationError)?;
            }
            VoteType::Reject => {
                self.reject_votes = self
                    .reject_votes
                    .checked_add(stake_amount)
                    .ok_or(ErrorCode::CalculationError)?;
            }
        }

        self.total_stake = self
            .total_stake
            .checked_add(stake_amount)
            .ok_or(ErrorCode::CalculationError)?;

        self.vote_count = self
            .vote_count
            .checked_add(1)
            .ok_or(ErrorCode::CalculationError)?;

        self.last_vote_timestamp = Clock::get()?.unix_timestamp;

        Ok(())
    }
}

#[account]
pub struct Vote {
    pub voter: Pubkey,
    pub content_id: Pubkey,
    pub vote_type: VoteType,
    pub stake_amount: u64,
    pub vote_timestamp: i64,
    pub status: VoteStatus,
}

impl Vote {
    pub const SIZE: usize = 8 + // discriminator
        32 + // voter
        32 + // content_id
        1 + // vote_type
        8 + // stake_amount
        8 + // vote_timestamp
        1; // status
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum VoteStatus {
    Active,
    Rewarded,
}
