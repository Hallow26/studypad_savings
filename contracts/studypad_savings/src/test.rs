#[cfg(test)]
mod tests {
    use soroban_sdk::{
        testutils::{Address as _, Ledger, LedgerInfo},
        token,
        Address, Bytes, Env,
    };

    use crate::{StudyPadContract, StudyPadContractClient};

    // ─── Helpers ─────────────────────────────────────────────────────────────

    fn setup_usdc(env: &Env, admin: &Address, recipient: &Address, amount: i128) -> Address {
        let token_id = env.register_stellar_asset_contract(admin.clone());
        let token_admin = token::StellarAssetClient::new(env, &token_id);
        token_admin.mint(recipient, &amount);
        token_id
    }

    fn deploy_contract(env: &Env, admin: &Address, usdc: &Address) -> StudyPadContractClient {
        let contract_id = env.register_contract(None, StudyPadContract);
        let client = StudyPadContractClient::new(env, &contract_id);
        client.initialize(admin, usdc);
        client
    }

    fn label(env: &Env, s: &[u8]) -> Bytes {
        Bytes::from_slice(env, s)
    }

    // ─── Test 1: Happy path ───────────────────────────────────────────────────
    #[test]
    fn test_happy_path_goal_met_and_withdraw() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let student = Address::generate(&env);

        let usdc = setup_usdc(&env, &admin, &student, 200_000_000);
        let client = deploy_contract(&env, &admin, &usdc);

        client.create_goal(
            &student,
            &label(&env, b"Tuition Sem 2 2025"),
            &150_000_000_i128,
            &1000_u32,
        );

        client.deposit(&student, &80_000_000_i128);
        client.deposit(&student, &70_000_000_i128);
        client.withdraw(&student);

        let goal = client.get_goal(&student);
        assert!(goal.completed);
        assert_eq!(goal.balance, 0);

        let token_client = token::Client::new(&env, &usdc);
        assert_eq!(token_client.balance(&student), 200_000_000_i128);
    }

    // ─── Test 2: Edge case — early withdraw blocked ───────────────────────────
    #[test]
    #[should_panic(expected = "goal not yet met and deadline not passed")]
    fn test_early_withdraw_blocked() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let student = Address::generate(&env);

        let usdc = setup_usdc(&env, &admin, &student, 100_000_000);
        let client = deploy_contract(&env, &admin, &usdc);

        client.create_goal(
            &student,
            &label(&env, b"Laptop Fund"),
            &100_000_000_i128,
            &9999_u32,
        );

        client.deposit(&student, &50_000_000_i128);
        client.withdraw(&student); // must panic
    }

    // ─── Test 3: State verification ───────────────────────────────────────────
    #[test]
    fn test_state_reflects_cumulative_deposits() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let student = Address::generate(&env);

        let usdc = setup_usdc(&env, &admin, &student, 300_000_000);
        let client = deploy_contract(&env, &admin, &usdc);

        client.create_goal(
            &student,
            &label(&env, b"Exchange Program"),
            &250_000_000_i128,
            &5000_u32,
        );

        client.deposit(&student, &100_000_000_i128);
        assert_eq!(client.get_goal(&student).balance, 100_000_000);

        client.deposit(&student, &75_000_000_i128);
        let goal = client.get_goal(&student);
        assert_eq!(goal.balance, 175_000_000);
        assert_eq!(goal.target, 250_000_000);
        assert!(!goal.completed);
    }

    // ─── Test 4: Deadline passed — withdraw allowed ───────────────────────────
    #[test]
    fn test_withdraw_allowed_after_deadline() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let student = Address::generate(&env);

        let usdc = setup_usdc(&env, &admin, &student, 200_000_000);
        let client = deploy_contract(&env, &admin, &usdc);

        client.create_goal(
            &student,
            &label(&env, b"Partial Emergency"),
            &200_000_000_i128,
            &100_u32,
        );

        client.deposit(&student, &60_000_000_i128);

        env.ledger().set(LedgerInfo {
            sequence_number: 101,
            timestamp: 0,
            protocol_version: 20,
            network_id: Default::default(),
            base_reserve: 5_000_000,
            min_temp_entry_ttl: 1,
            min_persistent_entry_ttl: 1,
            max_entry_ttl: 6_312_000,
        });

        client.withdraw(&student);

        let goal = client.get_goal(&student);
        assert!(goal.completed);
        assert_eq!(goal.balance, 0);
    }

    // ─── Test 5: Double initialize panics ─────────────────────────────────────
    #[test]
    #[should_panic(expected = "already initialized")]
    fn test_double_initialize_panics() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let usdc = setup_usdc(&env, &admin, &admin, 0);
        let client = deploy_contract(&env, &admin, &usdc);

        client.initialize(&admin, &usdc); // must panic
    }
}
