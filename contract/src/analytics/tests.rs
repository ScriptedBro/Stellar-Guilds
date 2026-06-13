#[cfg(test)]
mod tests {
    use crate::analytics::types::{
        BudgetUtilization, CategoryBreakdown, SpendingForecast, SpendingSummary, SpendingTrend,
        TreasurySnapshot,
    };
    use crate::treasury::types::{TransactionStatus, TransactionType};
    use crate::InitializerProof;
    use crate::StellarGuildsContract;
    use crate::StellarGuildsContractClient;
    use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
    use soroban_sdk::{Address, Env, String, Vec};

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

    fn create_treasury(
        env: &Env,
        client: &StellarGuildsContractClient<'_>,
        guild_id: u64,
    ) -> (u64, Address, Address, Address) {
        let owner = Address::generate(&env);
        let signer1 = Address::generate(&env);
        let signer2 = Address::generate(&env);

        env.mock_all_auths();

        let mut signers = Vec::new(env);
        signers.push_back(owner.clone());
        signers.push_back(signer1.clone());
        signers.push_back(signer2.clone());

        let treasury_id = client.initialize_treasury(&guild_id, &signers, &2u32);

        (treasury_id, owner, signer1, signer2)
    }

    #[test]
    fn test_spending_summary_basic() {
        let env = setup_env();
        set_ledger_timestamp(&env, 1000);
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        let guild_owner = Address::generate(&env);
        let guild_id = setup_guild(&client, &env, &guild_owner);

        let (treasury_id, owner, signer1, signer2) = create_treasury(&env, &client, guild_id);
        let depositor = Address::generate(&env);

        // Two deposits at different timestamps
        set_ledger_timestamp(&env, 2000);
        client.deposit_treasury(&treasury_id, &depositor, &1000i128, &None);

        set_ledger_timestamp(&env, 3000);
        client.deposit_treasury(&treasury_id, &depositor, &500i128, &None);

        // Query the full period
        let summary = client.get_spending_summary(&treasury_id, &1500u64, &3500u64);
        assert_eq!(summary.treasury_id, treasury_id);
        assert_eq!(summary.total_deposits, 1500);
        assert_eq!(summary.total_withdrawals, 0);
        assert_eq!(summary.net_flow, 1500);
        assert_eq!(summary.tx_count, 2);
        assert_eq!(summary.avg_tx_amount, 750); // (1000 + 500) / 2
    }

    #[test]
    fn test_spending_summary_empty_period() {
        let env = setup_env();
        set_ledger_timestamp(&env, 1000);
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        let guild_owner = Address::generate(&env);
        let guild_id = setup_guild(&client, &env, &guild_owner);

        let (treasury_id, _, _, _) = create_treasury(&env, &client, guild_id);
        let depositor = Address::generate(&env);

        // Deposit at timestamp 2000
        set_ledger_timestamp(&env, 2000);
        client.deposit_treasury(&treasury_id, &depositor, &1000i128, &None);

        // Query period that does NOT contain any transactions
        let summary = client.get_spending_summary(&treasury_id, &5000u64, &6000u64);
        assert_eq!(summary.total_deposits, 0);
        assert_eq!(summary.total_withdrawals, 0);
        assert_eq!(summary.net_flow, 0);
        assert_eq!(summary.tx_count, 0);
        assert_eq!(summary.avg_tx_amount, 0);
    }

