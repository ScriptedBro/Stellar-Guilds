#[cfg(test)]
mod tests {
    use crate::guild::types::Role;
    use crate::reputation::types::{BadgeType, ContributionType};
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
        let name = String::from_str(env, "Test Guild");
        let description = String::from_str(env, "A test guild");
        client.create_guild(&name, &description, owner, &None::<InitializerProof>)
    }

    #[test]
    fn test_record_contribution() {
        let env = setup_env();
        set_ledger_timestamp(&env, 1000);
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        let owner = Address::generate(&env);
        let guild_id = setup_guild(&client, &env, &owner);

        let contributor = Address::generate(&env);
        client.add_member(&guild_id, &contributor, &Role::Contributor, &owner);

        // Record a bounty completion
        client.record_contribution(
            &guild_id,
            &contributor,
            &ContributionType::BountyCompleted,
            &1u64,
        );

        let profile = client.get_reputation(&guild_id, &contributor);
        assert_eq!(profile.total_score, 100); // POINTS_BOUNTY_COMPLETED
        assert_eq!(profile.contributions_count, 1);
    }

    #[test]
    fn test_multiple_contributions() {
        let env = setup_env();
        set_ledger_timestamp(&env, 1000);
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        let owner = Address::generate(&env);
        let guild_id = setup_guild(&client, &env, &owner);

        let contributor = Address::generate(&env);
        client.add_member(&guild_id, &contributor, &Role::Contributor, &owner);

        // Record different contribution types
        client.record_contribution(
            &guild_id,
            &contributor,
            &ContributionType::BountyCompleted,
            &1u64,
        );
        client.record_contribution(
            &guild_id,
            &contributor,
            &ContributionType::MilestoneApproved,
            &2u64,
        );
        client.record_contribution(&guild_id, &contributor, &ContributionType::VoteCast, &3u64);

        let profile = client.get_reputation(&guild_id, &contributor);
        // 100 (bounty) + 50 (milestone) + 5 (vote) = 155
        assert_eq!(profile.total_score, 155);
        assert_eq!(profile.contributions_count, 3);

        // Verify contribution history
        let history = client.get_reputation_contributions(&guild_id, &contributor, &10u32);
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn test_reputation_decay() {
        let env = setup_env();
        set_ledger_timestamp(&env, 1000);
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        let owner = Address::generate(&env);
        let guild_id = setup_guild(&client, &env, &owner);

        let contributor = Address::generate(&env);
        client.add_member(&guild_id, &contributor, &Role::Contributor, &owner);

        // Record contribution at t=1000
        client.record_contribution(
            &guild_id,
            &contributor,
            &ContributionType::BountyCompleted,
            &1u64,
        );

        let profile_before = client.get_reputation(&guild_id, &contributor);
        assert_eq!(profile_before.decayed_score, 100);

        // Advance time by 1 decay period (604800 seconds = 1 week)
        set_ledger_timestamp(&env, 1000 + 604_800);

        let profile_after = client.get_reputation(&guild_id, &contributor);
        // After 1 period: 100 * 99/100 = 99
        assert_eq!(profile_after.decayed_score, 99);
        // Total score should remain unchanged
        assert_eq!(profile_after.total_score, 100);

        // Advance by 10 more periods
        set_ledger_timestamp(&env, 1000 + 604_800 * 11);

        let profile_later = client.get_reputation(&guild_id, &contributor);
        // After 11 periods: 100 * (99/100)^11 â‰ˆ 89
        assert!(profile_later.decayed_score < 100);
        assert!(profile_later.decayed_score > 80);
    }

    #[test]
    fn test_governance_weight() {
        let env = setup_env();
        set_ledger_timestamp(&env, 1000);
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        let owner = Address::generate(&env);
        let guild_id = setup_guild(&client, &env, &owner);

        let contributor = Address::generate(&env);
        client.add_member(&guild_id, &contributor, &Role::Contributor, &owner);

        // Before any contributions: weight = role_weight(Contributor) = 1
        let weight_before = client.get_governance_weight_for(&guild_id, &contributor);
        assert_eq!(weight_before, 1); // role weight only

        // Record contributions to build reputation = 100
        client.record_contribution(
            &guild_id,
            &contributor,
            &ContributionType::BountyCompleted,
            &1u64,
        );

        // After: weight = role_weight(1) + sqrt(100) = 1 + 10 = 11
        let weight_after = client.get_governance_weight_for(&guild_id, &contributor);
        assert_eq!(weight_after, 11);
    }

    #[test]
    fn test_badge_first_contribution() {
        let env = setup_env();
        set_ledger_timestamp(&env, 1000);
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        let owner = Address::generate(&env);
        let guild_id = setup_guild(&client, &env, &owner);

        let contributor = Address::generate(&env);
        client.add_member(&guild_id, &contributor, &Role::Contributor, &owner);

        // First contribution should award "First Contribution" badge
        client.record_contribution(&guild_id, &contributor, &ContributionType::VoteCast, &1u64);

        let badges = client.get_reputation_badges(&guild_id, &contributor);
        assert_eq!(badges.len(), 1);
        assert_eq!(
            badges.get(0).unwrap().badge_type,
            BadgeType::FirstContribution
        );
    }

    #[test]
    fn test_badge_veteran() {
        let env = setup_env();
        set_ledger_timestamp(&env, 1000);
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        let owner = Address::generate(&env);
        let guild_id = setup_guild(&client, &env, &owner);

        let contributor = Address::generate(&env);
        client.add_member(&guild_id, &contributor, &Role::Contributor, &owner);

        // Record enough bounties to reach > 1000 points (11 bounties * 100 = 1100)
        for i in 0u64..11 {
            client.record_contribution(
                &guild_id,
                &contributor,
                &ContributionType::BountyCompleted,
                &(i + 1),
            );
        }

        let profile = client.get_reputation(&guild_id, &contributor);
        assert!(profile.total_score > 1000);

        let badges = client.get_reputation_badges(&guild_id, &contributor);
        // Should have FirstContribution + BountyHunter (5+) + Veteran (>1000)
        let mut has_veteran = false;
        let mut has_bounty_hunter = false;
        let mut has_first = false;
        for badge in badges.iter() {
            match badge.badge_type {
                BadgeType::Veteran => has_veteran = true,
                BadgeType::BountyHunter => has_bounty_hunter = true,
                BadgeType::FirstContribution => has_first = true,
                _ => {}
            }
        }
        assert!(has_veteran);
        assert!(has_bounty_hunter);
        assert!(has_first);
    }

    #[test]
    fn test_cross_guild_reputation() {
        let env = setup_env();
        set_ledger_timestamp(&env, 1000);
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        let owner = Address::generate(&env);
        let guild1 = setup_guild(&client, &env, &owner);
        let guild2 = setup_guild(&client, &env, &owner);

        let contributor = Address::generate(&env);
        client.add_member(&guild1, &contributor, &Role::Contributor, &owner);
        client.add_member(&guild2, &contributor, &Role::Contributor, &owner);

        // Contributions in guild 1
        client.record_contribution(
            &guild1,
            &contributor,
            &ContributionType::BountyCompleted,
            &1u64,
        );

        // Contributions in guild 2
        client.record_contribution(
            &guild2,
            &contributor,
            &ContributionType::MilestoneApproved,
            &2u64,
        );

        // Global reputation should aggregate both
        let global = client.get_reputation_global(&contributor);
        assert_eq!(global, 150); // 100 + 50
    }

    #[test]
    fn test_no_reputation_fallback() {
        let env = setup_env();
        set_ledger_timestamp(&env, 1000);
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        let owner = Address::generate(&env);
        let guild_id = setup_guild(&client, &env, &owner);

        let member = Address::generate(&env);
        client.add_member(&guild_id, &member, &Role::Contributor, &owner);

        // No contributions at all
        // Weight should still equal role_weight (Contributor = 1)
        let weight = client.get_governance_weight_for(&guild_id, &member);
        assert_eq!(weight, 1);

        // Global reputation should be 0
        let global = client.get_reputation_global(&member);
        assert_eq!(global, 0);
    }
}
