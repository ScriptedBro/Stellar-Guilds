/// Anti-spam Proof-of-Work gate for guild initialization.
///
/// Provides [`verification::verify_proof`] which is called inside
/// `create_guild` to reject bot-driven guild-creation floods.
pub mod types;
pub mod verification;

pub use types::{InitializerProof, SpamError};
pub use verification::verify_proof;
