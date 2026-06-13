#[cfg(test)]
mod tests {
    use crate::governance::types::{
        ExecutionPayload, GovernanceConfig, Proposal, ProposalStatus, ProposalType, Vote,
        VoteDecision,
    };
    use crate::governance::{proposals, storage};
    use crate::guild::types::Role;
    use crate::InitializerProof;
    use crate::StellarGuildsContract;
    use crate::StellarGuildsContractClient;
    use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
    use soroban_sdk::{Address, Env, String};

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
        let name = String::from_str(env, "Gov Guild");
        let desc = String::from_str(env, "Governance test guild");
        client.create_guild(&name, &desc, owner, &None::<InitializerProof>)
    }

    fn setup_guild_with_members(
        env: &Env,
        client: &StellarGuildsContractClient<'_>,
        owner: &Address,
    ) -> (u64, Address, Address, Address) {
        let admin = Address::generate(&env);
        let member = Address::generate(&env);
        let contributor = Address::generate(&env);

        let guild_id = setup_guild(client, env, owner);

        client.add_member(&guild_id, &admin, &Role::Admin, owner);
        client.add_member(&guild_id, &member, &Role::Member, owner);
        client.add_member(&guild_id, &contributor, &Role::Contributor, owner);

        (guild_id, admin, member, contributor)
    }

    #[test]
    fn test_create_proposal_basic() {
        let env = setup_env();
        let owner = Address::generate(&env);

        set_ledger_timestamp(&env, 1000);
        env.mock_all_auths();

        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        let (guild_id, _admin, _member, _contributor) =
            setup_guild_with_members(&env, &client, &owner);

        let proposal_id = client.create_proposal(
            &guild_id,
            &owner,
            &ProposalType::GeneralDecision,
            &String::from_str(&env, "Test Proposal"),
            &String::from_str(&env, "Description"),
        );

        assert_eq!(proposal_id, 1);

        let proposal = client.get_proposal(&proposal_id);
        assert_eq!(proposal.guild_id, guild_id);
        assert_eq!(proposal.proposer, owner);
        assert_eq!(proposal.status, ProposalStatus::Active);
    }

    #[test]
    fn test_vote_weights_and_execution() {
        let env = setup_env();
        let owner = Address::generate(&env);

        set_ledger_timestamp(&env, 1000);
        env.mock_all_auths();

        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        let (guild_id, admin, member, contributor) =
            setup_guild_with_members(&env, &client, &owner);

        let proposal_id = client.create_proposal(
            &guild_id,
            &owner,
            &ProposalType::GeneralDecision,
            &String::from_str(&env, "Test Proposal"),
            &String::from_str(&env, "Description"),
        );

        client.vote(&proposal_id, &owner, &VoteDecision::For);
        client.vote(&proposal_id, &admin, &VoteDecision::For);
        client.vote(&proposal_id, &member, &VoteDecision::Against);
        client.vote(&proposal_id, &contributor, &VoteDecision::Abstain);

        let proposal = client.get_proposal(&proposal_id);
        let end = proposal.voting_end;
        set_ledger_timestamp(&env, end + 1);

        let status = client.finalize_proposal(&proposal_id);
        assert_eq!(status, ProposalStatus::Passed);

        let proposal_after_finalize = client.get_proposal(&proposal_id);
        assert_eq!(proposal_after_finalize.votes_for, 15);
        assert_eq!(proposal_after_finalize.votes_against, 2);
        assert_eq!(proposal_after_finalize.votes_abstain, 1);

        // Ensure proper execution utilizing the new auth executor paradigm
        let is_executed = client.execute_proposal(&proposal_id, &owner);
        assert!(is_executed);

        let final_proposal = client.get_proposal(&proposal_id);
        assert_eq!(final_proposal.status, ProposalStatus::Executed);
    }

    #[test]
    fn test_vote_delegation_and_execution() {
        let env = setup_env();
        let owner = Address::generate(&env);

        set_ledger_timestamp(&env, 1000);
        env.mock_all_auths();

        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        let (guild_id, admin, member, contributor) =
            setup_guild_with_members(&env, &client, &owner);

        let proposal_id = client.create_proposal(
            &guild_id,
            &owner,
            &ProposalType::GeneralDecision,
            &String::from_str(&env, "Delegation Proposal"),
            &String::from_str(&env, "Delegation"),
        );

        client.delegate_vote(&guild_id, &member, &admin);
        client.delegate_vote(&guild_id, &contributor, &member);

        client.vote(&proposal_id, &admin, &VoteDecision::For);

        let proposal = client.get_proposal(&proposal_id);
        let end = proposal.voting_end;
        set_ledger_timestamp(&env, end + 1);

        let status = client.finalize_proposal(&proposal_id);
        assert_eq!(status, ProposalStatus::Passed);

        let proposal_after_finalize = client.get_proposal(&proposal_id);
        assert_eq!(proposal_after_finalize.votes_for, 8);

        // Execute to prove lifecycle completion
        let is_executed = client.execute_proposal(&proposal_id, &admin);
        assert!(is_executed);
    }

    #[test]
    #[should_panic(expected = "only passed proposals can be executed")]
    fn test_quorum_rejection_prevents_execution() {
        let env = setup_env();
        let owner = Address::generate(&env);

        set_ledger_timestamp(&env, 1000);
        env.mock_all_auths();

        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        let (guild_id, _admin, _member, contributor) =
            setup_guild_with_members(&env, &client, &owner);

        let proposal_id = client.create_proposal(
            &guild_id,
            &owner,
            &ProposalType::GeneralDecision,
            &String::from_str(&env, "Low Quorum"),
            &String::from_str(&env, "Low quorum"),
        );

        client.vote(&proposal_id, &contributor, &VoteDecision::For);

        let proposal = client.get_proposal(&proposal_id);
        let end = proposal.voting_end;
        set_ledger_timestamp(&env, end + 1);

        let status = client.finalize_proposal(&proposal_id);
        assert_eq!(status, ProposalStatus::Rejected);

        // Should panic since it didn't pass quorum
        client.execute_proposal(&proposal_id, &owner);
    }

    #[test]
    fn test_storage_round_trip_for_votes_delegations_and_configs() {
        let env = setup_env();
        env.mock_all_auths();
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        let owner = Address::generate(&env);
        let guild_id = setup_guild(&client, &env, &owner);
        let voter = Address::generate(&env);
        let delegate = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let proposal_id = storage::get_next_proposal_id(&env);
            let proposal = Proposal {
                id: proposal_id,
                guild_id,
                proposer: owner.clone(),
                proposal_type: ProposalType::GeneralDecision,
                title: String::from_str(&env, "Stored"),
                description: String::from_str(&env, "Stored proposal"),
                voting_start: 100,
                voting_end: 200,
                status: ProposalStatus::Active,
                votes_for: 0,
                votes_against: 0,
                votes_abstain: 0,
                execution_payload: ExecutionPayload::GeneralDecision,
                passed_at: None,
                executed_at: None,
            };

            storage::store_proposal(&env, &proposal);
            storage::store_proposal(&env, &proposal);
            assert_eq!(
                storage::get_proposal(&env, proposal_id).unwrap().title,
                proposal.title
            );
            assert_eq!(storage::get_guild_proposals(&env, guild_id).len(), 1);

            let vote = Vote {
                voter: voter.clone(),
                proposal_id,
                decision: VoteDecision::For,
                weight: 5,
                timestamp: 123,
            };
            storage::store_vote(&env, &vote);
            assert_eq!(
                storage::get_vote(&env, proposal_id, &voter)
                    .unwrap()
                    .decision,
                VoteDecision::For
            );
            assert_eq!(storage::get_all_votes(&env, proposal_id).len(), 1);

            storage::set_delegation(&env, guild_id, &voter, &delegate);
            assert_eq!(
                storage::get_delegate(&env, guild_id, &voter),
                Some(delegate.clone())
            );
            storage::remove_delegation(&env, guild_id, &voter);
            assert_eq!(storage::get_delegate(&env, guild_id, &voter), None);

            assert_eq!(
                storage::get_config(&env, guild_id),
                GovernanceConfig::default()
            );
            let updated = GovernanceConfig {
                quorum_percentage: 45,
                approval_threshold: 70,
                voting_period_days: 5,
                min_proposer_reputation: 2,
            };
            storage::set_config(&env, guild_id, &updated);
            assert_eq!(storage::get_config(&env, guild_id), updated);
        });
    }

    #[test]
    fn test_cancel_proposal_updates_active_list_and_config() {
        let env = setup_env();
        let owner = Address::generate(&env);
        env.mock_all_auths();

        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        let (guild_id, admin, _member, _contributor) =
            setup_guild_with_members(&env, &client, &owner);

        let proposal_a = client.create_proposal(
            &guild_id,
            &owner,
            &ProposalType::GeneralDecision,
            &String::from_str(&env, "A"),
            &String::from_str(&env, "first"),
        );
        let proposal_b = client.create_proposal(
            &guild_id,
            &admin,
            &ProposalType::GeneralDecision,
            &String::from_str(&env, "B"),
            &String::from_str(&env, "second"),
        );

        assert_eq!(client.get_active_proposals(&guild_id).len(), 2);
        assert!(client.cancel_proposal(&proposal_b, &owner));
        assert_eq!(
            client.get_proposal(&proposal_b).status,
            ProposalStatus::Cancelled
        );
        assert_eq!(client.get_active_proposals(&guild_id).len(), 1);
        assert_eq!(
            client.get_active_proposals(&guild_id).get(0).unwrap().id,
            proposal_a
        );

        let new_cfg = GovernanceConfig {
            quorum_percentage: 40,
            approval_threshold: 66,
            voting_period_days: 10,
            min_proposer_reputation: 1,
        };
        assert!(client.update_governance_config(&guild_id, &owner, &new_cfg));

        env.as_contract(&contract_id, || {
            assert_eq!(storage::get_config(&env, guild_id), new_cfg);
        });
    }

    #[test]
    #[should_panic(expected = "execution payload does not match proposal type")]
    fn test_create_proposal_rejects_mismatched_payload() {
        let env = setup_env();
        let owner = Address::generate(&env);
        env.mock_all_auths();

        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        let guild_id = setup_guild(&client, &env, &owner);

        env.as_contract(&contract_id, || {
            proposals::create_proposal(
                &env,
                guild_id,
                owner.clone(),
                ProposalType::AddMember,
                String::from_str(&env, "Bad payload"),
                String::from_str(&env, "mismatch"),
                ExecutionPayload::GeneralDecision,
            );
        });
    }

    #[test]
    #[should_panic(expected = "invalid quorum percentage")]
    fn test_update_governance_config_rejects_invalid_quorum() {
        let env = setup_env();
        let owner = Address::generate(&env);
        env.mock_all_auths();

        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        let guild_id = setup_guild(&client, &env, &owner);

        client.update_governance_config(
            &guild_id,
            &owner,
            &GovernanceConfig {
                quorum_percentage: 0,
                approval_threshold: 60,
                voting_period_days: 7,
                min_proposer_reputation: 0,
            },
        );
    }
}
