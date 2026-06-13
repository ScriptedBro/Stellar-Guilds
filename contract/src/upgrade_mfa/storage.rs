use soroban_sdk::{symbol_short, Address, Env, Symbol};

use crate::upgrade_mfa::types::UpgradePendingState;

const ADMIN_KEY: Symbol = symbol_short!("mfa_admin");
const BACKUP_KEY: Symbol = symbol_short!("mfa_bkup");
const PENDING_KEY: Symbol = symbol_short!("mfa_pend");
const PENDING_FLAG_KEY: Symbol = symbol_short!("mfa_flag");

// ─── Config ───────────────────────────────────────────────────────────────────

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&ADMIN_KEY, admin);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&ADMIN_KEY)
}

pub fn set_backup_key(env: &Env, backup: &Address) {
    env.storage().persistent().set(&BACKUP_KEY, backup);
}

pub fn get_backup_key(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&BACKUP_KEY)
}

// ─── Pending-upgrade state ────────────────────────────────────────────────────

pub fn store_pending(env: &Env, state: &UpgradePendingState) {
    env.storage().persistent().set(&PENDING_KEY, state);
    env.storage().persistent().set(&PENDING_FLAG_KEY, &true);
}

pub fn get_pending(env: &Env) -> Option<UpgradePendingState> {
    env.storage().persistent().get(&PENDING_KEY)
}

pub fn is_pending(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get::<Symbol, bool>(&PENDING_FLAG_KEY)
        .unwrap_or(false)
}

pub fn clear_pending(env: &Env) {
    env.storage().persistent().remove(&PENDING_KEY);
    env.storage().persistent().set(&PENDING_FLAG_KEY, &false);
}
