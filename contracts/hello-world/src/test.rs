#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
        token::{Client as TokenClient, StellarAssetClient},
        Address, Env, IntoVal,
    };

    /// Helper: sets up env, deploys contract, mints USDC to contract address.
    fn setup() -> (Env, Address, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();

        // Parties
        let admin = Address::generate(&env);
        let student = Address::generate(&env);

        // Deploy a mock USDC token (Stellar Asset Contract)
        let token_admin = Address::generate(&env);
        let token_contract_id = env.register_stellar_asset_contract(token_admin.clone());
        let token_sac = StellarAssetClient::new(&env, &token_contract_id);
        let token = TokenClient::new(&env, &token_contract_id);

        // Deploy the ScholarPay contract
        let contract_id = env.register_contract(None, ScholarPayContract);
        let client = ScholarPayContractClient::new(&env, &contract_id);

        // Initialize
        client.initialize(&admin, &token_contract_id);

        // Mint 1000 USDC (in smallest units) to the contract so it can pay out
        token_sac.mint(&contract_id, &1_000);

        (env, contract_id, admin, student, token_contract_id)
    }

    // TEST 1 — Happy path: register student and release scholarship end-to-end
    #[test]
    fn test_happy_path_register_and_release() {
        let (env, contract_id, admin, student, token_contract_id) = setup();
        let client = ScholarPayContractClient::new(&env, &contract_id);
        let token = TokenClient::new(&env, &token_contract_id);

        // Register the student with a 500-unit scholarship
        client.register_student(&student, &500);

        // Balance before release
        let before = token.balance(&student);
        assert_eq!(before, 0);

        // Admin releases funds
        client.release(&student);

        // Student should now have 500 units
        let after = token.balance(&student);
        assert_eq!(after, 500);
    }

    // TEST 2 — Edge case: double-release should panic
    #[test]
    #[should_panic(expected = "scholarship already released")]
    fn test_double_release_panics() {
        let (env, contract_id, admin, student, _) = setup();
        let client = ScholarPayContractClient::new(&env, &contract_id);

        client.register_student(&student, &200);
        client.release(&student);
        // Second release must panic
        client.release(&student);
    }

    // TEST 3 — State verification: released flag is true after release
    #[test]
    fn test_state_after_release() {
        let (env, contract_id, admin, student, _) = setup();
        let client = ScholarPayContractClient::new(&env, &contract_id);

        client.register_student(&student, &300);

        // Before release: released == false
        let before = client.get_scholarship(&student);
        assert!(!before.released);
        assert_eq!(before.amount, 300);

        client.release(&student);

        // After release: released == true
        let after = client.get_scholarship(&student);
        assert!(after.released);
    }

    // TEST 4 — Edge case: registering unregistered student panics on release
    #[test]
    #[should_panic(expected = "student not registered")]
    fn test_release_unregistered_student_panics() {
        let (env, contract_id, admin, student, _) = setup();
        let client = ScholarPayContractClient::new(&env, &contract_id);

        // Never registered — should panic
        client.release(&student);
    }

    // TEST 5 — Edge case: registering a student with zero amount should panic
    #[test]
    #[should_panic(expected = "amount must be positive")]
    fn test_register_zero_amount_panics() {
        let (env, contract_id, admin, student, _) = setup();
        let client = ScholarPayContractClient::new(&env, &contract_id);

        client.register_student(&student, &0);
    }
}