    #[test]
    fn test_spending_summary_with_withdrawals() {
        let env = setup_env();
        set_ledger_timestamp(&env, 1000);
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        let guild_owner = Address::generate(&env);
        let guild_id = setup_guild(&client, &env, &guild_owner);

        let (treasury_id, owner, signer1, signer2) = create_treasury(&env, &client, guild_id);
        let recipient = Address::generate(&env);
        let depositor = Address::generate(&env);
        let reason = String::from_str(&env, "test withdrawal");

        // Deposit
        set_ledger_timestamp(&env, 2000);
        client.deposit_treasury(&treasury_id, &depositor, &2000i128, &None);

        // Propose + approve + execute withdrawal
        set_ledger_timestamp(&env, 3000);
        let tx_id =
            client.propose_withdrawal(&treasury_id, &signer1, &recipient, &500i128, &None, &reason);
        client.approve_transaction(&tx_id, &signer2);
        client.execute_transaction(&tx_id, &owner);

        // Query full period
        let summary = client.get_spending_summary(&treasury_id, &1500u64, &4000u64);
        assert_eq!(summary.total_deposits, 2000);
        assert_eq!(summary.total_withdrawals, 500);
        assert_eq!(summary.net_flow, 1500);
        assert_eq!(summary.tx_count, 2); // deposit + withdrawal
    }

    #[test]
    fn test_budget_utilization() {
        let env = setup_env();
        set_ledger_timestamp(&env, 1000);
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        let guild_owner = Address::generate(&env);
        let guild_id = setup_guild(&client, &env, &guild_owner);

        let (treasury_id, owner, signer1, signer2) = create_treasury(&env, &client, guild_id);
        let depositor = Address::generate(&env);
        let recipient = Address::generate(&env);
        let reason = String::from_str(&env, "test");

        // Set budget and fund treasury
        let category = String::from_str(&env, "withdrawal");
        client.set_budget(&treasury_id, &category, &1000i128, &3600u64, &owner);
        client.deposit_treasury(&treasury_id, &depositor, &5000i128, &None);

        // Spend 300 of 1000 budget (= 30%)
        set_ledger_timestamp(&env, 1500);
        let tx_id =
            client.propose_withdrawal(&treasury_id, &signer1, &recipient, &300i128, &None, &reason);
        client.approve_transaction(&tx_id, &signer2);
        client.execute_transaction(&tx_id, &owner);

        let utilization = client.get_budget_utilization(&treasury_id);
        assert_eq!(utilization.len(), 1);

        let bu = utilization.get(0).unwrap();
        assert_eq!(bu.allocated, 1000);
        assert_eq!(bu.spent, 300);
        assert_eq!(bu.remaining, 700);
        assert_eq!(bu.utilization_bps, 3000); // 30% = 3000 bps
    }

    #[test]
    fn test_category_breakdown() {
        let env = setup_env();
        set_ledger_timestamp(&env, 1000);
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        let guild_owner = Address::generate(&env);
        let guild_id = setup_guild(&client, &env, &guild_owner);

        let (treasury_id, _, _, _) = create_treasury(&env, &client, guild_id);
        let depositor = Address::generate(&env);

        // Multiple deposits
        set_ledger_timestamp(&env, 2000);
        client.deposit_treasury(&treasury_id, &depositor, &1000i128, &None);
        set_ledger_timestamp(&env, 2500);
        client.deposit_treasury(&treasury_id, &depositor, &500i128, &None);

        let breakdown = client.get_category_breakdown(&treasury_id, &1500u64, &3500u64);

        // Should have one category: Deposit
        assert!(breakdown.len() >= 1);
        let deposit_cat = breakdown.get(0).unwrap();
        assert_eq!(deposit_cat.total_amount, 1500);
        assert_eq!(deposit_cat.tx_count, 2);
    }

    #[test]
    fn test_spending_trend() {
        let env = setup_env();
        set_ledger_timestamp(&env, 1000);
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        let guild_owner = Address::generate(&env);
        let guild_id = setup_guild(&client, &env, &guild_owner);

        let (treasury_id, _, _, _) = create_treasury(&env, &client, guild_id);
        let depositor = Address::generate(&env);

        // Period 1: deposit 1000
        set_ledger_timestamp(&env, 2000);
        client.deposit_treasury(&treasury_id, &depositor, &1000i128, &None);

        // Period 2: deposit 2000 (100% increase)
        set_ledger_timestamp(&env, 5000);
        client.deposit_treasury(&treasury_id, &depositor, &2000i128, &None);

        let trend = client.get_spending_trend(&treasury_id, &1500u64, &3000u64, &4000u64, &6000u64);

        // Deposits went from 1000 to 2000 = +100% = +10000 bps
        assert_eq!(trend.deposits_change_bps, 10000);
    }

