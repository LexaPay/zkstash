#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Env, Address, BytesN, Bytes, Vec, symbol_short, xdr::ToXdr};

mod merkle;
mod verifier;

use merkle::MerkleTree;
use verifier::ZkVerifier;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Paused,
    FeeBps,
    AccruedFees,
}

#[contract]
pub struct ZkStashVault;

#[contractimpl]
impl ZkStashVault {
    /// Initializes the vault with an admin and protocol withdrawal fee in basis points.
    pub fn initialize(env: Env, admin: Address, fee_bps: u32) {
        if env.storage().persistent().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        assert!(fee_bps <= 1000, "Fee cannot exceed 10%");
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Paused, &false);
        env.storage().persistent().set(&DataKey::FeeBps, &fee_bps);
        env.storage().persistent().set(&DataKey::AccruedFees, &0i128);
    }

    /// Deposits tokens into the vault and registers a ZK commitment hash.
    pub fn deposit(
        env: Env,
        caller: Address,
        token: Address,
        commitment: BytesN<32>,
        amount: i128,
    ) {
        assert!(amount > 0, "Amount must be positive");
        assert!(!Self::is_paused(&env), "Contract is paused");
        caller.require_auth();

        // Transfer tokens from the caller to the vault
        let token_client = soroban_sdk::token::Client::new(&env, &token);
        token_client.transfer(&caller, &env.current_contract_address(), &amount);

        // Insert the commitment into the Merkle Tree
        let new_root = MerkleTree::insert(&env, commitment.clone());

        // Store the new root in the valid roots history
        let root_key = (symbol_short!("ROOT"), new_root.clone());
        env.storage().persistent().set(&root_key, &true);

        // Emit deposit event
        env.events().publish(
            (symbol_short!("deposit"), commitment),
            (caller, token, amount, new_root),
        );
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
        assert!(!Self::is_paused(&env), "Contract is paused");

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
        public_inputs.push_back(root.clone());
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

        // Calculate and deduct protocol fee
        let fee_bps: u32 = env.storage().persistent().get(&DataKey::FeeBps).unwrap_or(0);
        let fee_amount = (amount * fee_bps as i128) / 10000;
        let withdraw_amount = amount - fee_amount;

        if fee_amount > 0 {
            let mut accrued: i128 = env.storage().persistent().get(&DataKey::AccruedFees).unwrap_or(0);
            accrued += fee_amount;
            env.storage().persistent().set(&DataKey::AccruedFees, &accrued);
        }

        // Transfer the tokens to the recipient
        let token_client = soroban_sdk::token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &recipient, &withdraw_amount);

        // Emit withdrawal event
        env.events().publish(
            (symbol_short!("withdraw"), nullifier),
            (recipient, token, withdraw_amount, fee_amount),
        );
    }

    /// Exposes a public getter to check if contract is paused.
    pub fn is_paused(env: &Env) -> bool {
        env.storage().persistent().get(&DataKey::Paused).unwrap_or(false)
    }

    /// Pause or unpause contract deposits and withdrawals (Admin only).
    pub fn set_paused(env: Env, admin: Address, paused: bool) {
        Self::require_admin(&env, &admin);
        env.storage().persistent().set(&DataKey::Paused, &paused);
    }

    /// Claim accrued protocol fees (Admin only).
    pub fn claim_fees(env: Env, admin: Address, token: Address, recipient: Address) {
        Self::require_admin(&env, &admin);
        let accrued: i128 = env.storage().persistent().get(&DataKey::AccruedFees).unwrap_or(0);
        assert!(accrued > 0, "No fees accrued");

        env.storage().persistent().set(&DataKey::AccruedFees, &0i128);

        let token_client = soroban_sdk::token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &recipient, &accrued);
    }

    /// Gets the current Merkle Tree root.
    pub fn get_root(env: Env) -> BytesN<32> {
        MerkleTree::get_current_root(&env)
    }

    /// Gets the total number of commitments deposited in the Merkle Tree.
    pub fn get_commitment_count(env: Env) -> u32 {
        MerkleTree::get_next_index(&env)
    }

    /// Upgrades the contract WASM code to a new version (Admin only).
    pub fn upgrade(env: Env, admin: Address, new_wasm_hash: BytesN<32>) {
        Self::require_admin(&env, &admin);
        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    /// Transfers contract administration to a new address (Admin only).
    pub fn change_admin(env: Env, admin: Address, new_admin: Address) {
        Self::require_admin(&env, &admin);
        env.storage().persistent().set(&DataKey::Admin, &new_admin);
    }

    /// Updates the protocol withdrawal fee in basis points (Admin only).
    pub fn update_fee_bps(env: Env, admin: Address, new_fee_bps: u32) {
        Self::require_admin(&env, &admin);
        assert!(new_fee_bps <= 1000, "Fee cannot exceed 10%");
        env.storage().persistent().set(&DataKey::FeeBps, &new_fee_bps);
    }

    // Helpers
    fn require_admin(env: &Env, caller: &Address) {
        caller.require_auth();
        let admin: Address = env.storage().persistent().get(&DataKey::Admin).expect("Not initialized");
        assert!(*caller == admin, "Unauthorized");
    }
}

#[cfg(test)]
mod test;
pub mod version;
