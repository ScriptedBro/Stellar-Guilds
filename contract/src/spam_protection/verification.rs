use soroban_sdk::{symbol_short, Bytes, Env, Map, Symbol};

use crate::spam_protection::types::{InitializerProof, SpamError};

/// Number of leading zero *bits* required in the proof hash.
///
/// Each additional bit doubles the expected mining work.
/// 16 bits  ≈ 65 536 attempts on average — cheap for a human, expensive for a bot.
/// Adjust via the `POW_DIFFICULTY` compile-time constant if needed.
pub const POW_LEADING_ZERO_BITS: u32 = 16;

/// Storage key for the used-nonces map.
const USED_NONCES_KEY: Symbol = symbol_short!("pow_used");

// ─── Public verification entry point ─────────────────────────────────────────

/// Verify an `InitializerProof` for the given `creator` address.
///
/// Steps:
///   1. Reject empty / missing proof.
///   2. Check that the hash has the required number of leading zero bits.
///   3. Check that the nonce has not been used before (replay protection).
///   4. Persist the nonce so it cannot be reused.
///
/// # Errors
/// Returns [`SpamError`] on any failure.  Callers should `panic!` with the
/// error code so that the Soroban runtime surfaces it as a `ContractError`.
pub fn verify_proof(env: &Env, proof: &InitializerProof) -> Result<(), SpamError> {
    // ── 1. Reject empty proof ────────────────────────────────────────────
    if proof.hash.is_empty() {
        return Err(SpamError::MissingProof);
    }

    // ── 2. Difficulty check ──────────────────────────────────────────────
    if !meets_difficulty(&proof.hash, POW_LEADING_ZERO_BITS) {
        return Err(SpamError::InvalidProof);
    }

    // ── 3. Nonce replay check ────────────────────────────────────────────
    let mut used: Map<u64, bool> = env
        .storage()
        .persistent()
        .get(&USED_NONCES_KEY)
        .unwrap_or_else(|| Map::new(env));

    if used.contains_key(proof.nonce) {
        return Err(SpamError::NonceReused);
    }

    // ── 4. Persist nonce ─────────────────────────────────────────────────
    used.set(proof.nonce, true);
    env.storage().persistent().set(&USED_NONCES_KEY, &used);

    Ok(())
}

// ─── Difficulty helper ────────────────────────────────────────────────────────

