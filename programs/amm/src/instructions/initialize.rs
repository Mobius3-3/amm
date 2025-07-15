use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount}};

use crate::state::PoolState;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Initialize<'info> {
  #[account(mut)]
  pub initializer: Signer<'info>,

  #[account(
    init,
    payer = initializer,
    seeds = [b"lp", pool_state.key().as_ref()],
    bump,
    mint::decimals = 6,
    mint::authority = pool_state,
  )]
  pub mint_lp: Account<'info, Mint>,

  #[account(
    init,
    payer = initializer,
    associated_token::mint = mint_x,
    associated_token::authority = pool_state,
  )]
  pub vault_x_ata: Account<'info, TokenAccount>,

  #[account(
    init,
    payer = initializer,
    associated_token::mint = mint_y,
    associated_token::authority = pool_state,
  )]
  pub vault_y_ata: Account<'info, TokenAccount>,

  #[account(
    init,
    payer = initializer,
    seeds = [b"pool_state",seed.to_le_bytes().as_ref()],
    bump,
    space = 8 + PoolState::INIT_SPACE,  
  )]
  pub pool_state: Account<'info, PoolState>,

  pub mint_x: Account<'info, Mint>,
  pub mint_y: Account<'info, Mint>,
  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
  pub fn initialize(&mut self, seed: u64, fee: u16, authority: Option<Pubkey>, bumps: InitializeBumps) {
    self.pool_state.set_inner(
      PoolState {
        seed,
        authority,
        fee,
        mint_x: self.mint_x.key(),
        mint_y: self.mint_y.key(),
        mint_lp: self.mint_lp.key(),
        locked: false,
        bump_pool_state: bumps.pool_state,
        bump_mint_lp: bumps.mint_lp,
      }
    )
  }
}