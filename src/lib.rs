use borsh::{BorshSerialize, BorshDeserialize};
use solana_program::pubkey::Pubkey;
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct LiquidityPool{
    pub authority: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub liquidity_supply: u64,
    pub token_a_reserves: u64,
    pub token_b_reserves: u64,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum PoolInstruction {
    InitializePool,
    AddLiquidity { amount_a: u64, amount_b: u64 },
    RemoveLiquidity { liquidity_amount: u64 },
    Swap { amount_in: u64, a_to_b: bool },
}