    #[test]
    fn test_spending_forecast() {
        let env = setup_env();
        set_ledger_timestamp(&env, 1000);
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        let guild_owner = Address::generate(&env);
        let guild_id = setup_guild(&client, &env, &guild_owner);

        let (treasury_id, _, _, _) = create_treasury(&env, &client, guild_id);
        let depositor = Address::generate(&env);

        // Create deposits across multiple periods
        // Period 1 (1000-2000): 1000
        set_ledger_timestamp(&env, 1500);
        client.deposit_treasury(&treasury_id, &depositor, &1000i128, &None);

        // Period 2 (2000-3000): 2000
        set_ledger_timestamp(&env, 2500);
        client.deposit_treasury(&treasury_id, &depositor, &2000i128, &None);

        // Period 3 (3000-4000): 3000
        set_ledger_timestamp(&env, 3500);
        client.deposit_treasury(&treasury_id, &depositor, &3000i128, &None);

        // Forecast from current time = 4000, 3 periods of 1000s each
        set_ledger_timestamp(&env, 4000);
        let forecast = client.get_spending_forecast(&treasury_id, &3u32, &1000u64);

        // Average deposits = (1000 + 2000 + 3000) / 3 = 2000
        assert_eq!(forecast.projected_deposits, 2000);
        assert_eq!(forecast.projected_withdrawals, 0);
        assert_eq!(forecast.projected_net_flow, 2000);
        assert_eq!(forecast.periods_analyzed, 3);
    }

    #[test]
    fn test_treasury_snapshots() {
        let env = setup_env();
        set_ledger_timestamp(&env, 1000);
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        let guild_owner = Address::generate(&env);
        let guild_id = setup_guild(&client, &env, &guild_owner);

        let (treasury_id, _, _, _) = create_treasury(&env, &client, guild_id);
        let depositor = Address::generate(&env);

        // Three deposits should create three snapshots
        set_ledger_timestamp(&env, 2000);
        client.deposit_treasury(&treasury_id, &depositor, &100i128, &None);

        set_ledger_timestamp(&env, 3000);
        client.deposit_treasury(&treasury_id, &depositor, &200i128, &None);

        set_ledger_timestamp(&env, 4000);
        client.deposit_treasury(&treasury_id, &depositor, &300i128, &None);

        let snapshots = client.get_treasury_snapshots(&treasury_id, &10u32);
        assert_eq!(snapshots.len(), 3);

        // Verify order: oldest first
        let first = snapshots.get(0).unwrap();
        assert_eq!(first.balance_xlm, 100);
        assert_eq!(first.total_deposits, 100);

        let last = snapshots.get(2).unwrap();
        assert_eq!(last.balance_xlm, 600); // 100 + 200 + 300
        assert_eq!(last.total_deposits, 600);
    }

    #[test]
    fn test_snapshot_limit() {
        let env = setup_env();
        set_ledger_timestamp(&env, 1000);
        let contract_id = register_and_init_contract(&env);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        env.mock_all_auths();
        let guild_owner = Address::generate(&env);
        let guild_id = setup_guild(&client, &env, &guild_owner);

        let (treasury_id, _, _, _) = create_treasury(&env, &client, guild_id);
        let depositor = Address::generate(&env);

        // Create 5 snapshots
        for i in 0u64..5 {
            set_ledger_timestamp(&env, 2000 + i * 1000);
            client.deposit_treasury(&treasury_id, &depositor, &100i128, &None);
        }

        // Request only 2 most recent
        let snapshots = client.get_treasury_snapshots(&treasury_id, &2u32);
        assert_eq!(snapshots.len(), 2);

        // The last snapshot should have balance 500 (5 * 100)
        let last = snapshots.get(1).unwrap();
        assert_eq!(last.balance_xlm, 500);
    }
}
