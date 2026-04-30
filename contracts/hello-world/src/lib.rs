#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, Symbol, token,
};

// ---------------------------------------------------------------------------
// Storage key enum – each variant maps to one slot in persistent storage
// ---------------------------------------------------------------------------
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Order(u64),      // per-order data, keyed by order id
    Counter,         // monotonically incrementing order id
}

// ---------------------------------------------------------------------------
// Order status – tracks lifecycle of every trade
// ---------------------------------------------------------------------------
#[contracttype]
#[derive(Clone, PartialEq)]
pub enum OrderStatus {
    Funded,     // buyer locked USDC into contract
    Shipped,    // seller marked as shipped
    Completed,  // buyer confirmed receipt → seller paid
    Disputed,   // buyer raised dispute
    Refunded,   // admin resolved dispute in buyer's favour
}

// ---------------------------------------------------------------------------
// Core order struct stored on-chain for every transaction
// ---------------------------------------------------------------------------
#[contracttype]
#[derive(Clone)]
pub struct Order {
    pub id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub token: Address,   // USDC asset contract address
    pub amount: i128,     // in stroops / USDC smallest unit
    pub status: OrderStatus,
    pub admin: Address,   // marketplace admin – can resolve disputes
}

// ---------------------------------------------------------------------------
// Contract entry point
// ---------------------------------------------------------------------------
#[contract]
pub struct SariEscrow;

#[contractimpl]
impl SariEscrow {

    // -----------------------------------------------------------------------
    // create_order
    // Buyer calls this to lock USDC into the contract and create a new order.
    // The buyer must have already set a trustline and allowance for `token`.
    // -----------------------------------------------------------------------
    pub fn create_order(
        env: Env,
        buyer: Address,
        seller: Address,
        token: Address,
        amount: i128,
        admin: Address,
    ) -> u64 {
        // Require the buyer to authorise this call (prevents spoofing)
        buyer.require_auth();

        // Validate amount is positive
        if amount <= 0 {
            panic!("amount must be positive");
        }

        // Pull USDC from buyer wallet into this contract
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&buyer, &env.current_contract_address(), &amount);

        // Assign a new order id
        let id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::Counter)
            .unwrap_or(0)
            + 1;

        // Persist order to contract storage
        let order = Order {
            id,
            buyer: buyer.clone(),
            seller: seller.clone(),
            token: token.clone(),
            amount,
            status: OrderStatus::Funded,
            admin,
        };

        env.storage().persistent().set(&DataKey::Order(id), &order);
        env.storage().persistent().set(&DataKey::Counter, &id);

        // Emit an event so off-chain indexers can track new orders
        env.events().publish(
            (symbol_short!("ORDER"), symbol_short!("created")),
            id,
        );

        id // return new order id to caller
    }

    // -----------------------------------------------------------------------
    // mark_shipped
    // Seller confirms goods / service has been dispatched.
    // Only the seller of the specific order may call this.
    // -----------------------------------------------------------------------
    pub fn mark_shipped(env: Env, caller: Address, order_id: u64) {
        caller.require_auth();

        let mut order: Order = env
            .storage()
            .persistent()
            .get(&DataKey::Order(order_id))
            .expect("order not found");

        // Enforce caller is the seller
        if order.seller != caller {
            panic!("only the seller can mark an order as shipped");
        }

        // Order must be in Funded state to advance
        if order.status != OrderStatus::Funded {
            panic!("order is not in Funded state");
        }

        order.status = OrderStatus::Shipped;
        env.storage().persistent().set(&DataKey::Order(order_id), &order);

        env.events().publish(
            (symbol_short!("ORDER"), symbol_short!("shipped")),
            order_id,
        );
    }

    // -----------------------------------------------------------------------
    // confirm_receipt
    // Buyer confirms they received the goods. Releases USDC to seller.
    // This is the core MVP transaction: buyer confirms → seller gets paid.
    // -----------------------------------------------------------------------
    pub fn confirm_receipt(env: Env, caller: Address, order_id: u64) {
        caller.require_auth();

        let mut order: Order = env
            .storage()
            .persistent()
            .get(&DataKey::Order(order_id))
            .expect("order not found");

        // Only the buyer of this order can confirm receipt
        if order.buyer != caller {
            panic!("only the buyer can confirm receipt");
        }

        // Order must be Shipped before buyer can confirm
        if order.status != OrderStatus::Shipped {
            panic!("order has not been marked as shipped yet");
        }

        // Transfer USDC from contract to seller
        let token_client = token::Client::new(&env, &order.token);
        token_client.transfer(
            &env.current_contract_address(),
            &order.seller,
            &order.amount,
        );

        order.status = OrderStatus::Completed;
        env.storage().persistent().set(&DataKey::Order(order_id), &order);

        env.events().publish(
            (symbol_short!("ORDER"), symbol_short!("complete")),
            order_id,
        );
    }

    // -----------------------------------------------------------------------
    // raise_dispute
    // Buyer can dispute an order (Funded or Shipped), freezing funds.
    // -----------------------------------------------------------------------
    pub fn raise_dispute(env: Env, caller: Address, order_id: u64) {
        caller.require_auth();

        let mut order: Order = env
            .storage()
            .persistent()
            .get(&DataKey::Order(order_id))
            .expect("order not found");

        if order.buyer != caller {
            panic!("only the buyer can raise a dispute");
        }

        if order.status != OrderStatus::Funded && order.status != OrderStatus::Shipped {
            panic!("order cannot be disputed in its current state");
        }

        order.status = OrderStatus::Disputed;
        env.storage().persistent().set(&DataKey::Order(order_id), &order);

        env.events().publish(
            (symbol_short!("ORDER"), symbol_short!("dispute")),
            order_id,
        );
    }

    // -----------------------------------------------------------------------
    // resolve_dispute
    // Admin resolves a dispute. refund_buyer=true sends USDC back to buyer,
    // false releases to seller.
    // -----------------------------------------------------------------------
    pub fn resolve_dispute(
        env: Env,
        caller: Address,
        order_id: u64,
        refund_buyer: bool,
    ) {
        caller.require_auth();

        let mut order: Order = env
            .storage()
            .persistent()
            .get(&DataKey::Order(order_id))
            .expect("order not found");

        // Only the designated admin may resolve
        if order.admin != caller {
            panic!("only the admin can resolve disputes");
        }

        if order.status != OrderStatus::Disputed {
            panic!("order is not in Disputed state");
        }

        let token_client = token::Client::new(&env, &order.token);
        let recipient = if refund_buyer {
            order.status = OrderStatus::Refunded;
            order.buyer.clone()
        } else {
            order.status = OrderStatus::Completed;
            order.seller.clone()
        };

        token_client.transfer(
            &env.current_contract_address(),
            &recipient,
            &order.amount,
        );

        env.storage().persistent().set(&DataKey::Order(order_id), &order);

        env.events().publish(
            (symbol_short!("ORDER"), symbol_short!("resolved")),
            order_id,
        );
    }

    // -----------------------------------------------------------------------
    // get_order – read-only view of a stored order
    // -----------------------------------------------------------------------
    pub fn get_order(env: Env, order_id: u64) -> Order {
        env.storage()
            .persistent()
            .get(&DataKey::Order(order_id))
            .expect("order not found")
    }
}