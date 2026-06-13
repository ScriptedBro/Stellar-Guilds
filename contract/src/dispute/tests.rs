//! Dispute Resolution Contract Tests
use crate::dispute::types::VoteDecision;
use crate::guild::types::Role;
use crate::InitializerProof;
use crate::StellarGuildsContract;
use crate::StellarGuildsContractClient;
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{token, Address, Env, String, Vec};

fn setup_env() -> Env {
    let env = Env::default();
    env.budget().reset_unlimited();
    env
}

fn set_ledger_timestamp(env: &Env, timestamp: u64) {
    env.ledger().set(LedgerInfo {
        timestamp,
        protocol_version: 20,
        sequence_number: 0,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 100,
        min_persistent_entry_ttl: 100,
        max_entry_ttl: 1000000,
    });
}

fn register_and_init_contract(env: &Env) -> Address {
    let contract_id = env.register_contract(None, StellarGuildsContract);
    let client = StellarGuildsContractClient::new(env, &contract_id);
    client.initialize(&Address::generate(&env));
    contract_id
}

fn create_mock_token(env: &Env, admin: &Address) -> Address {
    let token_contract_id = env.register_stellar_asset_contract_v2(admin.clone());
    token_contract_id.address()
}

fn mint_tokens(env: &Env, token: &Address, to: &Address, amount: i128) {
    let client = token::StellarAssetClient::new(env, token);
    client.mint(to, &amount);
}

fn get_token_balance(env: &Env, token: &Address, addr: &Address) -> i128 {
    let client = token::TokenClient::new(env, token);
    client.balance(addr)
}

fn setup_guild(client: &StellarGuildsContractClient<'_>, env: &Env, owner: &Address) -> u64 {
    let name = String::from_str(env, "Dispute Guild");
    let description = String::from_str(env, "Guild for disputes");
    client.create_guild(&name, &description, owner, &None::<InitializerProof>)
}

fn setup_guild_with_members(
    client: &StellarGuildsContractClient<'_>,
    env: &Env,
) -> (u64, Address, Address, Address, Address) {
    let owner = Address::generate(&env);
    let admin = Address::generate(&env);
    let member = Address::generate(&env);
    let contributor = Address::generate(&env);

    let guild_id = setup_guild(client, env, &owner);

    client.add_member(&guild_id, &admin, &Role::Admin, &owner);
    client.add_member(&guild_id, &member, &Role::Member, &owner);
    client.add_member(&guild_id, &contributor, &Role::Contributor, &owner);

    (guild_id, owner, admin, member, contributor)
}

fn create_funded_bounty(
    client: &StellarGuildsContractClient<'_>,
    env: &Env,
    guild_id: u64,
    owner: &Address,
    funder: &Address,
    token: &Address,
) -> u64 {
    let title = String::from_str(env, "Dispute bounty");
    let description = String::from_str(env, "Dispute bounty description");
    let reward = 100i128;
    let expiry = env.ledger().timestamp() + 1000;

    let bounty_id = client.create_bounty(
        &guild_id,
        owner,
        &title,
        &description,
        &reward,
        token,
        &expiry,
    );

    mint_tokens(env, token, funder, 1000);
    client.fund_bounty(&bounty_id, funder, &reward);

    bounty_id
}

#[test]
fn test_create_dispute_bounty_success() {
    let env = setup_env();
    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let (guild_id, owner, _admin, _member, contributor) = setup_guild_with_members(&client, &env);

    let token = create_mock_token(&env, &owner);
    let bounty_id = create_funded_bounty(&client, &env, guild_id, &owner, &owner, &token);

    let reason = String::from_str(&env, "Work rejected unfairly");
    let evidence = String::from_str(&env, "ipfs://evidence-1");

    let dispute_id = client.create_dispute(&bounty_id, &contributor, &owner, &reason, &evidence);

    assert_eq!(dispute_id, 1);
}

#[test]
fn test_create_dispute_milestone_success() {
    let env = setup_env();
    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let (guild_id, owner, _admin, _member, contributor) = setup_guild_with_members(&client, &env);

    let mut milestones: Vec<crate::milestone::types::MilestoneInput> = Vec::new(&env);
    milestones.push_back(crate::milestone::types::MilestoneInput {
        title: String::from_str(&env, "Milestone 1"),
        description: String::from_str(&env, "First milestone"),
        payment_amount: 100,
        deadline: 2000,
    });

    client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &100i128,
        &1u64,
        &None,
        &true,
    );

    let milestone_id = 1u64;
    let reason = String::from_str(&env, "Milestone dispute");
    let evidence = String::from_str(&env, "https://example.com/evidence");

    let dispute_id = client.create_dispute(&milestone_id, &contributor, &owner, &reason, &evidence);

    assert_eq!(dispute_id, 1);
}

