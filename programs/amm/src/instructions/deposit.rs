use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Transfer, transfer, Mint, Token, TokenAccount, MintTo, mint_to}};
use constant_product_curve::ConstantProduct;
use super::shared::{transfer_tokens};

use crate::{state::PoolState, error::ErrorCode};

#[derive(Accounts)]
pub struct Deposit<'info> {
  #[account(mut)]
  pub depositor: Signer<'info>,

  #[account(
    mut,
    seeds = [b"lp", pool_state.key().as_ref()],
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
    bump = pool_state.bump_pool_state,
  )]
  pub pool_state: Account<'info, PoolState>,

  pub mint_x: Account<'info, Mint>,
  pub mint_y: Account<'info, Mint>,
  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
  pub fn deposit(&mut self, lp_mint: u64, x_deposit_max: u64, y_deposit_max: u64) -> Result<()> {
    // calculate provide amount of x,y,lp

    todo!()
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
        &[self.pool_state.bump_pool_state],
    ];

    let signers_seeds_bytes = &[&seeds[..]];

    let ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers_seeds_bytes);

    mint_to(ctx, amount)
  }
}