/// Returns `true` if the first `required_zero_bits` bits of `hash` are all 0.
///
/// Works on raw bytes — no SHA-256 host function is needed.
fn meets_difficulty(hash: &Bytes, required_zero_bits: u32) -> bool {
    if required_zero_bits == 0 {
        return true;
    }

    let full_zero_bytes = (required_zero_bits / 8) as usize;
    let remaining_bits = required_zero_bits % 8;

    // Check completely-zero bytes
    for i in 0..full_zero_bytes {
        let byte = hash.get(i as u32).unwrap_or(0xff);
        if byte != 0 {
            return false;
        }
    }

    // Check the partially-zero byte (high bits must be 0)
    if remaining_bits > 0 {
        let mask: u8 = 0xff_u8 << (8 - remaining_bits); // top N bits set
        let byte = hash.get(full_zero_bytes as u32).unwrap_or(0xff);
        if byte & mask != 0 {
            return false;
        }
    }

    true
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Bytes, Env};

    /// Build a `Bytes` value from a fixed-size array literal.
    fn bytes_from_slice(env: &Env, data: &[u8]) -> Bytes {
        Bytes::from_slice(env, data)
    }

    // Helper: produce a 32-byte hash with a given first byte, rest zeroed.
    fn hash_with_first_byte(env: &Env, first: u8) -> Bytes {
        let mut raw = [0u8; 32];
        raw[0] = first;
        bytes_from_slice(env, &raw)
    }

    #[test]
    fn test_meets_difficulty_all_zeros_passes() {
        let env = Env::default();
        let hash = bytes_from_slice(&env, &[0u8; 32]);
        assert!(meets_difficulty(&hash, 16));
        assert!(meets_difficulty(&hash, 24));
    }

    #[test]
    fn test_meets_difficulty_first_byte_nonzero_fails() {
        let env = Env::default();
        let hash = hash_with_first_byte(&env, 0x01);
        assert!(!meets_difficulty(&hash, 16)); // first 16 bits must be 0
    }

    #[test]
    fn test_meets_difficulty_partial_byte() {
        let env = Env::default();
        // 0x0F = 0000_1111: first 4 bits are zero → passes 4-bit requirement
        let hash = bytes_from_slice(
            &env,
            &[0x00, 0x0f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        );
        assert!(meets_difficulty(&hash, 12)); // 1 zero byte + 4 zero bits in byte[1]
        assert!(!meets_difficulty(&hash, 13)); // byte[1] bit 4 is set → fails 13-bit check
    }

    #[test]
    fn test_meets_difficulty_zero_requirement_always_passes() {
        let env = Env::default();
        let hash = bytes_from_slice(&env, &[0xff; 32]);
        assert!(meets_difficulty(&hash, 0));
    }

    #[test]
    fn test_empty_proof_returns_missing_error() {
        let env = Env::default();
        env.budget().reset_unlimited();
        let contract = env.register_contract(None, crate::StellarGuildsContract);
        env.as_contract(&contract, || {
            let proof = InitializerProof {
                hash: Bytes::new(&env),
                nonce: 42,
            };
            assert_eq!(verify_proof(&env, &proof), Err(SpamError::MissingProof));
        });
    }

    #[test]
    fn test_insufficient_difficulty_returns_invalid_proof() {
        let env = Env::default();
        env.budget().reset_unlimited();
        let contract = env.register_contract(None, crate::StellarGuildsContract);
        env.as_contract(&contract, || {
            // Hash with non-zero first byte: difficulty 16 fails.
            let proof = InitializerProof {
                hash: hash_with_first_byte(&env, 0x01),
                nonce: 1,
            };
            assert_eq!(verify_proof(&env, &proof), Err(SpamError::InvalidProof));
        });
    }

    #[test]
    fn test_valid_proof_accepted() {
        let env = Env::default();
        env.budget().reset_unlimited();
        let contract = env.register_contract(None, crate::StellarGuildsContract);
        env.as_contract(&contract, || {
            // All-zero hash passes any difficulty.
            let proof = InitializerProof {
                hash: bytes_from_slice(&env, &[0u8; 32]),
                nonce: 99,
            };
            assert_eq!(verify_proof(&env, &proof), Ok(()));
        });
    }

    #[test]
    fn test_nonce_reuse_rejected() {
        let env = Env::default();
        env.budget().reset_unlimited();
        let contract = env.register_contract(None, crate::StellarGuildsContract);
        env.as_contract(&contract, || {
            let proof = InitializerProof {
                hash: bytes_from_slice(&env, &[0u8; 32]),
                nonce: 7,
            };
            // First use succeeds.
            assert_eq!(verify_proof(&env, &proof), Ok(()));
            // Second use with same nonce fails.
            let proof2 = InitializerProof {
                hash: bytes_from_slice(&env, &[0u8; 32]),
                nonce: 7,
            };
            assert_eq!(verify_proof(&env, &proof2), Err(SpamError::NonceReused));
        });
    }

    #[test]
    fn test_different_nonces_both_accepted() {
        let env = Env::default();
        env.budget().reset_unlimited();
        let contract = env.register_contract(None, crate::StellarGuildsContract);
        env.as_contract(&contract, || {
            let proof_a = InitializerProof {
                hash: bytes_from_slice(&env, &[0u8; 32]),
                nonce: 100,
            };
            let proof_b = InitializerProof {
                hash: bytes_from_slice(&env, &[0u8; 32]),
                nonce: 101,
            };
            assert_eq!(verify_proof(&env, &proof_a), Ok(()));
            assert_eq!(verify_proof(&env, &proof_b), Ok(()));
        });
    }
}
