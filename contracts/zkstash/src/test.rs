#![cfg(test)]
use super::*;
use soroban_sdk::{Env, Address, BytesN, Bytes, testutils::Address as _};

#[test]
fn test_deposit_and_withdrawal_flow() {
    let env = Env::default();
    env.mock_all_auths();

    // 1. Register our vault contract
    let contract_id = env.register(ZkStashVault, ());
    let client = ZkStashVaultClient::new(&env, &contract_id);

    // 2. Register mock Stellar Asset Contract (SAC) representing the token
    let admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_id = token_contract.address();
    let token_client = soroban_sdk::token::Client::new(&env, &token_id);
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);

    // 3. Setup accounts and mint tokens
    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let amount = 100i128;

    token_admin_client.mint(&depositor, &1000i128);
    assert_eq!(token_client.balance(&depositor), 1000i128);

    // 4. Perform deposit
    let commitment = BytesN::from_array(&env, &[1u8; 32]);
    client.deposit(&depositor, &token_id, &commitment, &amount);

    // Check balances after deposit
    assert_eq!(token_client.balance(&depositor), 900i128);
    assert_eq!(token_client.balance(&contract_id), 100i128);

    // Get root after deposit
    let root = client.get_root();
    assert_ne!(root, BytesN::from_array(&env, &[0u8; 32]));

    // 5. Perform withdrawal with mock ZK proof
    let nullifier = BytesN::from_array(&env, &[2u8; 32]);
    let proof = Bytes::from_slice(&env, b"ZK_PASS_MOCK_PROOF");

    client.withdraw(&token_id, &recipient, &amount, &nullifier, &root, &proof);

    // Verify balances after withdrawal
    assert_eq!(token_client.balance(&contract_id), 0i128);
    assert_eq!(token_client.balance(&recipient), 100i128);

    // 6. Attempt double-spending with the same nullifier — should fail
    let res = client.try_withdraw(&token_id, &recipient, &amount, &nullifier, &root, &proof);
    assert!(res.is_err(), "Expected double-spending to fail");
}

#[test]
#[should_panic(expected = "Invalid Merkle root")]
fn test_withdraw_invalid_root() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ZkStashVault, ());
    let client = ZkStashVaultClient::new(&env, &contract_id);

    let token_id = Address::generate(&env);
    let recipient = Address::generate(&env);
    let amount = 100i128;
    let nullifier = BytesN::from_array(&env, &[2u8; 32]);
    let root = BytesN::from_array(&env, &[99u8; 32]); // Fake/unused root
    let proof = Bytes::from_slice(&env, b"ZK_PASS_MOCK_PROOF");

    client.withdraw(&token_id, &recipient, &amount, &nullifier, &root, &proof);
}
