use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount}};
use fixed::types::I64F64;

use super::shared::{transfer_tokens};

use crate::{state::PoolState, error::ErrorCode};

#[derive(Accounts)]
pub struct Swap<'info> {
  #[account(mut)]
  pub trader: Signer<'info>,

  #[account(
    init_if_needed,
    payer = trader,
    associated_token::mint = mint_x,
    associated_token::authority = trader,
  )]
  pub trader_x_ata: Account<'info, TokenAccount>,

  #[account(
    init_if_needed,
    payer = trader,
    associated_token::mint = mint_y,
    associated_token::authority = trader,
  )]
  trader_y_ata: Account<'info, TokenAccount>,

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
    seeds = [b"lp", pool_state.key().as_ref()],
    bump
  )]
  pub mint_lp: Account<'info, Mint>,

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

impl<'info> Swap<'info> {
  pub fn swap_exact_tokens_for_tokens(
    &mut self,
    input_is_x: bool,
    input_amount: u64,
    min_output_amount: u64,
  ) -> Result<()> {
    // pool should not be locked and initialized
    require!(self.pool_state.locked == false, ErrorCode::PoolLocked);
    require!(self.mint_lp.supply > 0, ErrorCode::PoolNotFound);

    let input = if input_is_x && input_amount > self.trader_x_ata.amount {
      self.trader_x_ata.amount
    } else if !input_is_x && input_amount > self.trader_y_ata.amount {
      self.trader_y_ata.amount
    } else {
      input_amount
    };

    let taxed_input = input.checked_sub(input.checked_mul(self.pool_state.fee as u64).unwrap().checked_div(10000).unwrap()).unwrap();

    // calculate swap out amount
    let output = if input_is_x {
      I64F64::from_num(taxed_input)
          .checked_mul(I64F64::from_num(self.vault_y_ata.amount))
          .unwrap()
          .checked_div(
              I64F64::from_num(self.vault_x_ata.amount)
                  .checked_add(I64F64::from_num(taxed_input))
                  .unwrap(),
          )
          .unwrap()
    } else {
        I64F64::from_num(taxed_input)
            .checked_mul(I64F64::from_num(self.vault_x_ata.amount))
            .unwrap()
            .checked_div(
                I64F64::from_num(self.vault_y_ata.amount)
                    .checked_add(I64F64::from_num(taxed_input))
                    .unwrap(),
            )
            .unwrap()
    }
    .to_num::<u64>();

    if output < min_output_amount {
      return Err(ErrorCode::OutputTooSmall.into());
    }

    // transfer
    self.swap_token(input_is_x, input, output)?;

    Ok(())
  }

  pub fn swap_token(&self, input_is_x: bool, input: u64, output: u64) -> Result<()> {
    let seeds = &[
      &b"pool_state"[..],
      &self.pool_state.seed.to_le_bytes(),
      &[self.pool_state.bump],
    ];

    match input_is_x {
      true => {
        transfer_tokens(
          &self.trader_x_ata,
          &self.vault_x_ata,
          &input,
          &self.mint_x,
          &self.trader,
          &self.token_program,
          None,
        )
        .map_err(|_| ErrorCode::SwapInFailed)?;

        transfer_tokens(
          &self.vault_y_ata,
          &self.trader_y_ata,
          &output,
          &self.mint_y,
          &self.pool_state.to_account_info(),
          &self.token_program,
          Some(&seeds[..]),
        )
        .map_err(|_| ErrorCode::SwapOutFailed)?;

      },
      false => {
        transfer_tokens(
          &self.trader_y_ata,
          &self.vault_y_ata,
          &input,
          &self.mint_y,
          &self.trader,
          &self.token_program,
          None,
        )
        .map_err(|_| ErrorCode::SwapInFailed)?;

        transfer_tokens(
          &self.vault_x_ata,
          &self.trader_x_ata,
          &output,
          &self.mint_x,
          &self.pool_state.to_account_info(),
          &self.token_program,
          Some(&seeds[..]),
        )
        .map_err(|_| ErrorCode::SwapOutFailed)?;
      },
    };

    Ok(())
  }
}