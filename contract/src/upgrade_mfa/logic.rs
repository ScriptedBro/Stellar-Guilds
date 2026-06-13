use soroban_sdk::{Address, Env, String};

use crate::upgrade_mfa::storage;
use crate::upgrade_mfa::types::{UpgradeMfaError, UpgradePendingState};

/// How long (in seconds) the second signature window stays open after the first.
/// Default: 1 hour.  Adjust as needed.
pub const SECOND_SIG_WINDOW_SECS: u64 = 3_600;

// ─── Setup ────────────────────────────────────────────────────────────────────

/// Register the primary admin and the backup signing key.
///
/// Should be called once during contract initialization or by a governance
/// proposal.  Both addresses must sign so neither can be set unilaterally.
pub fn initialize_mfa(env: &Env, admin: &Address, backup_key: &Address) {
    admin.require_auth();
    backup_key.require_auth();
    storage::set_admin(env, admin);
    storage::set_backup_key(env, backup_key);
}

// ─── Step 1: Admin proposes upgrade ──────────────────────────────────────────

/// Record the first signature for a sensitive upgrade action.
///
/// Sets `upgrade_pending = true` and stores the proposal metadata.
/// The second signature from `backup_key` must arrive within
/// [`SECOND_SIG_WINDOW_SECS`] seconds or the proposal expires.
///
/// # Errors
/// - [`UpgradeMfaError::NotInitialized`] — MFA not set up yet.
/// - [`UpgradeMfaError::NotAdmin`] — `caller` is not the registered admin.
/// - [`UpgradeMfaError::AlreadyPending`] — a previous proposal is still open.
pub fn upgrade_first_signature(
    env: &Env,
    caller: &Address,
    description: String,
) -> Result<(), UpgradeMfaError> {
    caller.require_auth();

    let admin = storage::get_admin(env).ok_or(UpgradeMfaError::NotInitialized)?;
    if *caller != admin {
        return Err(UpgradeMfaError::NotAdmin);
    }

    if storage::is_pending(env) {
        // Lazily check expiry before rejecting: if the previous proposal
        // has already expired we allow a fresh one to be created.
        if let Some(pending) = storage::get_pending(env) {
            if env.ledger().timestamp() <= pending.expires_at {
                return Err(UpgradeMfaError::AlreadyPending);
            }
            // Expired — silently clear and proceed.
            storage::clear_pending(env);
        }
    }

    let expires_at = env.ledger().timestamp() + SECOND_SIG_WINDOW_SECS;
    storage::store_pending(
        env,
        &UpgradePendingState {
            proposed_by: caller.clone(),
            expires_at,
            description,
        },
    );

    Ok(())
}

// ─── Step 2: Backup key confirms upgrade ─────────────────────────────────────

/// Provide the second (backup-key) signature to complete a pending upgrade.
///
/// Executes the upgrade if the proposal is still within the time window.
/// Clears `upgrade_pending` regardless of outcome (success or expiry).
///
/// # Errors
/// - [`UpgradeMfaError::NotInitialized`] — MFA not set up yet.
/// - [`UpgradeMfaError::NotBackupKey`] — `caller` is not the registered backup key.
/// - [`UpgradeMfaError::NoPendingUpgrade`] — no proposal is open.
/// - [`UpgradeMfaError::UpgradeExpired`] — the proposal's time window closed.
pub fn upgrade_second_signature(
    env: &Env,
    caller: &Address,
) -> Result<UpgradePendingState, UpgradeMfaError> {
    caller.require_auth();

    let backup = storage::get_backup_key(env).ok_or(UpgradeMfaError::NotInitialized)?;
    if *caller != backup {
        return Err(UpgradeMfaError::NotBackupKey);
    }

    let pending = storage::get_pending(env).ok_or(UpgradeMfaError::NoPendingUpgrade)?;

    // Time-window check — clear regardless so expired proposals are GC'd.
    if env.ledger().timestamp() > pending.expires_at {
        storage::clear_pending(env);
        return Err(UpgradeMfaError::UpgradeExpired);
    }

    // Both signatures collected — clear pending flag and return the approved state.
    storage::clear_pending(env);
    Ok(pending)
}

// ─── Query ────────────────────────────────────────────────────────────────────

/// Returns whether an upgrade is currently pending a second signature.
pub fn is_upgrade_pending(env: &Env) -> bool {
    if !storage::is_pending(env) {
        return false;
    }
    // Lazily expire
    if let Some(pending) = storage::get_pending(env) {
        if env.ledger().timestamp() > pending.expires_at {
            storage::clear_pending(env);
            return false;
        }
    }
    true
}

