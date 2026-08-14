use soroban_sdk::{Env, Bytes, BytesN, Vec, symbol_short};

// Define tree depth
pub const TREE_DEPTH: u32 = 20;

pub fn get_zero_value(env: &Env, level: u32) -> BytesN<32> {
    let mut current = BytesN::from_array(env, &[0u8; 32]);
    for _ in 0..level {
        current = hash_left_right(env, &current, &current);
    }
    current
}

pub fn hash_left_right(env: &Env, left: &BytesN<32>, right: &BytesN<32>) -> BytesN<32> {
    let mut bytes = Bytes::new(env);
    bytes.append(&left.clone().into());
    bytes.append(&right.clone().into());
    env.crypto().sha256(&bytes).into()
}

pub struct MerkleTree;

impl MerkleTree {
    pub fn init(env: &Env) {
        if !env.storage().persistent().has(&symbol_short!("M_INDEX")) {
            env.storage().persistent().set(&symbol_short!("M_INDEX"), &0u32);

            let mut filled = Vec::new(env);
            for i in 0..TREE_DEPTH {
                filled.push_back(get_zero_value(env, i));
            }
            env.storage().persistent().set(&symbol_short!("M_SUBS"), &filled);

            let initial_root = get_zero_value(env, TREE_DEPTH);
            env.storage().persistent().set(&symbol_short!("M_ROOT"), &initial_root);
        }
    }

    pub fn get_current_root(env: &Env) -> BytesN<32> {
        env.storage().persistent().get(&symbol_short!("M_ROOT")).unwrap_or_else(|| get_zero_value(env, TREE_DEPTH))
    }

    pub fn get_next_index(env: &Env) -> u32 {
        env.storage().persistent().get(&symbol_short!("M_INDEX")).unwrap_or(0)
    }

    pub fn insert(env: &Env, leaf: BytesN<32>) -> BytesN<32> {
        Self::init(env);

        let mut next_index: u32 = env.storage().persistent().get(&symbol_short!("M_INDEX")).unwrap();
        let max_leaves = 2u32.pow(TREE_DEPTH);
        if next_index >= max_leaves {
            panic!("Merkle Tree is full");
        }

        let mut filled_subtrees: Vec<BytesN<32>> = env.storage().persistent().get(&symbol_short!("M_SUBS")).unwrap();
        let mut current_level_hash = leaf;
        let mut index = next_index;

        for i in 0..TREE_DEPTH {
            if index % 2 == 1 {
                let left = filled_subtrees.get(i).unwrap();
                current_level_hash = hash_left_right(env, &left, &current_level_hash);
            } else {
                filled_subtrees.set(i, current_level_hash.clone());
                current_level_hash = hash_left_right(env, &current_level_hash, &get_zero_value(env, i));
            }
            index /= 2;
        }

        next_index += 1;
        env.storage().persistent().set(&symbol_short!("M_INDEX"), &next_index);
        env.storage().persistent().set(&symbol_short!("M_SUBS"), &filled_subtrees);
        env.storage().persistent().set(&symbol_short!("M_ROOT"), &current_level_hash);

        current_level_hash
    }
}

// NOTE: Depth of 20 allows up to 2^20 (1,048,576) leaf insertions.