#[test]
#[should_panic(expected = "only parties can submit evidence")]
fn test_submit_evidence_non_party_fails() {
    let env = setup_env();
    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let (guild_id, owner, _admin, _member, contributor) = setup_guild_with_members(&client, &env);

    let token = create_mock_token(&env, &owner);
    let bounty_id = create_funded_bounty(&client, &env, guild_id, &owner, &owner, &token);

    let reason = String::from_str(&env, "Dispute reason");
    let evidence = String::from_str(&env, "ipfs://evidence");

    let dispute_id = client.create_dispute(&bounty_id, &contributor, &owner, &reason, &evidence);

    let non_party = Address::generate(&env);
    let new_evidence = String::from_str(&env, "ipfs://other");

    client.submit_evidence(&dispute_id, &non_party, &new_evidence);
}

#[test]
fn test_cast_vote_weighted_and_quorum() {
    let env = setup_env();
    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let (guild_id, owner, admin, member, contributor) = setup_guild_with_members(&client, &env);
    let member2 = Address::generate(&env);
    client.add_member(&guild_id, &member2, &Role::Member, &owner);

    let token = create_mock_token(&env, &owner);
    let bounty_id = create_funded_bounty(&client, &env, guild_id, &owner, &owner, &token);

    let reason = String::from_str(&env, "Dispute reason");
    let evidence = String::from_str(&env, "ipfs://evidence");

    let dispute_id = client.create_dispute(&bounty_id, &contributor, &owner, &reason, &evidence);

    let weight_admin = client.calculate_dispute_vote_weight(&guild_id, &admin);
    let weight_member = client.calculate_dispute_vote_weight(&guild_id, &member);

    assert_eq!(weight_admin, 5);
    assert_eq!(weight_member, 2);

    client.cast_dispute_vote(&dispute_id, &admin, &VoteDecision::FavorPlaintiff);
    client.cast_dispute_vote(&dispute_id, &member, &VoteDecision::FavorPlaintiff);

    let tally = client.tally_dispute_votes(&dispute_id);
    assert_eq!(tally.vote_count, 2);
    assert_eq!(tally.votes_for_plaintiff, 7);
    assert_eq!(tally.quorum_reached, true);
}

#[test]
#[should_panic(expected = "voter already voted")]
fn test_double_vote_fails() {
    let env = setup_env();
    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let (guild_id, owner, admin, _member, contributor) = setup_guild_with_members(&client, &env);

    let token = create_mock_token(&env, &owner);
    let bounty_id = create_funded_bounty(&client, &env, guild_id, &owner, &owner, &token);

    let reason = String::from_str(&env, "Dispute reason");
    let evidence = String::from_str(&env, "ipfs://evidence");

    let dispute_id = client.create_dispute(&bounty_id, &contributor, &owner, &reason, &evidence);

    client.cast_dispute_vote(&dispute_id, &admin, &VoteDecision::FavorPlaintiff);
    client.cast_dispute_vote(&dispute_id, &admin, &VoteDecision::FavorPlaintiff);
}

#[test]
#[should_panic(expected = "voter must be guild member")]
fn test_non_member_vote_fails() {
    let env = setup_env();
    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let (guild_id, owner, _admin, _member, contributor) = setup_guild_with_members(&client, &env);

    let token = create_mock_token(&env, &owner);
    let bounty_id = create_funded_bounty(&client, &env, guild_id, &owner, &owner, &token);

    let reason = String::from_str(&env, "Dispute reason");
    let evidence = String::from_str(&env, "ipfs://evidence");

    let dispute_id = client.create_dispute(&bounty_id, &contributor, &owner, &reason, &evidence);

    let outsider = Address::generate(&env);
    client.cast_dispute_vote(&dispute_id, &outsider, &VoteDecision::FavorPlaintiff);
}

#[test]
#[should_panic(expected = "parties cannot vote")]
fn test_self_voting_fails() {
    let env = setup_env();
    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let (guild_id, owner, _admin, _member, contributor) = setup_guild_with_members(&client, &env);

    let token = create_mock_token(&env, &owner);
    let bounty_id = create_funded_bounty(&client, &env, guild_id, &owner, &owner, &token);

    let reason = String::from_str(&env, "Dispute reason");
    let evidence = String::from_str(&env, "ipfs://evidence");

    let dispute_id = client.create_dispute(&bounty_id, &contributor, &owner, &reason, &evidence);

    client.cast_dispute_vote(&dispute_id, &owner, &VoteDecision::FavorDefendant);
}

