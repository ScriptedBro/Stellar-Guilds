use soroban_sdk::{contracterror, contracttype, Bytes};

/// Error returned when guild initialization proof is missing or invalid.
///
/// Returned as a contract panic with a well-known discriminant so callers
/// can distinguish spam rejections from other errors.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SpamError {
    /// `initializer_proof` was not provided (empty bytes).
    MissingProof = 1,
    /// The supplied proof hash does not meet the required difficulty.
    InvalidProof = 2,
    /// The nonce encoded in the proof has already been used.
    NonceReused = 3,
}

/// Proof-of-Work token submitted alongside guild initialization.
///
/// The proof is a raw SHA-256 hash of `(sender_address_bytes || nonce_bytes)`
/// that must begin with at least `POW_LEADING_ZERO_BITS` zero bits.
///
/// # On-chain computation note
/// The contract does **not** compute the hash itself (no SHA-256 host function
/// is exposed by Soroban at this time).  Instead it verifies that the supplied
/// `hash` satisfies the difficulty requirement and that the nonce has not been
/// used before.  The client (off-chain) performs the actual mining.
#[contracttype]
#[derive(Clone, Debug)]
pub struct InitializerProof {
    /// SHA-256( address_bytes || nonce ) produced off-chain.
    pub hash: Bytes,
    /// Unique counter chosen by the miner; stored on-chain to prevent reuse.
    pub nonce: u64,
}
