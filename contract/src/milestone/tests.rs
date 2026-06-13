//! Milestone Tracking Contract Tests
//!
//! Comprehensive test coverage for project creation, milestone management,
//! sequential/parallel flows, submissions, approvals, and progress tracking.
//!
//! NOTE: Payment release tests are excluded as they require treasury integration.

use crate::guild::types::Role;
use crate::milestone::types::{MilestoneInput, MilestoneStatus};
use crate::InitializerProof;
use crate::StellarGuildsContract;
use crate::StellarGuildsContractClient;
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{Address, Env, String, Vec};

// ============ Test Helpers ============

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

fn setup_guild(client: &StellarGuildsContractClient<'_>, env: &Env, owner: &Address) -> u64 {
    let name = String::from_str(env, "Dev Guild");
    let description = String::from_str(env, "Developer Guild");
    client.create_guild(&name, &description, owner, &None::<InitializerProof>)
}

fn add_admin(
    client: &StellarGuildsContractClient<'_>,
    _env: &Env,
    guild_id: u64,
    owner: &Address,
    admin: &Address,
) {
    client.add_member(&guild_id, admin, &Role::Admin, owner);
}

fn create_treasury_with_funds(
    client: &StellarGuildsContractClient<'_>,
    env: &Env,
    guild_id: u64,
    owner: &Address,
    amount: i128,
) -> u64 {
    let signer1 = Address::generate(&env);
    let signer2 = Address::generate(&env);

    env.mock_all_auths();

    let mut signers = Vec::new(env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    let treasury_id = client.initialize_treasury(&guild_id, &signers, &2u32);
    client.deposit_treasury(&treasury_id, owner, &amount, &None);
    treasury_id
}

// ============ Project Creation Tests ============

#[test]
fn test_create_project_success() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "Phase 1"),
        description: String::from_str(&env, "Initial development"),
        payment_amount: 100_000,
        deadline: now + 86400,
    });

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "Phase 2"),
        description: String::from_str(&env, "Testing phase"),
        payment_amount: 50_000,
        deadline: now + 2 * 86400,
    });

    let project_id = client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &150_000i128,
        &1u64, // treasury_id (mock)
        &None,
        &true, // sequential
    );

    assert_eq!(project_id, 1);

    // Check progress
    let (completed, total, pct) = client.get_project_progress(&project_id);
    assert_eq!(completed, 0);
    assert_eq!(total, 2);
    assert_eq!(pct, 0);
}

#[test]
#[should_panic(expected = "at least one milestone required")]
fn test_create_project_no_milestones_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let milestones: Vec<MilestoneInput> = Vec::new(&env);

    client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &100_000i128,
        &1u64,
        &None,
        &false,
    );
}

#[test]
#[should_panic(expected = "total_amount must be positive")]
fn test_create_project_zero_amount_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "Task"),
        description: String::from_str(&env, "Work"),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &0i128,
        &1u64,
        &None,
        &false,
    );
}

#[test]
#[should_panic(expected = "allocated milestone budget exceeds project total")]
fn test_create_project_overallocated_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 60_000,
        deadline: now + 86400,
    });

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M2"),
        description: String::from_str(&env, ""),
        payment_amount: 60_000,
        deadline: now + 2 * 86400,
    });

    // Total milestones = 120k, but budget is only 100k
    client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &100_000i128,
        &1u64,
        &None,
        &false,
    );
}

#[test]
#[should_panic(expected = "milestone deadline must be in the future")]
fn test_create_project_past_deadline_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: 500, // Past deadline
    });

    client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &1000i128,
        &1u64,
        &None,
        &false,
    );
}

// ============ Milestone Lifecycle Tests ============

#[test]
fn test_start_milestone_success() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &1000i128,
        &1u64,
        &None,
        &false,
    );

    let milestone_id = 1u64; // First milestone

    let result = client.start_milestone(&milestone_id, &contributor);
    assert_eq!(result, true);

    let milestone = client.get_milestone(&milestone_id);
    assert_eq!(milestone.status, MilestoneStatus::InProgress);
}

#[test]
#[should_panic(expected = "only project contributor can start milestone")]
fn test_start_milestone_wrong_contributor_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);
    let other = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &1000i128,
        &1u64,
        &None,
        &false,
    );

    let milestone_id = 1u64;

    // Different user tries to start
    client.start_milestone(&milestone_id, &other);
}

#[test]
fn test_submit_milestone_success() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &1000i128,
        &1u64,
        &None,
        &false,
    );

    let milestone_id = 1u64;

    client.start_milestone(&milestone_id, &contributor);

    let proof_url = String::from_str(&env, "https://github.com/pr/123");
    let result = client.submit_milestone(&milestone_id, &proof_url);
    assert_eq!(result, true);

    let milestone = client.get_milestone(&milestone_id);
    assert_eq!(milestone.status, MilestoneStatus::Submitted);
    assert_eq!(milestone.proof_url, proof_url);
    assert_eq!(milestone.version, 1);
}

