use soroban_sdk::{contracttype, Address};

/// Error variants for the two-step upgrade MFA flow.
#[soroban_sdk::contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum UpgradeMfaError {
    /// Caller is not the primary admin address.
    NotAdmin = 1,
    /// Caller is not the secondary backup key address.
    NotBackupKey = 2,
    /// `upgrade_second_signature` was called but no pending upgrade exists.
    NoPendingUpgrade = 3,
    /// `upgrade_second_signature` was called but the proposal has expired.
    UpgradeExpired = 4,
    /// The upgrade system has not been initialized (no admin set).
    NotInitialized = 5,
    /// A pending upgrade already exists; cancel it before proposing a new one.
    AlreadyPending = 6,
}

/// Persisted state of a two-step upgrade proposal.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct UpgradePendingState {
    /// The admin address that submitted the first signature.
    pub proposed_by: Address,
    /// Ledger timestamp after which the second signature window closes.
    pub expires_at: u64,
    /// Free-form description of what is being upgraded (for audit).
    pub description: soroban_sdk::String,
}
