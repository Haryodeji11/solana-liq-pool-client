use anyhow::{Ok, Result};
use solana_client::rpc_client::RpcClient;
use solana_liq_pool_client::{LiquidityPool, PoolInstruction};
use solana_sdk::{
    address_lookup_table::instruction, instruction::{AccountMeta, Instruction}, pubkey::Pubkey, signature::{Keypair, Signer}, system_instruction, sysvar::rent, transaction::Transaction
};
use solana_program::{pubkey, system_program};
use borsh::{BorshDeserialize, BorshSerialize};
use std::str::FromStr;
use shellexpand;



fn main() -> Result<()>  {
    let client = RpcClient::new("https://api.devnet.solana.com".to_string());
    let keypair_path = shellexpand::tilde("~/.config/solana/id.json").to_string();
    let keypair_data = std::fs::read_to_string(&keypair_path)?;
    let keypair_bytes: Vec<u8> = serde_json::from_str(&keypair_data)?;
    let payer = Keypair::from_bytes(&keypair_bytes)?;

    let program_id = Pubkey::from_str("2SRp5ENH631KzuRGNXWcdLi59pnvKNNoTm37pMRcBH3Q")?;

    let token_a_mint = Pubkey::from_str("CgVsKEMPtwjvF9PWpHrRU6jceL332rBoKwp3cHW9uJDQ").expect("invalid Token a mint pub key");
    let token_b_mint = Pubkey::from_str("EJdfrzS9H9kk1NBLnkNhMepmJrqUMEgnqybaVxUHi8xK").expect("invalid Token b mint pub key");
    let user_token_a = Pubkey::from_str("3i2ArjeWj4UErchteD3f9KYQsx3MacgcP4fmhC3a7WFL").expect("invalid user token a pub key");
    let user_token_b = Pubkey::from_str("5FCN8mzbDQStPfUJjXL7Dna38ts7BaByAprNj8TjMSQD").expect("invalid user token b pub key");
    let token_a_vault = Pubkey::from_str("3jLdg1WL8TyaFJHH8AwTPKiubpyydhwQebe4bAG3iNox").expect("invalid Token a vault pub key");
    let token_b_vault = Pubkey::from_str("9onYUiyA6FHLpdZzJFJBnJqV9jRuVFyEmrFWcKHf9M6P").expect("invalid Token b vault pub key");
    let liquidity_mint = Pubkey::from_str("HTMV6cQXhtohtFUWoJRiNNwt7v9ML645gyqboWgcJcZ2").expect("invalid liquidity mint token");
    let user_liquidity = Pubkey::from_str("2x7dgVCeSufQMN9KPtgNG39xm32kSyPgUrDFwko75stX").expect("invalid user liquidity mint token"); 


    // creating pool
    let pool_state_keypair = Keypair::new();
    let pool_state_pubkey = pool_state_keypair.pubkey();

    // testing all pool instruction
    test_initialize_pool(&client, &payer, &program_id, &pool_state_keypair, &token_a_mint, &token_b_mint, &token_a_vault, &token_b_vault)?;
    test_add_liquidity(&client, &payer, &program_id, &pool_state_pubkey, &token_a_mint, &token_b_mint, &token_a_vault, &token_b_vault, &user_token_a, &user_token_b, &liquidity_mint, &user_liquidity)?;
    test_remove_liquidity(&client, &payer, &program_id, &pool_state_pubkey, &token_a_mint, &token_b_mint, &token_a_vault, &token_b_vault, &user_token_a, &user_token_b, &liquidity_mint, &user_liquidity)?;
    test_swap(&client, &payer, &program_id, &pool_state_pubkey, &token_a_mint, &token_b_mint, &token_a_vault, &token_b_vault, &user_token_a, &user_token_b)?;
    

  
    println!("All tests passed!");
    Ok(())
}

fn test_initialize_pool(
    client: &RpcClient, 
    payer:  &Keypair,
    program_id:  &Pubkey, 
    pool_state_keypair: &Keypair, 
    token_a_mint:  &Pubkey,
    token_b_mint:  &Pubkey, 
    token_a_vault:  &Pubkey, 
    token_b_vault: &Pubkey ) -> Result<()>{

    let pool_state_pubkey = pool_state_keypair.pubkey();
    // liquidity size
    let lamports = client.get_minimum_balance_for_rent_exemption(152)?;

    let (authority, _bump) = Pubkey::find_program_address(&[b"authority", pool_state_pubkey.as_ref()], program_id);

    // initiazlizing instruction

    let accounts = vec![
        AccountMeta::new(pool_state_pubkey, true),
        AccountMeta::new_readonly(authority, false), 
        AccountMeta::new_readonly(*token_a_mint, false),
        AccountMeta::new_readonly(*token_b_mint, false),
        AccountMeta::new(*token_a_vault, false),
        AccountMeta::new(*token_b_vault, false),
        AccountMeta::new_readonly(solana_program::pubkey::Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")?, false),
    ];

    let instruction_data = PoolInstruction::InitializePool.try_to_vec()?;
    let instruction = Instruction {
        program_id: *program_id, 
        accounts, 
        data: instruction_data 
    };

    let create_account_ix = system_instruction::create_account(
        &payer.pubkey(), 
        &pool_state_pubkey, 
        lamports, 
        152, 
        program_id
    );



    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[create_account_ix, instruction],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );

    let signature = client.send_and_confirm_transaction(&transaction)?;
    println!("InitializePool succeeded! Signature: {}", signature);
    
    // Verifying pool

    let pool_data = client.get_account_data(&pool_state_pubkey)?;
    let pool: LiquidityPool = LiquidityPool::try_from_slice(&pool_data)?;
    assert_eq!(pool.authority, authority);
    assert_eq!(pool.token_a_mint, *token_a_mint);
    assert_eq!(pool.token_b_mint, *token_b_mint);
    assert_eq!(pool.liquidity_supply, 0);
    println!("Pool state verified: {:?}", pool);

    
    Ok(())
}

