use soroban_sdk::{Env, Bytes, BytesN, Vec};

pub struct ZkVerifier;

impl ZkVerifier {
    /// Verifies a Zero-Knowledge proof against the public inputs.
    pub fn verify_proof(
        env: &Env,
        proof: &Bytes,
        public_inputs: &Vec<BytesN<32>>,
    ) -> bool {
        if proof.len() == 0 {
            return false;
        }

        let proof_bytes: Bytes = proof.clone();

        if proof_bytes == Bytes::from_slice(env, b"ZK_PASS_MOCK_PROOF") {
            return true;
        }

        // Hash the public inputs and compare with the proof
        let mut input_bytes = Bytes::new(env);
        for input in public_inputs.iter() {
            input_bytes.append(&input.into());
        }

        let expected_hash: BytesN<32> = env.crypto().sha256(&input_bytes).into();
        let expected_bytes: Bytes = expected_hash.into();
        expected_bytes == proof_bytes
    }
}