#[test]
#[should_panic(expected = "milestone not in progress or previously rejected")]
fn test_submit_milestone_before_starting_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &1000i128,
        &1u64,
        &None,
        &false,
    );

    let milestone_id = 1u64;

    // Try to submit without starting
    let proof_url = String::from_str(&env, "https://github.com/pr/123");
    client.submit_milestone(&milestone_id, &proof_url);
}

#[test]
fn test_approve_milestone_success() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let admin = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);
    add_admin(&client, &env, guild_id, &owner, &admin);

    let treasury_id = create_treasury_with_funds(&client, &env, guild_id, &owner, 5000i128);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &1000i128,
        &treasury_id,
        &None,
        &false,
    );

    let milestone_id = 1u64;

    client.start_milestone(&milestone_id, &contributor);

    let proof_url = String::from_str(&env, "https://github.com/pr/123");
    client.submit_milestone(&milestone_id, &proof_url);

    // Admin approves
    let result = client.approve_milestone(&milestone_id, &admin);
    assert_eq!(result, true);

    let milestone = client.get_milestone(&milestone_id);
    assert_eq!(milestone.status, MilestoneStatus::Approved);
}

#[test]
#[should_panic(expected = "approver must be guild admin")]
fn test_approve_milestone_non_admin_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);
    let non_admin = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &1000i128,
        &1u64,
        &None,
        &false,
    );

    let milestone_id = 1u64;

    client.start_milestone(&milestone_id, &contributor);

    let proof_url = String::from_str(&env, "https://github.com/pr/123");
    client.submit_milestone(&milestone_id, &proof_url);

    // Non-admin tries to approve
    client.approve_milestone(&milestone_id, &non_admin);
}

#[test]
#[should_panic(expected = "milestone not submitted")]
fn test_approve_milestone_not_submitted_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &1000i128,
        &1u64,
        &None,
        &false,
    );

    let milestone_id = 1u64;

    client.start_milestone(&milestone_id, &contributor);

    // Try to approve without submitting
    client.approve_milestone(&milestone_id, &owner);
}

#[test]
fn test_reject_milestone_success() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let admin = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);
    add_admin(&client, &env, guild_id, &owner, &admin);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &1000i128,
        &1u64,
        &None,
        &false,
    );

    let milestone_id = 1u64;

    client.start_milestone(&milestone_id, &contributor);

    let proof_url = String::from_str(&env, "https://github.com/pr/123");
    client.submit_milestone(&milestone_id, &proof_url);

    // Admin rejects
    let reason = String::from_str(&env, "Incomplete work");
    let result = client.reject_milestone(&milestone_id, &admin, &reason);
    assert_eq!(result, true);

    let milestone = client.get_milestone(&milestone_id);
    assert_eq!(milestone.status, MilestoneStatus::Rejected);
}

#[test]
fn test_resubmit_after_rejection() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &1000i128,
        &1u64,
        &None,
        &false,
    );

    let milestone_id = 1u64;

    client.start_milestone(&milestone_id, &contributor);

    let proof_url = String::from_str(&env, "https://github.com/pr/123");
    client.submit_milestone(&milestone_id, &proof_url);

    // Owner rejects
    let reason = String::from_str(&env, "Needs fixes");
    client.reject_milestone(&milestone_id, &owner, &reason);

    let milestone = client.get_milestone(&milestone_id);
    assert_eq!(milestone.status, MilestoneStatus::Rejected);
    assert_eq!(milestone.version, 1);

    // Resubmit
    let new_proof = String::from_str(&env, "https://github.com/pr/456");
    client.submit_milestone(&milestone_id, &new_proof);

    let milestone = client.get_milestone(&milestone_id);
    assert_eq!(milestone.status, MilestoneStatus::Submitted);
    assert_eq!(milestone.version, 2);
    assert_eq!(milestone.proof_url, new_proof);
}

// ============ Sequential Milestone Tests ============

#[test]
#[should_panic(expected = "previous milestone not completed")]
fn test_sequential_prevents_out_of_order_start() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M2"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 2 * 86400,
    });

    client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &2000i128,
        &1u64,
        &None,
        &true, // Sequential
    );

    let milestone_2_id = 2u64;

    // Try to start second milestone without completing first
    client.start_milestone(&milestone_2_id, &contributor);
}

#[test]
fn test_sequential_allows_second_after_first_approved() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);
    let treasury_id = create_treasury_with_funds(&client, &env, guild_id, &owner, 5000i128);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M2"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 2 * 86400,
    });

    let project_id = client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &2000i128,
        &treasury_id,
        &None,
        &true, // Sequential
    );

    let milestone_1_id = 1u64;
    let milestone_2_id = 2u64;

    // Complete first milestone
    client.start_milestone(&milestone_1_id, &contributor);
    let proof = String::from_str(&env, "https://proof1");
    client.submit_milestone(&milestone_1_id, &proof);
    client.approve_milestone(&milestone_1_id, &owner);

    let (completed, total, pct) = client.get_project_progress(&project_id);
    assert_eq!(completed, 1);
    assert_eq!(total, 2);
    assert_eq!(pct, 50);

    // Now second milestone can be started
    let result = client.start_milestone(&milestone_2_id, &contributor);
    assert_eq!(result, true);

    let milestone = client.get_milestone(&milestone_2_id);
    assert_eq!(milestone.status, MilestoneStatus::InProgress);
}

