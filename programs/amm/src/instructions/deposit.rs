use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Transfer, transfer, Mint, Token, TokenAccount, MintTo, mint_to}};
use constant_product_curve::ConstantProduct;
use fixed::types::I64F64;

use super::shared::{transfer_tokens};

use crate::{state::PoolState, constants::{ MINIMUM_LIQUIDITY}, error::ErrorCode};

#[derive(Accounts)]
pub struct Deposit<'info> {
  #[account(mut)]
  pub depositor: Signer<'info>,

  #[account(
    mut,
    seeds = [b"mint_lp", pool_state.key().as_ref()],
    bump = pool_state.bump_mint_lp,
  )]
  pub mint_lp: Account<'info, Mint>,

  #[account(
    mut,
    associated_token::mint = mint_x,
    associated_token::authority = depositor,
  )]
  pub depositor_x_ata: Account<'info, TokenAccount>,

  #[account(
    mut,
    associated_token::mint = mint_y,
    associated_token::authority = depositor,
  )]
  depositor_y_ata: Account<'info, TokenAccount>,

  #[account(
    init_if_needed,
    payer = depositor,
    associated_token::mint = mint_lp,
    associated_token::authority = depositor,
  )]
  pub depositor_lp_ata: Account<'info, TokenAccount>,

  #[account(
    mut,
    associated_token::mint = mint_x,
    associated_token::authority = pool_state,
  )]
  pub vault_x_ata: Account<'info, TokenAccount>,

  #[account(
    mut,
    associated_token::mint = mint_y,
    associated_token::authority = pool_state,
  )]
  pub vault_y_ata: Account<'info, TokenAccount>,

  #[account(
    seeds = [b"pool_state",pool_state.seed.to_le_bytes().as_ref()],
    bump = pool_state.bump,
  )]
  pub pool_state: Account<'info, PoolState>,

  pub mint_x: Account<'info, Mint>,
  pub mint_y: Account<'info, Mint>,
  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
  pub fn deposit(&mut self, max_x: u64, max_y: u64) -> Result<()> {
    // pool not locked
    require!(self.pool_state.locked == false, ErrorCode::PoolLocked);

    let mut x = if max_x > self.depositor_x_ata.amount  {
      self.depositor_x_ata.amount
    } else {
        max_x
    };

    let mut y = if max_y > self.depositor_y_ata.amount {
      self.depositor_y_ata.amount
    } else {
        max_y
    };

    let pool_creation = self.vault_x_ata.amount == 0 && self.vault_y_ata.amount == 0;

    // first deposit
    (x, y) = match pool_creation {
      // leave mininum lp then it cannot be drained
      true => (self.vault_x_ata.amount, self.vault_y_ata.amount),
      false => {
        let ratio = I64F64::from_num(self.vault_x_ata.amount)
        .checked_div(I64F64::from_num(self.vault_y_ata.amount))
        .unwrap(); // a / b

        let delta_x = I64F64::from_num(y)
            .checked_mul(ratio)
            .unwrap()
            .to_num::<u64>();

        if delta_x <= x {
            (
                I64F64::from_num(y)
                    .checked_mul(ratio)
                    .unwrap()
                    .to_num::<u64>(), //  Δa = Δb * a / b 
                y, 
            )
        } else {
            (
                x,
                I64F64::from_num(x)
                    .checked_div(ratio)
                    .unwrap()
                    .to_num::<u64>(),
            )
        }
      }
    };

    let mut lp = I64F64::from_num(x)
      .checked_mul(I64F64::from_num(y))
      .unwrap()
      .sqrt()
      .to_num::<u64>();

    if pool_creation {
      require!(lp < MINIMUM_LIQUIDITY, ErrorCode::DepositTooSmall);

      lp = lp.checked_sub(MINIMUM_LIQUIDITY).unwrap();
    }

    self.deposit_token(true, x)?;
    self.deposit_token(false, y)?;
    self.mint_lp(lp)?;

    Ok(())
  }

  pub fn deposit_token(&self, is_x: bool, amount: u64) -> Result<()> {
    let (mint, from, to) = match is_x {
      true => (&self.mint_x, &self.depositor_x_ata, &self.vault_x_ata),
      false => (&self.mint_y, &self.depositor_y_ata, &self.vault_y_ata),
    };

    transfer_tokens(
      from,
      to,
      &amount,
      mint,
      &self.depositor,
      &self.token_program,
      None,
    )
    .map_err(|_| ErrorCode::DepositToPoolFailed)?;

    Ok(())
  }

  pub fn mint_lp(&self, amount: u64) -> Result<()> {
    let cpi_program = self.token_program.to_account_info();

    let cpi_accounts = MintTo {
        mint: self.mint_lp.to_account_info(),
        to: self.depositor_lp_ata.to_account_info(),
        authority: self.pool_state.to_account_info()
    };

    let seeds = &[
        &b"pool_state"[..],
        &self.pool_state.seed.to_le_bytes(),
        &[self.pool_state.bump],
    ];

    let signers_seeds = &[&seeds[..]];

    let ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers_seeds);

    mint_to(ctx, amount)
  }
}