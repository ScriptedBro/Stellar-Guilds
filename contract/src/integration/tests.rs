#[cfg(test)]
mod tests {
    use crate::integration::auth;
    use crate::integration::types::{
        ContractType, CrossContractPermission, EventFilter, EventType,
    };
    use crate::interfaces::{
        BountyContractCall, ContractCallResponse, ContractCallResult, GuildContractCall,
        PaymentContractCall,
    };
    use crate::payment::types::DistributionRule;
    use crate::upgrade::types::Version;
    use crate::utils::errors::IntegrationErrorCode;
    use crate::InitializerProof;
    use crate::{guild::types::Role, StellarGuildsContract, StellarGuildsContractClient};
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::xdr::{Hash, ScAddress};
    use soroban_sdk::{vec, Address, Env, IntoVal, String, Symbol, TryFromVal, Vec};

    fn setup() -> (Env, Address, Address) {
        let env = Env::default();
        env.budget().reset_unlimited();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, StellarGuildsContract);
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        (env, admin, contract_id)
    }

    fn empty_filter() -> EventFilter {
        EventFilter {
            has_contract_source: false,
            contract_source: ContractType::Guild,
            has_event_type: false,
            event_type: EventType::GuildCreated,
            subscriber: None,
        }
    }

    #[test]
    fn registry_crud_round_trip() {
        let (env, admin, contract_id) = setup();
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        let guild_contract = Address::generate(&env);
        let bounty_contract = Address::generate(&env);

        assert!(client.register_contract(
            &ContractType::Guild,
            &guild_contract,
            &Version::new(1, 0, 0),
            &admin
        ));
        assert!(client.register_contract(
            &ContractType::Bounty,
            &bounty_contract,
            &Version::new(1, 0, 0),
            &admin
        ));

        assert_eq!(
            client.get_contract_address(&ContractType::Guild),
            guild_contract
        );
        assert_eq!(client.get_all_contracts().len(), 2);

        let updated_address = Address::generate(&env);
        assert!(client.update_contract(
            &ContractType::Guild,
            &updated_address,
            &Version::new(1, 1, 0),
            &admin
        ));

        assert_eq!(
            client.get_contract_address(&ContractType::Guild),
            updated_address
        );
        let contracts = client.get_all_contracts();
        assert!(contracts.iter().any(|contract| {
            contract.contract_type == ContractType::Guild
                && contract.version == Version::new(1, 1, 0)
                && contract.address == updated_address
        }));
    }

    #[test]
    fn event_log_filters_and_subscriptions() {
        let (env, admin, contract_id) = setup();
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        let guild_contract = Address::generate(&env);
        let subscriber = Address::generate(&env);

        client.register_contract(
            &ContractType::Guild,
            &guild_contract,
            &Version::new(1, 0, 0),
            &admin,
        );
        client.subscribe_to_events(
            &subscriber,
            &Vec::from_array(&env, [EventType::GuildCreated]),
        );

        assert!(client.emit_integration_event(
            &EventType::GuildCreated,
            &ContractType::Guild,
            &String::from_str(&env, "guild created"),
            &1u32,
            &admin
        ));
        assert!(client.emit_integration_event(
            &EventType::GuildRoleUpdated,
            &ContractType::Guild,
            &String::from_str(&env, "role updated"),
            &1u32,
            &admin
        ));

        let filtered = client.get_events(
            &EventFilter {
                has_contract_source: true,
                contract_source: ContractType::Guild,
                has_event_type: true,
                event_type: EventType::GuildCreated,
                subscriber: Some(subscriber.clone()),
            },
            &0u64,
            &10u32,
        );

        assert_eq!(filtered.len(), 1);
        assert_eq!(
            filtered.get_unchecked(0).event_type,
            EventType::GuildCreated
        );
        assert_eq!(filtered.get_unchecked(0).contract_address, guild_contract);

        let paged = client.get_events(&empty_filter(), &0u64, &2u32);
        assert_eq!(paged.len(), 2);
    }

    #[test]
    fn cross_contract_auth_uses_registry_matrix() {
        let (env, admin, contract_id) = setup();
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        let guild_contract = Address::generate(&env);
        let bounty_contract = Address::generate(&env);
        let stranger = Address::generate(&env);

        client.register_contract(
            &ContractType::Guild,
            &guild_contract,
            &Version::new(1, 0, 0),
            &admin,
        );
        client.register_contract(
            &ContractType::Bounty,
            &bounty_contract,
            &Version::new(1, 0, 0),
            &admin,
        );

        assert!(client.verify_cross_contract_auth(
            &admin,
            &ContractType::Guild,
            &CrossContractPermission::Admin
        ));
        assert!(client.verify_cross_contract_auth(
            &guild_contract,
            &ContractType::Bounty,
            &CrossContractPermission::Read
        ));
        assert!(!client.verify_cross_contract_auth(
            &stranger,
            &ContractType::Guild,
            &CrossContractPermission::Read
        ));
    }

    #[test]
    fn guild_dispatch_returns_member_data() {
        let (env, admin, contract_id) = setup();
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        let guild_contract = contract_id.clone();
        let guild_id = client.create_guild(
            &String::from_str(&env, "Core Guild"),
            &String::from_str(&env, "Main guild"),
            &admin,
            &None::<InitializerProof>,
        );

        client.register_contract(
            &ContractType::Guild,
            &guild_contract,
            &Version::new(1, 0, 0),
            &admin,
        );

        let result = client.call_guild_contract(
            &admin,
            &GuildContractCall::IsMember(guild_id, admin.clone()),
        );

        assert_eq!(result, Ok(ContractCallResult::Bool(true)));

        let result = client.call_guild_contract(
            &admin,
            &GuildContractCall::HasPermission(guild_id, admin.clone(), Role::Admin),
        );

        assert_eq!(result, Ok(ContractCallResult::Bool(true)));
    }

    #[test]
    fn bounty_dispatch_returns_bounty_data() {
        let (env, admin, contract_id) = setup();
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        let bounty_contract = contract_id.clone();
        let token_admin = Address::generate(&env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin);
        let guild_id = client.create_guild(
            &String::from_str(&env, "Bounty Guild"),
            &String::from_str(&env, "Guild with bounty"),
            &admin,
            &None::<InitializerProof>,
        );

        let bounty_id = client.create_bounty(
            &guild_id,
            &admin,
            &String::from_str(&env, "Ship integration"),
            &String::from_str(&env, "Finish the integration layer"),
            &1000i128,
            &token_contract.address(),
            &(env.ledger().timestamp() + 1000),
        );

        client.register_contract(
            &ContractType::Bounty,
            &bounty_contract,
            &Version::new(1, 0, 1),
            &admin,
        );

        let result = client.call_bounty_contract(&admin, &BountyContractCall::GetBounty(bounty_id));
        match result {
            Ok(ContractCallResult::Bounty(bounty)) => {
                assert_eq!(bounty.id, bounty_id);
                assert_eq!(bounty.guild_id, guild_id);
            }
            _ => panic!("expected bounty response"),
        }
    }

    #[test]
    fn dispatch_requires_registered_or_admin_caller() {
        let (env, admin, contract_id) = setup();
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        let stranger = Address::generate(&env);
        let guild_id = client.create_guild(
            &String::from_str(&env, "Secure Guild"),
            &String::from_str(&env, "Auth checks"),
            &admin,
            &None::<InitializerProof>,
        );

        client.register_contract(
            &ContractType::Guild,
            &contract_id,
            &Version::new(1, 0, 0),
            &admin,
        );

        let result = env.try_invoke_contract::<ContractCallResponse, IntegrationErrorCode>(
            &contract_id,
            &Symbol::new(&env, "call_guild_contract"),
            vec![
                &env,
                stranger.into_val(&env),
                GuildContractCall::IsMember(guild_id, admin.clone()).into_val(&env),
            ],
        );
        assert_eq!(result, Err(Ok(IntegrationErrorCode::Unauthorized)));
    }

    #[test]
    fn remote_call_failures_map_to_integration_errors() {
        let (env, admin, contract_id) = setup();
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        let remote_contract = env.register_contract(None, StellarGuildsContract);
        let remote_client = StellarGuildsContractClient::new(&env, &remote_contract);
        remote_client.initialize(&admin);

        client.register_contract(
            &ContractType::Guild,
            &remote_contract,
            &Version::new(1, 0, 0),
            &admin,
        );

        let result = env.try_invoke_contract::<ContractCallResponse, IntegrationErrorCode>(
            &contract_id,
            &Symbol::new(&env, "call_guild_contract"),
            vec![
                &env,
                admin.clone().into_val(&env),
                GuildContractCall::GetMember(999, admin.clone()).into_val(&env),
            ],
        );
        assert_eq!(
            result,
            Err(Ok(IntegrationErrorCode::CrossContractCallFailed))
        );
    }

    #[test]
    fn three_contract_integration_flow_bounty_treasury_payment() {
        let (env, admin, integration_contract) = setup();
        let hub = StellarGuildsContractClient::new(&env, &integration_contract);

        let treasury_contract = env.register_contract(None, StellarGuildsContract);
        let treasury_client = StellarGuildsContractClient::new(&env, &treasury_contract);
        treasury_client.initialize(&admin);

        let payment_contract = env.register_contract(None, StellarGuildsContract);
        let payment_client = StellarGuildsContractClient::new(&env, &payment_contract);
        payment_client.initialize(&admin);

        hub.register_contract(
            &ContractType::Bounty,
            &integration_contract,
            &Version::new(1, 0, 0),
            &admin,
        );
        hub.register_contract(
            &ContractType::Treasury,
            &treasury_contract,
            &Version::new(1, 0, 0),
            &admin,
        );
        hub.register_contract(
            &ContractType::Payment,
            &payment_contract,
            &Version::new(1, 0, 0),
            &admin,
        );

        let token_admin = Address::generate(&env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin);
        let guild_id = hub.create_guild(
            &String::from_str(&env, "Integration Guild"),
            &String::from_str(&env, "bounty -> treasury -> payment"),
            &admin,
            &None::<InitializerProof>,
        );
        let bounty_id = hub.create_bounty(
            &guild_id,
            &admin,
            &String::from_str(&env, "Ship flow"),
            &String::from_str(&env, "Cover 3-contract integration"),
            &500i128,
            &token_contract.address(),
            &(env.ledger().timestamp() + 1_000),
        );

        let treasury_id = treasury_client.initialize_treasury(
            &guild_id,
            &Vec::from_array(&env, [admin.clone()]),
            &1u32,
        );
        assert!(treasury_client.deposit_treasury(&treasury_id, &admin, &750i128, &None,));

        let pool_id = payment_client.create_payment_pool(
            &500i128,
            &None,
            &DistributionRule::EqualSplit,
            &admin,
        );
        assert!(payment_client.add_recipient(&pool_id, &admin, &100u32, &admin));

        let bounty = hub.call_bounty_contract(
            &treasury_contract,
            &BountyContractCall::GetBounty(bounty_id),
        );
        match bounty {
            Ok(ContractCallResult::Bounty(bounty)) => assert_eq!(bounty.id, bounty_id),
            _ => panic!("expected bounty response"),
        }

        let payment_result = hub.call_payment_contract(
            &treasury_contract,
            &PaymentContractCall::ValidateDistribution(pool_id),
        );
        assert_eq!(payment_result, Ok(ContractCallResult::Bool(true)));
    }

    #[test]
    #[should_panic(expected = "address already registered for another contract")]
    fn registry_rejects_address_collisions() {
        let (env, admin, contract_id) = setup();
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        let shared_address = Address::generate(&env);

        client.register_contract(
            &ContractType::Guild,
            &shared_address,
            &Version::new(1, 0, 0),
            &admin,
        );
        client.register_contract(
            &ContractType::Bounty,
            &shared_address,
            &Version::new(1, 0, 0),
            &admin,
        );
    }

    #[test]
    #[should_panic(expected = "contract type not found")]
    fn calling_unregistered_contract_panics() {
        let (env, admin, contract_id) = setup();
        let client = StellarGuildsContractClient::new(&env, &contract_id);

        let _ = client.call_payment_contract(&admin, &PaymentContractCall::ValidateDistribution(1));
    }

    #[test]
    fn zero_address_is_rejected() {
        let (env, _admin, contract_id) = setup();
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        let zero_contract_address =
            Address::try_from_val(&env, &ScAddress::Contract(Hash([0; 32]))).unwrap();

        assert!(!client.validate_address(&zero_contract_address));
    }

    #[test]
    fn direct_auth_matrix_covers_permissions() {
        let (env, admin, contract_id) = setup();
        let client = StellarGuildsContractClient::new(&env, &contract_id);
        let guild_contract = Address::generate(&env);
        let bounty_contract = Address::generate(&env);
        let treasury_contract = Address::generate(&env);
        let governance_contract = Address::generate(&env);
        let subscription_contract = Address::generate(&env);

        client.register_contract(
            &ContractType::Guild,
            &guild_contract,
            &Version::new(1, 0, 0),
            &admin,
        );
        client.register_contract(
            &ContractType::Bounty,
            &bounty_contract,
            &Version::new(1, 0, 0),
            &admin,
        );
        client.register_contract(
            &ContractType::Treasury,
            &treasury_contract,
            &Version::new(1, 0, 0),
            &admin,
        );
        client.register_contract(
            &ContractType::Governance,
            &governance_contract,
            &Version::new(1, 0, 0),
            &admin,
        );
        client.register_contract(
            &ContractType::Subscription,
            &subscription_contract,
            &Version::new(1, 0, 0),
            &admin,
        );

        env.as_contract(&contract_id, || {
            assert!(auth::verify_cross_contract_auth(
                &env,
                admin.clone(),
                ContractType::Guild,
                CrossContractPermission::Admin,
            ));
            assert!(auth::verify_cross_contract_auth(
                &env,
                guild_contract.clone(),
                ContractType::Bounty,
                CrossContractPermission::Read,
            ));
            assert!(auth::verify_cross_contract_auth(
                &env,
                bounty_contract.clone(),
                ContractType::Treasury,
                CrossContractPermission::Read,
            ));
            assert!(auth::verify_cross_contract_auth(
                &env,
                bounty_contract.clone(),
                ContractType::Bounty,
                CrossContractPermission::EmitEvents,
            ));
            assert!(auth::verify_cross_contract_auth(
                &env,
                governance_contract.clone(),
                ContractType::Treasury,
                CrossContractPermission::Execute,
            ));
            assert!(auth::verify_cross_contract_auth(
                &env,
                governance_contract.clone(),
                ContractType::Guild,
                CrossContractPermission::RegistryWrite,
            ));
            assert!(auth::verify_cross_contract_auth(
                &env,
                subscription_contract.clone(),
                ContractType::Guild,
                CrossContractPermission::Read,
            ));
            assert!(!auth::verify_cross_contract_auth(
                &env,
                guild_contract.clone(),
                ContractType::Treasury,
                CrossContractPermission::Execute,
            ));
            assert!(!auth::verify_cross_contract_auth(
                &env,
                guild_contract.clone(),
                ContractType::Bounty,
                CrossContractPermission::EmitEvents,
            ));
            assert!(!auth::verify_cross_contract_auth(
                &env,
                Address::generate(&env),
                ContractType::Guild,
                CrossContractPermission::Read,
            ));
        });
    }

    #[test]
    #[should_panic(expected = "only admin can manage integration layer")]
    fn require_admin_rejects_non_admin() {
        let (env, _admin, contract_id) = setup();
        let outsider = Address::generate(&env);

        env.as_contract(&contract_id, || auth::require_admin(&env, &outsider));
    }
}
