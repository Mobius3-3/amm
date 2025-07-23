use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PoolState {
  pub seed: u64,
  pub authority: Option<Pubkey>,
  pub mint_x: Pubkey,
  pub mint_y: Pubkey,
  pub fee: u16, // minumum denomination: 1 / 10000
  pub locked: bool,
  pub bump: u8,
  pub bump_mint_lp: u8,
}