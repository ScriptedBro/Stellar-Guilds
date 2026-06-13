//! Guild membership tests — join_guild
//!
//! Covers authorized self-join, duplicate join rejection, join on a
//! non-existent guild, and unauthorized join (missing signature).

#![cfg(test)]

use crate::guild::types::Role;
use crate::InitializerProof;
use crate::{StellarGuildsContract, StellarGuildsContractClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, String};

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn setup_env() -> Env {
    let env = Env::default();
    env.budget().reset_unlimited();
    env
}

fn register_and_init(env: &Env) -> Address {
    let contract_id = env.register_contract(None, StellarGuildsContract);
    let client = StellarGuildsContractClient::new(env, &contract_id);
    client.initialize(&Address::generate(env));
    contract_id
}

fn create_test_guild(client: &StellarGuildsContractClient<'_>, env: &Env, owner: &Address) -> u64 {
    client.create_guild(
        &String::from_str(env, "Test Guild"),
        &String::from_str(env, "A guild for testing"),
        owner,
        &None::<InitializerProof>,
    )
}

// ─── Tests ────────────────────────────────────────────────────────────────────

/// An address that signs its own transaction can successfully join a guild
/// and is thereafter visible as a `Role::Member` in the membership map.
#[test]
fn test_join_guild_authorized() {
    let env = setup_env();
    env.mock_all_auths();

    let contract_id = register_and_init(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let guild_id = create_test_guild(&client, &env, &owner);

    let joiner = Address::generate(&env);
    assert!(
        !client.is_member(&guild_id, &joiner),
        "should not be a member yet"
    );

    let result = client.join_guild(&guild_id, &joiner);
    assert!(result, "join_guild should return true");

    // Verify membership and role
    assert!(
        client.is_member(&guild_id, &joiner),
        "should now be a member"
    );
    let member = client.get_member(&guild_id, &joiner);
    assert_eq!(member.role, Role::Member, "default role should be Member");
    assert_eq!(member.address, joiner);

    // Verify guild member_count incremented (owner = 1, joiner = 2)
    let all = client.get_all_members(&guild_id);
    assert_eq!(all.len(), 2u32);
}

/// Joining a guild that does not exist must panic with "Guild not found".
#[test]
#[should_panic(expected = "Guild not found")]
fn test_join_nonexistent_guild_panics() {
    let env = setup_env();
    env.mock_all_auths();

    let contract_id = register_and_init(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let joiner = Address::generate(&env);
    client.join_guild(&999u64, &joiner);
}

/// Attempting to join a guild the caller has already joined must panic
/// with "Already a member of this guild".
#[test]
#[should_panic(expected = "Already a member of this guild")]
fn test_join_guild_duplicate_panics() {
    let env = setup_env();
    env.mock_all_auths();

    let contract_id = register_and_init(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let guild_id = create_test_guild(&client, &env, &owner);

    let joiner = Address::generate(&env);
    client.join_guild(&guild_id, &joiner); // first join — ok
    client.join_guild(&guild_id, &joiner); // second join — must panic
}

/// A caller who has NOT authorised the transaction must NOT be able to join.
/// Soroban's `require_auth` panics when no matching authorisation is present.
#[test]
#[should_panic]
fn test_join_guild_unauthorized_panics() {
    let env = setup_env();
    // Intentionally do NOT call env.mock_all_auths() so auth checks are live.

    let contract_id = register_and_init(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    // Neither initialize nor create_guild call require_auth, so no auth mock
    // is needed for setup. The env has no mocked auths at all, which means
    // caller.require_auth() inside join_guild will panic as expected.
    let owner = Address::generate(&env);
    let guild_id = create_test_guild(&client, &env, &owner);

    let joiner = Address::generate(&env);
    // No mock_all_auths → require_auth() inside join_guild panics.
    client.join_guild(&guild_id, &joiner);
}