fn test_add_liquidity(
    client: &RpcClient,
    payer: &Keypair,
    program_id: &Pubkey,
    pool_state_pubkey: &Pubkey,
    token_a_mint: &Pubkey,
    token_b_mint: &Pubkey,
    token_a_vault: &Pubkey,
    token_b_vault: &Pubkey,
    user_token_a: &Pubkey,
    user_token_b: &Pubkey,
    liquidity_mint: &Pubkey,
    user_liquidity: &Pubkey,
) -> Result<()> {
    // Build AddLiquidity instruction
    let accounts = vec![
        AccountMeta::new(*pool_state_pubkey, false),
        AccountMeta::new(*user_token_a, false),
        AccountMeta::new(*user_token_b, false),
        AccountMeta::new(*token_a_vault, false),
        AccountMeta::new(*token_b_vault, false),
        AccountMeta::new(*liquidity_mint, false),
        AccountMeta::new(*user_liquidity, false),
        AccountMeta::new_readonly(solana_program::pubkey::Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")?, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    let instruction_data = PoolInstruction::AddLiquidity {
        amount_a: 1000,
        amount_b: 1000,
    }.try_to_vec()?;
    let instruction = Instruction {
        program_id: *program_id,
        accounts,
        data: instruction_data,
    };

    // Send transaction
    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );
    let signature = client.send_and_confirm_transaction(&transaction)?;
    println!("AddLiquidity succeeded! Signature: {}", signature);

    // Verify pool state
    let pool_data = client.get_account_data(pool_state_pubkey)?;
    let pool: LiquidityPool = LiquidityPool::try_from_slice(&pool_data)?;
    assert_eq!(pool.token_a_reserves, 1000);
    assert_eq!(pool.token_b_reserves, 1000);
    assert!(pool.liquidity_supply > 0);
    println!("Pool state after AddLiquidity: {:?}", pool);

    Ok(())
}

fn test_remove_liquidity(
    client: &RpcClient,
    payer: &Keypair,
    program_id: &Pubkey,
    pool_state_pubkey: &Pubkey,
    token_a_mint: &Pubkey,
    token_b_mint: &Pubkey,
    token_a_vault: &Pubkey,
    token_b_vault: &Pubkey,
    user_token_a: &Pubkey,
    user_token_b: &Pubkey,
    liquidity_mint: &Pubkey,
    user_liquidity: &Pubkey,
) -> Result<()> {
    // Build RemoveLiquidity instruction
    let accounts = vec![
        AccountMeta::new(*pool_state_pubkey, false),
        AccountMeta::new(*user_liquidity, false),
        AccountMeta::new(*token_a_vault, false),
        AccountMeta::new(*token_b_vault, false),
        AccountMeta::new(*user_token_a, false),
        AccountMeta::new(*user_token_b, false),
        AccountMeta::new(*liquidity_mint, false),
        AccountMeta::new_readonly(solana_program::pubkey::Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")?, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    let instruction_data = PoolInstruction::RemoveLiquidity {
        liquidity_amount: 500, // Burn half the liquidity tokens
    }.try_to_vec()?;
    let instruction = Instruction {
        program_id: *program_id,
        accounts,
        data: instruction_data,
    };

    // Send transaction
    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );
    let signature = client.send_and_confirm_transaction(&transaction)?;
    println!("RemoveLiquidity succeeded! Signature: {}", signature);

    // Verify pool state
    let pool_data = client.get_account_data(pool_state_pubkey)?;
    let pool: LiquidityPool = LiquidityPool::try_from_slice(&pool_data)?;
    assert_eq!(pool.token_a_reserves, 500); 
    assert_eq!(pool.token_b_reserves, 500); 
    println!("Pool state after RemoveLiquidity: {:?}", pool);

    Ok(())
}

fn test_swap(
    client: &RpcClient,
    payer: &Keypair,
    program_id: &Pubkey,
    pool_state_pubkey: &Pubkey,
    _token_a_mint: &Pubkey,
    _token_b_mint: &Pubkey,
    token_a_vault: &Pubkey,
    token_b_vault: &Pubkey,
    user_token_a: &Pubkey,
    user_token_b: &Pubkey,
) -> Result<()> {
    // Build Swap instruction (A to B)
    let accounts = vec![
        AccountMeta::new(*pool_state_pubkey, false),
        AccountMeta::new(*user_token_a, false),
        AccountMeta::new(*user_token_b, false),
        AccountMeta::new(*token_a_vault, false),
        AccountMeta::new(*token_b_vault, false),
        AccountMeta::new_readonly(solana_program::pubkey::Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")?, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
    ];
    let instruction_data = PoolInstruction::Swap {
        amount_in: 100,
        a_to_b: true,
    }.try_to_vec()?;
    let instruction = Instruction {
        program_id: *program_id,
        accounts,
        data: instruction_data,
    };

    // Send transaction
    let recent_blockhash = client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );
    let signature = client.send_and_confirm_transaction(&transaction)?;
    println!("Swap succeeded! Signature: {}", signature);

    // Verify pool state
    let pool_data = client.get_account_data(pool_state_pubkey)?;
    let pool: LiquidityPool = LiquidityPool::try_from_slice(&pool_data)?;
    assert!(pool.token_a_reserves > 500); 
    assert!(pool.token_b_reserves < 500); 
    println!("Pool state after Swap: {:?}", pool);

    Ok(())
}