#[test]
fn test_resolve_dispute_tie_splits_funds() {
    let env = setup_env();
    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let (guild_id, owner, _admin, member, contributor) = setup_guild_with_members(&client, &env);
    let member2 = Address::generate(&env);
    client.add_member(&guild_id, &member2, &Role::Member, &owner);

    let token = create_mock_token(&env, &owner);
    let bounty_id = create_funded_bounty(&client, &env, guild_id, &owner, &owner, &token);

    let reason = String::from_str(&env, "Dispute reason");
    let evidence = String::from_str(&env, "ipfs://evidence");

    let dispute_id = client.create_dispute(&bounty_id, &contributor, &owner, &reason, &evidence);

    client.cast_dispute_vote(&dispute_id, &member, &VoteDecision::FavorPlaintiff);
    client.cast_dispute_vote(&dispute_id, &member2, &VoteDecision::FavorDefendant);

    let before_plaintiff = get_token_balance(&env, &token, &contributor);
    let before_defendant = get_token_balance(&env, &token, &owner);

    // Move time forward past deadline
    set_ledger_timestamp(&env, 1000 + 7 * 24 * 60 * 60 + 1);

    let resolution = client.resolve_dispute(&dispute_id);
    assert_eq!(resolution.quorum_reached, true);

    let after_plaintiff = get_token_balance(&env, &token, &contributor);
    let after_defendant = get_token_balance(&env, &token, &owner);

    assert_eq!(after_plaintiff - before_plaintiff, 50);
    assert_eq!(after_defendant - before_defendant, 50);
}

#[test]
fn test_insufficient_quorum_expires_and_refunds_creator() {
    let env = setup_env();
    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let (guild_id, owner, admin, _member, contributor) = setup_guild_with_members(&client, &env);

    let token = create_mock_token(&env, &owner);
    let bounty_id = create_funded_bounty(&client, &env, guild_id, &owner, &owner, &token);

    let reason = String::from_str(&env, "Dispute reason");
    let evidence = String::from_str(&env, "ipfs://evidence");

    let dispute_id = client.create_dispute(&bounty_id, &contributor, &owner, &reason, &evidence);

    // Only one vote -> below 30% quorum (1 of 4 members)
    client.cast_dispute_vote(&dispute_id, &admin, &VoteDecision::FavorPlaintiff);

    let before_owner = get_token_balance(&env, &token, &owner);

    set_ledger_timestamp(&env, 1000 + 7 * 24 * 60 * 60 + 1);
    let resolution = client.resolve_dispute(&dispute_id);

    assert_eq!(resolution.quorum_reached, false);

    let after_owner = get_token_balance(&env, &token, &owner);
    assert_eq!(after_owner - before_owner, 100);
}

#[test]
#[should_panic(expected = "voting period ended")]
fn test_voting_deadline_enforced() {
    let env = setup_env();
    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let (guild_id, owner, admin, _member, contributor) = setup_guild_with_members(&client, &env);

    let token = create_mock_token(&env, &owner);
    let bounty_id = create_funded_bounty(&client, &env, guild_id, &owner, &owner, &token);

    let reason = String::from_str(&env, "Dispute reason");
    let evidence = String::from_str(&env, "ipfs://evidence");

    let dispute_id = client.create_dispute(&bounty_id, &contributor, &owner, &reason, &evidence);

    set_ledger_timestamp(&env, 1000 + 7 * 24 * 60 * 60 + 2);

    client.cast_dispute_vote(&dispute_id, &admin, &VoteDecision::FavorPlaintiff);
}

#[test]
#[should_panic(expected = "dispute already active for reference")]
fn test_concurrent_disputes_blocked() {
    let env = setup_env();
    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let (guild_id, owner, _admin, _member, contributor) = setup_guild_with_members(&client, &env);

    let token = create_mock_token(&env, &owner);
    let bounty_id = create_funded_bounty(&client, &env, guild_id, &owner, &owner, &token);

    let reason = String::from_str(&env, "Dispute reason");
    let evidence = String::from_str(&env, "ipfs://evidence");

    let _dispute_id = client.create_dispute(&bounty_id, &contributor, &owner, &reason, &evidence);

    client.create_dispute(&bounty_id, &contributor, &owner, &reason, &evidence);
}

#[test]
fn test_dispute_status_updates_after_resolution() {
    let env = setup_env();
    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let (guild_id, owner, admin, member, contributor) = setup_guild_with_members(&client, &env);

    let token = create_mock_token(&env, &owner);
    let bounty_id = create_funded_bounty(&client, &env, guild_id, &owner, &owner, &token);

    let reason = String::from_str(&env, "Dispute reason");
    let evidence = String::from_str(&env, "ipfs://evidence");

    let dispute_id = client.create_dispute(&bounty_id, &contributor, &owner, &reason, &evidence);

    client.cast_dispute_vote(&dispute_id, &admin, &VoteDecision::FavorPlaintiff);
    client.cast_dispute_vote(&dispute_id, &member, &VoteDecision::FavorPlaintiff);

    set_ledger_timestamp(&env, 1000 + 7 * 24 * 60 * 60 + 1);
    let resolution = client.resolve_dispute(&dispute_id);

    assert_eq!(resolution.quorum_reached, true);
    assert_eq!(resolution.vote_count, 2);

    let dispute = client.tally_dispute_votes(&dispute_id);
    assert_eq!(dispute.vote_count, 2);
}
