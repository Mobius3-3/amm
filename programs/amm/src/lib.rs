#![allow(unexpected_cfgs)]
#![allow(deprecated)]
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("Hfr3Wu1JFvBoPRA542MUebxnsFZXMRe41koRGhPijyBQ");

#[program]
pub mod anchor_escrow {
    use super::*;

}