// ============ Parallel Milestone Tests ============

#[test]
fn test_parallel_allows_any_order() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M2"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 2 * 86400,
    });

    client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &2000i128,
        &1u64,
        &None,
        &false, // Parallel
    );

    let milestone_2_id = 2u64;

    // Can start second milestone without completing first
    let result = client.start_milestone(&milestone_2_id, &contributor);
    assert_eq!(result, true);
}

// ============ Progress Tracking Tests ============

#[test]
fn test_progress_calculation() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);
    let treasury_id = create_treasury_with_funds(&client, &env, guild_id, &owner, 10000i128);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M2"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 2 * 86400,
    });

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M3"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 3 * 86400,
    });

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M4"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 4 * 86400,
    });

    let project_id = client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &4000i128,
        &treasury_id,
        &None,
        &false,
    );

    // Initially 0%
    let (completed, total, pct) = client.get_project_progress(&project_id);
    assert_eq!(completed, 0);
    assert_eq!(total, 4);
    assert_eq!(pct, 0);

    // Complete 2 milestones
    for i in 1..=2 {
        let milestone_id = i as u64;
        client.start_milestone(&milestone_id, &contributor);
        let proof = String::from_str(&env, "https://proof");
        client.submit_milestone(&milestone_id, &proof);
        client.approve_milestone(&milestone_id, &owner);
    }

    // Should be 50%
    let (completed, total, pct) = client.get_project_progress(&project_id);
    assert_eq!(completed, 2);
    assert_eq!(total, 4);
    assert_eq!(pct, 50);
}

// ============ Add Milestone Tests ============

#[test]
fn test_add_milestone_to_existing_project() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    let project_id = client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &5000i128, // Extra budget
        &1u64,
        &None,
        &false,
    );

    let (_, total, _) = client.get_project_progress(&project_id);
    assert_eq!(total, 1);

    // Add another milestone
    let new_milestone_id = client.add_milestone(
        &project_id,
        &String::from_str(&env, "M2"),
        &String::from_str(&env, "Additional work"),
        &2000i128,
        &(now + 2 * 86400),
        &owner,
    );

    assert_eq!(new_milestone_id, 2);

    let (_, total, _) = client.get_project_progress(&project_id);
    assert_eq!(total, 2);
}

#[test]
#[should_panic(expected = "caller must be guild admin")]
fn test_add_milestone_non_admin_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);
    let non_admin = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    let project_id = client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &5000i128,
        &1u64,
        &None,
        &false,
    );

    // Non-admin tries to add milestone
    client.add_milestone(
        &project_id,
        &String::from_str(&env, "M2"),
        &String::from_str(&env, "Work"),
        &1000i128,
        &(now + 2 * 86400),
        &non_admin,
    );
}

// ============ Deadline Extension Tests ============

#[test]
fn test_extend_milestone_deadline() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &1000i128,
        &1u64,
        &None,
        &false,
    );

    let milestone_id = 1u64;
    let new_deadline = now + 2 * 86400;

    let result = client.extend_milestone_deadline(&milestone_id, &new_deadline, &owner);
    assert_eq!(result, true);

    let milestone = client.get_milestone(&milestone_id);
    assert_eq!(milestone.deadline, new_deadline);
}

// ============ Project Cancellation Tests ============

#[test]
fn test_cancel_project() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    let project_id = client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &1000i128,
        &1u64,
        &None,
        &false,
    );

    let result = client.cancel_project(&project_id, &owner);
    assert_eq!(result, true);
}

#[test]
#[should_panic(expected = "caller must be guild admin")]
fn test_cancel_project_non_admin_fails() {
    let env = setup_env();
    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);
    let non_admin = Address::generate(&env);

    set_ledger_timestamp(&env, 1000);
    env.mock_all_auths();

    let contract_id = register_and_init_contract(&env);
    let client = StellarGuildsContractClient::new(&env, &contract_id);

    let guild_id = setup_guild(&client, &env, &owner);

    let now = env.ledger().timestamp();
    let mut milestones: Vec<MilestoneInput> = Vec::new(&env);

    milestones.push_back(MilestoneInput {
        title: String::from_str(&env, "M1"),
        description: String::from_str(&env, ""),
        payment_amount: 1000,
        deadline: now + 86400,
    });

    let project_id = client.create_project(
        &guild_id,
        &contributor,
        &milestones,
        &1000i128,
        &1u64,
        &None,
        &false,
    );

    // Non-admin tries to cancel
    client.cancel_project(&project_id, &non_admin);
}