/// Returns the pending upgrade proposal if one is active and not expired.
pub fn get_pending_upgrade(env: &Env) -> Option<UpgradePendingState> {
    let pending = storage::get_pending(env)?;
    if env.ledger().timestamp() > pending.expires_at {
        storage::clear_pending(env);
        return None;
    }
    Some(pending)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::StellarGuildsContract;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env, String};

    fn setup() -> (Env, Address, Address, Address) {
        let env = Env::default();
        env.budget().reset_unlimited();
        env.mock_all_auths();
        let contract = env.register_contract(None, StellarGuildsContract);
        let admin = Address::generate(&env);
        let backup = Address::generate(&env);
        (env, contract, admin, backup)
    }

    fn init(env: &Env, contract: &Address, admin: &Address, backup: &Address) {
        env.as_contract(contract, || {
            initialize_mfa(env, admin, backup);
        });
    }

    fn desc(env: &Env, s: &str) -> String {
        String::from_str(env, s)
    }

    // ── Happy path ────────────────────────────────────────────────────────────

    /// Full two-step flow: admin proposes, backup confirms within the window.
    #[test]
    fn test_two_step_upgrade_success() {
        let (env, contract, admin, backup) = setup();
        init(&env, &contract, &admin, &backup);

        env.as_contract(&contract, || {
            // Step 1 — admin submits first signature
            assert_eq!(
                upgrade_first_signature(&env, &admin, desc(&env, "upgrade to v2.0")),
                Ok(())
            );
            assert!(is_upgrade_pending(&env), "upgrade should be pending after step 1");

            // Step 2 — backup key confirms
            let result = upgrade_second_signature(&env, &backup);
            assert!(result.is_ok(), "second signature should succeed");

            let approved = result.unwrap();
            assert_eq!(approved.proposed_by, admin);
            assert!(!is_upgrade_pending(&env), "pending flag should be cleared after step 2");
        });
    }

    /// Second signature from a non-backup address must be rejected.
    #[test]
    fn test_second_sig_wrong_caller_rejected() {
        let (env, contract, admin, backup) = setup();
        init(&env, &contract, &admin, &backup);
        let impostor = Address::generate(&env);

        env.as_contract(&contract, || {
            upgrade_first_signature(&env, &admin, desc(&env, "upgrade")).unwrap();
            let err = upgrade_second_signature(&env, &impostor).unwrap_err();
            assert_eq!(err, UpgradeMfaError::NotBackupKey);
            // Pending state untouched
            assert!(is_upgrade_pending(&env));
        });
    }

    /// First signature from a non-admin address must be rejected.
    #[test]
    fn test_first_sig_wrong_caller_rejected() {
        let (env, contract, admin, backup) = setup();
        init(&env, &contract, &admin, &backup);
        let impostor = Address::generate(&env);

        env.as_contract(&contract, || {
            let err = upgrade_first_signature(&env, &impostor, desc(&env, "attack")).unwrap_err();
            assert_eq!(err, UpgradeMfaError::NotAdmin);
            assert!(!is_upgrade_pending(&env));
        });
    }

    // ── Expiry ────────────────────────────────────────────────────────────────

    /// Second signature after the expiry window must return `UpgradeExpired`
    /// and clear the pending flag.
    #[test]
    fn test_second_sig_after_expiry_rejected() {
        let (env, contract, admin, backup) = setup();
        init(&env, &contract, &admin, &backup);

        env.as_contract(&contract, || {
            upgrade_first_signature(&env, &admin, desc(&env, "upgrade")).unwrap();

            // Advance ledger time past the window.
            env.ledger().with_mut(|li| {
                li.timestamp = li.timestamp + SECOND_SIG_WINDOW_SECS + 1;
            });

            let err = upgrade_second_signature(&env, &backup).unwrap_err();
            assert_eq!(err, UpgradeMfaError::UpgradeExpired);
            assert!(!is_upgrade_pending(&env), "expired proposal should be cleared");
        });
    }

    /// After an expired proposal is cleared, a new proposal can be created.
    #[test]
    fn test_new_proposal_allowed_after_expiry() {
        let (env, contract, admin, backup) = setup();
        init(&env, &contract, &admin, &backup);

        env.as_contract(&contract, || {
            upgrade_first_signature(&env, &admin, desc(&env, "first attempt")).unwrap();

            // Expire the first proposal.
            env.ledger().with_mut(|li| {
                li.timestamp = li.timestamp + SECOND_SIG_WINDOW_SECS + 1;
            });

            // A fresh proposal should succeed (expired one is lazily cleared).
            assert_eq!(
                upgrade_first_signature(&env, &admin, desc(&env, "second attempt")),
                Ok(())
            );
        });
    }

    // ── Guard rails ───────────────────────────────────────────────────────────

    /// Proposing when a non-expired proposal is already pending must return
    /// `AlreadyPending`.
    #[test]
    fn test_duplicate_proposal_rejected() {
        let (env, contract, admin, backup) = setup();
        let _ = backup;
        init(&env, &contract, &admin, &backup);

        env.as_contract(&contract, || {
            upgrade_first_signature(&env, &admin, desc(&env, "first")).unwrap();
            let err =
                upgrade_first_signature(&env, &admin, desc(&env, "second")).unwrap_err();
            assert_eq!(err, UpgradeMfaError::AlreadyPending);
        });
    }

    /// Calling `upgrade_second_signature` when no proposal exists must return
    /// `NoPendingUpgrade`.
    #[test]
    fn test_second_sig_without_first_sig_rejected() {
        let (env, contract, admin, backup) = setup();
        init(&env, &contract, &admin, &backup);

        env.as_contract(&contract, || {
            let err = upgrade_second_signature(&env, &backup).unwrap_err();
            assert_eq!(err, UpgradeMfaError::NoPendingUpgrade);
        });
    }

    /// Calling either function before `initialize_mfa` must return
    /// `NotInitialized`.
    #[test]
    fn test_operations_without_initialization_rejected() {
        let env = Env::default();
        env.budget().reset_unlimited();
        env.mock_all_auths();
        let contract = env.register_contract(None, StellarGuildsContract);
        let caller = Address::generate(&env);

        env.as_contract(&contract, || {
            assert_eq!(
                upgrade_first_signature(&env, &caller, desc(&env, "x")),
                Err(UpgradeMfaError::NotInitialized)
            );
            assert_eq!(
                upgrade_second_signature(&env, &caller),
                Err(UpgradeMfaError::NotInitialized)
            );
        });
    }
}