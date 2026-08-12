#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Address, BytesN, Bytes, Vec, symbol_short, xdr::ToXdr};

mod merkle;
mod verifier;

use merkle::MerkleTree;
use verifier::ZkVerifier;

#[contract]
pub struct ZkStashVault;

#[contractimpl]
impl ZkStashVault {
    /// Deposits tokens into the vault and registers a ZK commitment hash.
    pub fn deposit(
        env: Env,
        caller: Address,
        token: Address,
        commitment: BytesN<32>,
        amount: i128,
    ) {
        assert!(amount > 0, "Amount must be positive");
        caller.require_auth();

        // Transfer tokens from the caller to the vault
        let token_client = soroban_sdk::token::Client::new(&env, &token);
        token_client.transfer(&caller, &env.current_contract_address(), &amount);

        // Insert the commitment into the Merkle Tree
        let new_root = MerkleTree::insert(&env, commitment);

        // Store the new root in the valid roots history
        let root_key = (symbol_short!("ROOT"), new_root);
        env.storage().persistent().set(&root_key, &true);
    }

    /// Withdraws tokens to a clean recipient address using a ZK proof.
    pub fn withdraw(
        env: Env,
        token: Address,
        recipient: Address,
        amount: i128,
        nullifier: BytesN<32>,
        root: BytesN<32>,
        proof: Bytes,
    ) {
        assert!(amount > 0, "Amount must be positive");

        // Ensure nullifier has not been spent yet
        let nullifier_key = (symbol_short!("NULL"), nullifier.clone());
        if env.storage().persistent().has(&nullifier_key) {
            panic!("Nullifier already spent");
        }

        // Verify the root is a valid historical root
        let root_key = (symbol_short!("ROOT"), root.clone());
        if !env.storage().persistent().has(&root_key) {
            panic!("Invalid Merkle root");
        }

        // Construct public inputs for the ZK proof
        let mut public_inputs = Vec::new(&env);
        public_inputs.push_back(root);
        public_inputs.push_back(nullifier.clone());

        // Hash parameters (recipient, token, amount) to bind them to this proof
        let mut params = Bytes::new(&env);
        params.append(&recipient.clone().to_xdr(&env));
        params.append(&token.clone().to_xdr(&env));

        let mut amount_bytes = [0u8; 16];
        let amount_u128 = amount as u128;
        amount_bytes.copy_from_slice(&amount_u128.to_be_bytes());
        params.append(&Bytes::from_slice(&env, &amount_bytes));

        let param_hash: BytesN<32> = env.crypto().sha256(&params).into();
        public_inputs.push_back(param_hash);

        // Verify the ZK proof
        let is_valid = ZkVerifier::verify_proof(&env, &proof, &public_inputs);
        if !is_valid {
            panic!("Invalid ZK proof");
        }

        // Mark nullifier as spent
        env.storage().persistent().set(&nullifier_key, &true);

        // Transfer the tokens to the recipient
        let token_client = soroban_sdk::token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &recipient, &amount);
    }

    /// Gets the current Merkle Tree root.
    pub fn get_root(env: Env) -> BytesN<32> {
        MerkleTree::get_current_root(&env)
    }
}

#[cfg(test)]
mod test;
