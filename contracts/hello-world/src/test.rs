#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
        token::{Client as TokenClient, StellarAssetClient},
        Address, Env, IntoVal,
    };

    // -----------------------------------------------------------------------
    // Helper: deploy a mock USDC token and mint `amount` to `recipient`
    // -----------------------------------------------------------------------
    fn setup_token(env: &Env, admin: &Address, recipient: &Address, amount: i128) -> Address {
        let token_id = env.register_stellar_asset_contract(admin.clone());
        let sac = StellarAssetClient::new(env, &token_id);
        sac.mint(recipient, &amount);
        token_id
    }

    // -----------------------------------------------------------------------
    // Helper: deploy SariEscrow contract and return its client
    // -----------------------------------------------------------------------
    fn setup_contract(env: &Env) -> SariEscrowClient {
        let contract_id = env.register_contract(None, SariEscrow);
        SariEscrowClient::new(env, &contract_id)
    }

    // -----------------------------------------------------------------------
    // Test 1 – Happy path
    // Full MVP flow: create order → mark shipped → confirm receipt
    // Buyer's USDC should end up in seller's wallet
    // -----------------------------------------------------------------------
    #[test]
    fn test_happy_path_full_flow() {
        let env = Env::default();
        env.mock_all_auths();

        let buyer  = Address::generate(&env);
        let seller = Address::generate(&env);
        let admin  = Address::generate(&env);

        // Mint 1000 USDC to buyer
        let token = setup_token(&env, &admin, &buyer, 1_000);
        let client = setup_contract(&env);

        // Give contract a token allowance via the buyer's mock auth
        let token_client = TokenClient::new(&env, &token);
        token_client.approve(&buyer, &client.address, &1_000, &200);

        // Step 1: Buyer creates order (locks 500 USDC into escrow)
        let order_id = client.create_order(&buyer, &seller, &token, &500, &admin);
        assert_eq!(order_id, 1);

        // Step 2: Seller marks order as shipped
        client.mark_shipped(&seller, &order_id);

        // Step 3: Buyer confirms receipt → USDC flows to seller
        client.confirm_receipt(&buyer, &order_id);

        // Verify seller received the funds
        assert_eq!(token_client.balance(&seller), 500);
        assert_eq!(token_client.balance(&buyer), 500); // 1000 - 500

        // Verify order status is Completed
        let order = client.get_order(&order_id);
        assert_eq!(order.status, OrderStatus::Completed);
    }

    // -----------------------------------------------------------------------
    // Test 2 – Edge case / failure
    // A non-seller address attempting to call mark_shipped should panic
    // -----------------------------------------------------------------------
    #[test]
    #[should_panic(expected = "only the seller can mark an order as shipped")]
    fn test_unauthorized_mark_shipped() {
        let env = Env::default();
        env.mock_all_auths();

        let buyer    = Address::generate(&env);
        let seller   = Address::generate(&env);
        let imposter = Address::generate(&env);
        let admin    = Address::generate(&env);

        let token = setup_token(&env, &admin, &buyer, 1_000);
        let client = setup_contract(&env);
        let token_client = TokenClient::new(&env, &token);
        token_client.approve(&buyer, &client.address, &1_000, &200);

        let order_id = client.create_order(&buyer, &seller, &token, &300, &admin);

        // Imposter tries to mark as shipped – should panic
        client.mark_shipped(&imposter, &order_id);
    }

    // -----------------------------------------------------------------------
    // Test 3 – State verification
    // After create_order, contract storage must reflect Funded status
    // and the contract must hold the locked tokens
    // -----------------------------------------------------------------------
    #[test]
    fn test_state_after_create_order() {
        let env = Env::default();
        env.mock_all_auths();

        let buyer  = Address::generate(&env);
        let seller = Address::generate(&env);
        let admin  = Address::generate(&env);

        let token = setup_token(&env, &admin, &buyer, 1_000);
        let client = setup_contract(&env);
        let token_client = TokenClient::new(&env, &token);
        token_client.approve(&buyer, &client.address, &1_000, &200);

        let order_id = client.create_order(&buyer, &seller, &token, &400, &admin);

        // Fetch order from on-chain storage and assert fields
        let order = client.get_order(&order_id);
        assert_eq!(order.status, OrderStatus::Funded);
        assert_eq!(order.amount, 400);
        assert_eq!(order.buyer, buyer);
        assert_eq!(order.seller, seller);

        // Contract should hold exactly 400 USDC
        assert_eq!(token_client.balance(&client.address), 400);
        // Buyer should have 600 remaining
        assert_eq!(token_client.balance(&buyer), 600);
    }

    // -----------------------------------------------------------------------
    // Test 4 – Dispute & admin refund
    // Buyer raises dispute; admin resolves in buyer's favour → refund issued
    // -----------------------------------------------------------------------
    #[test]
    fn test_dispute_and_refund() {
        let env = Env::default();
        env.mock_all_auths();

        let buyer  = Address::generate(&env);
        let seller = Address::generate(&env);
        let admin  = Address::generate(&env);

        let token = setup_token(&env, &admin, &buyer, 1_000);
        let client = setup_contract(&env);
        let token_client = TokenClient::new(&env, &token);
        token_client.approve(&buyer, &client.address, &1_000, &200);

        let order_id = client.create_order(&buyer, &seller, &token, &700, &admin);
        client.mark_shipped(&seller, &order_id);

        // Buyer not happy – raises dispute
        client.raise_dispute(&buyer, &order_id);
        let order = client.get_order(&order_id);
        assert_eq!(order.status, OrderStatus::Disputed);

        // Admin refunds buyer
        client.resolve_dispute(&admin, &order_id, &true);
        let order = client.get_order(&order_id);
        assert_eq!(order.status, OrderStatus::Refunded);

        // Buyer gets their 700 USDC back
        assert_eq!(token_client.balance(&buyer), 1_000);
        assert_eq!(token_client.balance(&seller), 0);
    }

    // -----------------------------------------------------------------------
    // Test 5 – Prevent double-completion
    // Calling confirm_receipt twice on the same Completed order should panic
    // -----------------------------------------------------------------------
    #[test]
    #[should_panic(expected = "order has not been marked as shipped yet")]
    fn test_cannot_double_confirm() {
        let env = Env::default();
        env.mock_all_auths();

        let buyer  = Address::generate(&env);
        let seller = Address::generate(&env);
        let admin  = Address::generate(&env);

        let token = setup_token(&env, &admin, &buyer, 1_000);
        let client = setup_contract(&env);
        let token_client = TokenClient::new(&env, &token);
        token_client.approve(&buyer, &client.address, &1_000, &200);

        let order_id = client.create_order(&buyer, &seller, &token, &200, &admin);
        client.mark_shipped(&seller, &order_id);
        client.confirm_receipt(&buyer, &order_id); // first confirmation – ok

        // Second confirm on same order – status is Completed, not Shipped → panic
        client.confirm_receipt(&buyer, &order_id);
    }
}