#[cfg(test)]
mod tests {
    use crate::governance::{ProposalType, VoteDecision};
    use crate::multisig::types::{OperationStatus, OperationType, TIMEOUT_24H, TIMEOUT_48H};
    use crate::InitializerProof;
    use crate::{StellarGuildsContract, StellarGuildsContractClient};
    use soroban_sdk::testutils::{Address as _, Ledger as _, LedgerInfo};
    use soroban_sdk::{Address, Env, String, Vec};

    fn setup_env() -> (Env, Address, Address, Address) {
        let env = Env::default();
        env.budget().reset_unlimited();
        let owner = Address::generate(&env);
        let signer1 = Address::generate(&env);
        let signer2 = Address::generate(&env);
        (env, owner, signer1, signer2)
    }

    fn init_client(env: &Env) -> StellarGuildsContractClient<'_> {
        let contract_id = env.register_contract(None, StellarGuildsContract);
        let client = StellarGuildsContractClient::new(env, &contract_id);
        client.initialize(&Address::generate(&env));
        client
    }

    fn set_timestamp(env: &Env, timestamp: u64) {
        env.ledger().set(LedgerInfo {
            timestamp,
            protocol_version: 20,
            sequence_number: 1,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 100,
            min_persistent_entry_ttl: 100,
            max_entry_ttl: 1_000_000,
        });
    }

    fn register_ms_account(
        env: &Env,
        client: &StellarGuildsContractClient<'_>,
        owner: &Address,
        signer1: &Address,
        signer2: &Address,
    ) -> u64 {
        let mut signers = Vec::new(env);
        signers.push_back(signer1.clone());
        signers.push_back(signer2.clone());
        client.ms_register_account(owner, &signers, &2u32, &None, &TIMEOUT_24H)
    }

    #[test]
    fn test_register_and_propose() {
        let (env, owner, signer1, signer2) = setup_env();
        env.mock_all_auths();
        let client = init_client(&env);

        let account_id = register_ms_account(&env, &client, &owner, &signer1, &signer2);

        let desc = String::from_str(&env, "Test Tx");
        let op_id = client.ms_propose_operation(
            &account_id,
            &OperationType::TreasuryWithdrawal,
            &desc,
            &owner,
        );

        let op = client.ms_get_operation(&op_id);
        assert_eq!(op.status, OperationStatus::Pending);
        assert_eq!(op.signatures.len(), 1); // Proposer auto-signs
    }

    #[test]
    fn test_signing_and_execution() {
        let (env, owner, signer1, signer2) = setup_env();
        env.mock_all_auths();
        let client = init_client(&env);

        let account_id = register_ms_account(&env, &client, &owner, &signer1, &signer2);

        let desc = String::from_str(&env, "Test Tx");
        let op_id = client.ms_propose_operation(
            &account_id,
            &OperationType::TreasuryWithdrawal,
            &desc,
            &owner,
        );

        // Signer 1 signs (threshold is 2, so this meets it)
        client.ms_sign_operation(&op_id, &signer1);

        // Execute
        let executed = client.ms_execute_operation(&op_id, &signer2);
        assert!(executed);

        let op = client.ms_get_operation(&op_id);
        assert_eq!(op.status, OperationStatus::Executed);

        // Verify Replay Protection (nonce incremented)
        let account = client.ms_get_account(&account_id);
        assert_eq!(account.nonce, 1);
    }

    #[test]
    fn test_pending_ops_and_sweep_expired() {
        let (env, owner, signer1, signer2) = setup_env();
        env.mock_all_auths();
        let client = init_client(&env);
        let account_id = register_ms_account(&env, &client, &owner, &signer1, &signer2);

        let now = env.ledger().timestamp();
        let desc = String::from_str(&env, "Sweep me");
        let op_id = client.ms_propose_operation(
            &account_id,
            &OperationType::EmergencyAction,
            &desc,
            &owner,
        );

        // Default policy timeout is 48h; move past it before sweeping.
        set_timestamp(&env, now + TIMEOUT_48H + 1);
        let swept = client.ms_sweep_expired(&account_id);
        assert_eq!(swept, 1);

        let op = client.ms_get_operation(&op_id);
        assert_eq!(op.status, OperationStatus::Expired);
        let pending = client.ms_get_pending_ops(&account_id);
        assert_eq!(pending.len(), 0);
    }

    #[test]
    fn test_policy_require_owner_signature() {
        let (env, owner, signer1, signer2) = setup_env();
        env.mock_all_auths();
        let client = init_client(&env);
        let account_id = register_ms_account(&env, &client, &owner, &signer1, &signer2);

        // Require owner signature for governance updates.
        client.ms_set_policy(
            &account_id,
            &OperationType::GovernanceUpdate,
            &2u32,
            &false,
            &TIMEOUT_24H,
            &true,
            &owner,
        );

        let desc = String::from_str(&env, "gov gate");
        let op_id = client.ms_propose_operation(
            &account_id,
            &OperationType::GovernanceUpdate,
            &desc,
            &signer1,
        );
        client.ms_sign_operation(&op_id, &signer2);

        client.ms_sign_operation(&op_id, &owner);
        assert!(client.ms_execute_operation(&op_id, &signer2));
    }

    #[test]
    fn test_list_accounts_by_owner() {
        let (env, owner, signer1, signer2) = setup_env();
        env.mock_all_auths();
        let client = init_client(&env);
        let _a1 = register_ms_account(&env, &client, &owner, &signer1, &signer2);
        let _a2 = register_ms_account(&env, &client, &owner, &signer1, &signer2);
        let accounts = client.ms_list_accounts(&owner);
        assert_eq!(accounts.len(), 2);
    }

    #[test]
    fn test_rotate_signer_key() {
        let (env, owner, signer1, signer2) = setup_env();
        env.mock_all_auths();
        let client = init_client(&env);
        let account_id = register_ms_account(&env, &client, &owner, &signer1, &signer2);
        let replacement = Address::generate(&env);

        assert!(client.ms_rotate_signer(&account_id, &signer1, &replacement, &owner));
        let account = client.ms_get_account(&account_id);
        assert!(account.signers.contains(&replacement));
        assert!(!account.signers.contains(&signer1));
    }

    #[test]
    fn test_treasury_withdrawal_multisig_gate_integration() {
        let (env, owner, signer1, signer2) = setup_env();
        env.mock_all_auths();
        let client = init_client(&env);

        // Treasury setup
        let guild_name = String::from_str(&env, "Treasury Guild");
        let guild_desc = String::from_str(&env, "Guild for treasury integration");
        let guild_id =
            client.create_guild(&guild_name, &guild_desc, &owner, &None::<InitializerProof>);

        let mut treasury_signers = Vec::new(&env);
        treasury_signers.push_back(owner.clone());
        treasury_signers.push_back(signer1.clone());
        treasury_signers.push_back(signer2.clone());
        let treasury_id = client.initialize_treasury(&guild_id, &treasury_signers, &2u32);
        client.deposit_treasury(&treasury_id, &owner, &1_000i128, &None);

        // Multisig gate setup
        let account_id = register_ms_account(&env, &client, &owner, &signer1, &signer2);
        let op_desc = String::from_str(&env, "Approve treasury withdrawal");
        let op_id = client.ms_propose_operation(
            &account_id,
            &OperationType::TreasuryWithdrawal,
            &op_desc,
            &owner,
        );
        client.ms_sign_operation(&op_id, &signer1);
        assert!(client.ms_execute_operation(&op_id, &signer2));

        let reason = String::from_str(&env, "multisig-approved withdrawal");
        let tx_id = client.ms_propose_treasury_withdrawal(
            &op_id,
            &treasury_id,
            &owner,
            &signer1,
            &100i128,
            &None,
            &reason,
        );
        // Tx id 1 is the deposit; withdrawal proposal is the next tx.
        assert_eq!(tx_id, 2);
    }

    #[test]
    fn test_governance_multisig_gate_integration() {
        let (env, owner, signer1, signer2) = setup_env();
        env.mock_all_auths();
        let client = init_client(&env);

        let guild_name = String::from_str(&env, "Gov Guild");
        let guild_desc = String::from_str(&env, "Guild for governance integration");
        let guild_id =
            client.create_guild(&guild_name, &guild_desc, &owner, &None::<InitializerProof>);

        let title = String::from_str(&env, "General decision");
        let description = String::from_str(&env, "Should we proceed?");
        let proposal_id = client.create_proposal(
            &guild_id,
            &owner,
            &ProposalType::GeneralDecision,
            &title,
            &description,
        );
        client.vote(&proposal_id, &owner, &VoteDecision::For);
        // End voting period.
        set_timestamp(&env, env.ledger().timestamp() + 8 * 24 * 60 * 60);

        let account_id = register_ms_account(&env, &client, &owner, &signer1, &signer2);
        let op_desc = String::from_str(&env, "Approve governance execution");
        let op_id = client.ms_propose_operation(
            &account_id,
            &OperationType::GovernanceUpdate,
            &op_desc,
            &owner,
        );
        client.ms_sign_operation(&op_id, &signer1);
        assert!(client.ms_execute_operation(&op_id, &signer2));
        assert!(client.ms_execute_governance_proposal(&op_id, &proposal_id, &owner));
    }

    #[test]
    fn test_account_management_controls() {
        let (env, owner, signer1, signer2) = setup_env();
        env.mock_all_auths();
        let client = init_client(&env);
        let account_id = register_ms_account(&env, &client, &owner, &signer1, &signer2);
        let signer3 = Address::generate(&env);
        let replacement = Address::generate(&env);

        assert!(client.ms_add_signer(&account_id, &signer3, &owner));
        let account = client.ms_get_account(&account_id);
        assert!(account.signers.contains(&signer3));

        assert!(client.ms_update_threshold(&account_id, &3, &owner));
        assert_eq!(client.ms_get_account(&account_id).threshold, 3);

        assert!(client.ms_freeze_account(&account_id, &owner));
        assert_eq!(
            client.ms_get_account(&account_id).status,
            crate::multisig::types::AccountStatus::Frozen
        );
        assert!(client.ms_unfreeze_account(&account_id, &owner));
        assert_eq!(
            client.ms_get_account(&account_id).status,
            crate::multisig::types::AccountStatus::Active
        );

        assert!(client.ms_rotate_signer(&account_id, &signer3, &replacement, &owner));
        let account = client.ms_get_account(&account_id);
        assert!(account.signers.contains(&replacement));
        assert!(!account.signers.contains(&signer3));

        assert!(client.ms_remove_signer(&account_id, &replacement, &owner, &2));
        let account = client.ms_get_account(&account_id);
        assert_eq!(account.threshold, 2);
        assert!(!account.signers.contains(&replacement));
    }

    #[test]
    fn test_cancel_expire_and_pending_operation_queries() {
        let (env, owner, signer1, signer2) = setup_env();
        env.mock_all_auths();
        let client = init_client(&env);
        let account_id = register_ms_account(&env, &client, &owner, &signer1, &signer2);

        let op_a = client.ms_propose_operation(
            &account_id,
            &OperationType::EmergencyAction,
            &String::from_str(&env, "cancel me"),
            &owner,
        );
        assert_eq!(client.ms_get_pending_ops(&account_id).len(), 1);
        assert!(client.ms_cancel_operation(&op_a, &owner));
        assert_eq!(
            client.ms_get_operation(&op_a).status,
            OperationStatus::Cancelled
        );
        assert_eq!(client.ms_get_pending_ops(&account_id).len(), 0);

        let op_b = client.ms_propose_operation(
            &account_id,
            &OperationType::GovernanceUpdate,
            &String::from_str(&env, "expire me"),
            &owner,
        );
        set_timestamp(&env, env.ledger().timestamp() + TIMEOUT_48H + 5);
        assert!(client.ms_check_and_expire(&op_b));
        assert_eq!(
            client.ms_get_operation(&op_b).status,
            OperationStatus::Expired
        );

        let op_c = client.ms_propose_operation(
            &account_id,
            &OperationType::TreasuryWithdrawal,
            &String::from_str(&env, "sweep me"),
            &owner,
        );
        let op_d = client.ms_propose_operation(
            &account_id,
            &OperationType::TreasuryWithdrawal,
            &String::from_str(&env, "expire now"),
            &owner,
        );
        assert!(client.ms_emergency_extend_timeout(&op_c, &TIMEOUT_24H, &owner));
        assert!(client.ms_emergency_expire(&op_d, &owner));
        assert_eq!(
            client.ms_get_operation(&op_d).status,
            OperationStatus::Expired
        );

        set_timestamp(&env, env.ledger().timestamp() + TIMEOUT_24H + 5);
        assert_eq!(client.ms_sweep_expired(&account_id), 1);
        assert_eq!(
            client.ms_get_operation(&op_c).status,
            OperationStatus::Expired
        );
    }

    #[test]
    fn test_policy_round_trip_and_reset() {
        let (env, owner, signer1, signer2) = setup_env();
        env.mock_all_auths();
        let client = init_client(&env);
        let account_id = register_ms_account(&env, &client, &owner, &signer1, &signer2);

        assert!(client.ms_set_policy(
            &account_id,
            &OperationType::EmergencyAction,
            &3,
            &true,
            &TIMEOUT_24H,
            &true,
            &owner,
        ));
        let policy = client.ms_get_policy(&account_id, &OperationType::EmergencyAction);
        assert_eq!(policy.min_signatures, 3);
        assert!(policy.require_all_signers);
        assert!(policy.require_owner_signature);

        assert!(client.ms_reset_policy(&account_id, &OperationType::EmergencyAction, &owner));
        let reset = client.ms_get_policy(&account_id, &OperationType::EmergencyAction);
        assert_eq!(reset.min_signatures, 1);
        assert!(!reset.require_all_signers);
        assert!(!reset.require_owner_signature);
    }

    #[test]
    #[should_panic(expected = "ms_register_account error")]
    fn test_register_invalid_threshold_panics() {
        let (env, owner, signer1, signer2) = setup_env();
        env.mock_all_auths();
        let client = init_client(&env);
        let mut signers = Vec::new(&env);
        signers.push_back(signer1);
        signers.push_back(signer2);

        client.ms_register_account(&owner, &signers, &1, &None, &TIMEOUT_24H);
    }

    #[test]
    #[should_panic(expected = "ms_propose_operation error")]
    fn test_frozen_account_cannot_propose_operation() {
        let (env, owner, signer1, signer2) = setup_env();
        env.mock_all_auths();
        let client = init_client(&env);
        let account_id = register_ms_account(&env, &client, &owner, &signer1, &signer2);

        assert!(client.ms_freeze_account(&account_id, &owner));
        client.ms_propose_operation(
            &account_id,
            &OperationType::EmergencyAction,
            &String::from_str(&env, "blocked"),
            &owner,
        );
    }
}
